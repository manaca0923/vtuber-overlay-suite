use crate::youtube::{
    client::YouTubeClient, poller::ChatPoller, poller::PollingEvent, state::PollingState,
    types::ChatMessage,
};
use crate::{server::types::WsMessage, AppState};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};

#[tauri::command]
pub async fn validate_api_key(api_key: String) -> Result<bool, String> {
    let client = YouTubeClient::new(api_key);
    client.validate_api_key().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_live_chat_id(api_key: String, video_id: String) -> Result<String, String> {
    let client = YouTubeClient::new(api_key);
    client
        .get_live_chat_id(&video_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_chat_messages(
    api_key: String,
    live_chat_id: String,
    page_token: Option<String>,
) -> Result<(Vec<ChatMessage>, Option<String>, u64), String> {
    let client = YouTubeClient::new(api_key);
    let response = client
        .get_live_chat_messages(&live_chat_id, page_token.as_deref())
        .await
        .map_err(|e| e.to_string())?;

    // レスポンスをChatMessage型に変換
    let total_items = response.items.len();
    let mut parse_errors = 0;

    let messages: Vec<ChatMessage> = response
        .items
        .into_iter()
        .filter_map(|item| {
            use chrono::DateTime;

            // publishedAtのパースに失敗した場合はそのメッセージをスキップ
            let published_at = match DateTime::parse_from_rfc3339(&item.snippet.published_at) {
                Ok(dt) => dt.with_timezone(&chrono::Utc),
                Err(e) => {
                    parse_errors += 1;
                    log::warn!(
                        "Failed to parse publishedAt for message {}: {}. Skipping message.",
                        item.id,
                        e
                    );
                    return None;
                }
            };

            // snippet.message_typeをパースしてMessageTypeを設定（共通関数を使用）
            let message_type = crate::youtube::types::parse_message_type(&item.snippet);

            Some(ChatMessage {
                id: item.id,
                message: item.snippet.display_message,
                author_name: item.author_details.display_name,
                author_channel_id: item.author_details.channel_id,
                author_image_url: item.author_details.profile_image_url,
                published_at,
                is_owner: item.author_details.is_chat_owner,
                is_moderator: item.author_details.is_chat_moderator,
                is_member: item.author_details.is_chat_sponsor,
                is_verified: item.author_details.is_verified,
                message_type,
            })
        })
        .collect();

    // 多数のパースエラーが発生した場合は警告
    if parse_errors > 0 {
        log::warn!(
            "Skipped {} messages due to parse errors (total: {})",
            parse_errors,
            total_items
        );
    }

    // 半分以上のメッセージでパースエラーが発生した場合はエラーを返す
    if parse_errors > total_items / 2 && total_items > 0 {
        return Err(format!(
            "多数のメッセージパースエラーが発生しました ({}/{}件)",
            parse_errors, total_items
        ));
    }

    Ok((
        messages,
        response.next_page_token,
        response.polling_interval_millis,
    ))
}

/// ポーリングを開始
#[tauri::command]
pub async fn start_polling(
    api_key: String,
    live_chat_id: String,
    next_page_token: Option<String>,
    quota_used: Option<u64>,
    polling_interval_millis: Option<u64>,
    app: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    log::info!("Starting polling for live chat ID: {}", live_chat_id);

    // 新しいポーラーを作成（ロックの外で）
    let poller = ChatPoller::new(api_key);

    // 既存のポーラーを停止して新しいポーラーを設定
    {
        let mut poller_lock = state
            .poller
            .lock()
            .map_err(|e| format!("Failed to acquire poller lock: {}", e))?;
        if let Some(existing_poller) = poller_lock.as_ref() {
            if existing_poller.is_running() {
                existing_poller.stop();
                log::info!("Stopped existing poller");
            }
        }
        *poller_lock = Some(poller.clone());
    } // ここでロックを解放

    // WebSocketサーバー状態を取得
    let server_state = Arc::clone(&state.server);

    // イベントコールバックを設定
    let app_clone = app.clone();
    let event_callback = move |event: PollingEvent| {
        // Tauriアプリへのイベント送信
        if let Err(e) = app_clone.emit("polling-event", &event) {
            log::error!("Failed to emit polling event: {}", e);
        }

        // WebSocketでブロードキャスト
        if let PollingEvent::Messages { messages } = event {
            let server_state_clone = Arc::clone(&server_state);
            let messages_clone = messages.clone();
            tokio::spawn(async move {
                let state_lock = server_state_clone.read().await;
                for message in messages_clone {
                    state_lock
                        .broadcast(WsMessage::CommentAdd {
                            payload: message.clone(),
                        })
                        .await;
                }
            });
        }
    };

    // ポーリングを開始（ロックの外で）
    poller
        .start_with_state(
            live_chat_id,
            next_page_token,
            quota_used.unwrap_or(0),
            polling_interval_millis,
            event_callback,
        )
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// ポーリングを停止
#[tauri::command]
pub async fn stop_polling(state: tauri::State<'_, AppState>) -> Result<(), String> {
    log::info!("Stopping polling");

    let poller_lock = state
        .poller
        .lock()
        .map_err(|e| format!("Failed to acquire poller lock: {}", e))?;
    if let Some(poller) = poller_lock.as_ref() {
        poller.stop();
        log::info!("Poller stopped");
    }

    Ok(())
}

/// ポーリング状態を取得
#[tauri::command]
pub async fn get_polling_state(
    state: tauri::State<'_, AppState>,
) -> Result<Option<PollingState>, String> {
    let poller_lock = state
        .poller
        .lock()
        .map_err(|e| format!("Failed to acquire poller lock: {}", e))?;
    if let Some(poller) = poller_lock.as_ref() {
        Ok(poller.get_state())
    } else {
        Ok(None)
    }
}

/// クォータ情報を取得
#[tauri::command]
pub async fn get_quota_info(state: tauri::State<'_, AppState>) -> Result<(u64, i64), String> {
    let poller_lock = state
        .poller
        .lock()
        .map_err(|e| format!("Failed to acquire poller lock: {}", e))?;
    if let Some(poller) = poller_lock.as_ref() {
        if let Some(polling_state) = poller.get_state() {
            Ok((
                polling_state.quota_used,
                polling_state.estimated_remaining_quota(),
            ))
        } else {
            Ok((0, 10_000))
        }
    } else {
        Ok((0, 10_000))
    }
}

/// ポーリングが実行中かどうかを確認
#[tauri::command]
pub async fn is_polling_running(state: tauri::State<'_, AppState>) -> Result<bool, String> {
    let poller_lock = state
        .poller
        .lock()
        .map_err(|e| format!("Failed to acquire poller lock: {}", e))?;
    if let Some(poller) = poller_lock.as_ref() {
        Ok(poller.is_running())
    } else {
        Ok(false)
    }
}

/// ポーリング状態をDBに保存
#[tauri::command]
pub async fn save_polling_state(
    live_chat_id: String,
    next_page_token: Option<String>,
    quota_used: u64,
    polling_interval_millis: u64,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let pool = &state.db;
    let now = chrono::Utc::now().to_rfc3339();

    // JSON形式でポーリング状態を保存
    let polling_data = serde_json::json!({
        "live_chat_id": live_chat_id,
        "next_page_token": next_page_token,
        "quota_used": quota_used,
        "polling_interval_millis": polling_interval_millis,
        "saved_at": now
    });
    let polling_data_str =
        serde_json::to_string(&polling_data).map_err(|e| format!("JSON serialize error: {}", e))?;

    // settingsテーブルにUPSERT
    sqlx::query!(
        r#"
        INSERT INTO settings (key, value, updated_at)
        VALUES ('polling_state', ?, ?)
        ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at
        "#,
        polling_data_str,
        now
    )
    .execute(pool)
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    log::info!("Saved polling state for live_chat_id: {}", live_chat_id);
    Ok(())
}

/// 保存されたポーリング状態をDBから読み込む
#[tauri::command]
pub async fn load_polling_state(
    state: tauri::State<'_, AppState>,
) -> Result<Option<PollingStateData>, String> {
    let pool = &state.db;

    let result: Option<String> = sqlx::query_scalar(
        "SELECT value FROM settings WHERE key = 'polling_state'"
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    if let Some(json_str) = result {
        let data: PollingStateData =
            serde_json::from_str(&json_str).map_err(|e| format!("JSON parse error: {}", e))?;
        Ok(Some(data))
    } else {
        Ok(None)
    }
}

/// 保存されたポーリング状態のデータ構造
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PollingStateData {
    pub live_chat_id: String,
    pub next_page_token: Option<String>,
    pub quota_used: u64,
    #[serde(default)]
    pub polling_interval_millis: Option<u64>,
    pub saved_at: String,
}

/// テストモード: ダミーコメントを送信
#[tauri::command]
pub async fn send_test_comment(
    comment_text: String,
    author_name: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    use crate::youtube::types::MessageType;
    use chrono::Utc;

    // ダミーコメント作成
    let test_message = ChatMessage {
        id: format!("test-{}", Utc::now().timestamp_millis()),
        message: comment_text,
        author_name,
        author_channel_id: "test-channel".to_string(),
        author_image_url: "https://via.placeholder.com/48".to_string(),
        published_at: Utc::now(),
        is_owner: false,
        is_moderator: false,
        is_member: false,
        is_verified: false,
        message_type: MessageType::Text,
    };

    // WebSocketでブロードキャスト
    let server_state = Arc::clone(&state.server);
    let state_lock = server_state.read().await;
    state_lock
        .broadcast(WsMessage::CommentAdd {
            payload: test_message,
        })
        .await;

    Ok(())
}

/// ウィザード設定を保存（videoId, liveChatId）
#[tauri::command]
pub async fn save_wizard_settings(
    video_id: String,
    live_chat_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let pool = &state.db;
    let now = chrono::Utc::now().to_rfc3339();

    // JSON形式でウィザード設定を保存
    let settings_data = serde_json::json!({
        "video_id": video_id,
        "live_chat_id": live_chat_id,
        "saved_at": now
    });
    let settings_str =
        serde_json::to_string(&settings_data).map_err(|e| format!("JSON serialize error: {}", e))?;

    // settingsテーブルにUPSERT
    sqlx::query!(
        r#"
        INSERT INTO settings (key, value, updated_at)
        VALUES ('wizard_settings', ?, ?)
        ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at
        "#,
        settings_str,
        now
    )
    .execute(pool)
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    log::info!("Saved wizard settings: video_id={}, live_chat_id={}", video_id, live_chat_id);
    Ok(())
}

/// 保存されたウィザード設定を読み込む
#[tauri::command]
pub async fn load_wizard_settings(
    state: tauri::State<'_, AppState>,
) -> Result<Option<WizardSettingsData>, String> {
    let pool = &state.db;

    let result: Option<String> = sqlx::query_scalar(
        "SELECT value FROM settings WHERE key = 'wizard_settings'"
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    if let Some(json_str) = result {
        let data: WizardSettingsData =
            serde_json::from_str(&json_str).map_err(|e| format!("JSON parse error: {}", e))?;
        Ok(Some(data))
    } else {
        Ok(None)
    }
}

/// ウィザード設定のデータ構造
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WizardSettingsData {
    pub video_id: String,
    pub live_chat_id: String,
    pub saved_at: String,
}

