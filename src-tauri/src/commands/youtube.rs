use crate::youtube::{
    api_key_manager::get_api_key_manager,
    client::YouTubeClient,
    db::save_comments_to_db,
    innertube,
    poller::ChatPoller,
    poller::PollingEvent,
    state::PollingState,
    types::ChatMessage,
};
use crate::{server::types::WsMessage, AppState};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};

/// ポーラー停止後のグレースフルシャットダウン待機時間（ミリ秒）
const POLLER_GRACEFUL_SHUTDOWN_MS: u64 = 200;

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
                message_runs: None, // 公式APIでは絵文字情報なし
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

    // 相互排他: InnerTubeポーリングが動いていたら即時停止（JoinHandleをabort）
    {
        let mut handle_lock = get_innertube_handle().lock().await;
        if let Some(handle) = handle_lock.take() {
            log::info!("Aborting InnerTube polling task (mutual exclusion)");
            handle.abort();
        }
    }
    get_innertube_running().store(false, Ordering::SeqCst);

    // 相互排他: 統合ポーラーが動いていたら停止してUI通知
    {
        let poller = get_unified_poller().lock().await;
        if poller.is_running() {
            log::info!("Stopping unified polling (mutual exclusion)");
            poller.stop().await;
            // UI更新のためStopped通知を送信
            if let Err(e) = app.emit("polling-event", PollingEvent::Stopped {
                reason: "公式APIポーリングに切り替え".to_string(),
            }) {
                log::error!("Failed to emit stopped event: {}", e);
            }
        }
    }

    // 新しいポーラーを作成（ロックの外で）
    let poller = ChatPoller::new(api_key);

    // 既存のポーラーを停止（ロックを解放してから待機）
    let needs_wait = {
        let poller_lock = state
            .poller
            .lock()
            .map_err(|e| format!("Failed to acquire poller lock: {}", e))?;
        if let Some(existing_poller) = poller_lock.as_ref() {
            if existing_poller.is_running() {
                existing_poller.stop();
                log::info!("Stopped existing poller, waiting for graceful shutdown...");
                true
            } else {
                false
            }
        } else {
            false
        }
    }; // ここでロック解放

    // ロック解放後に待機（二重ポーリング防止）
    if needs_wait {
        tokio::time::sleep(std::time::Duration::from_millis(POLLER_GRACEFUL_SHUTDOWN_MS)).await;
    }

    // 新しいポーラーを設定
    {
        let mut poller_lock = state
            .poller
            .lock()
            .map_err(|e| format!("Failed to reacquire poller lock: {}", e))?;
        *poller_lock = Some(poller.clone());
    } // ここでロック解放

    // WebSocketサーバー状態を取得
    let server_state = Arc::clone(&state.server);

    // DBプールを取得（コメントログ保存用）
    let db_pool = state.db.clone();

    // イベントコールバックを設定
    let app_clone = app.clone();
    let event_callback = move |event: PollingEvent| {
        // Tauriアプリへのイベント送信
        if let Err(e) = app_clone.emit("polling-event", &event) {
            log::error!("Failed to emit polling event: {}", e);
        }

        // WebSocketでブロードキャスト & DBに保存
        if let PollingEvent::Messages { messages } = event {
            let server_state_clone = Arc::clone(&server_state);
            let db_pool_clone = db_pool.clone();
            let messages_clone = messages.clone();
            tokio::spawn(async move {
                // DBに保存
                save_comments_to_db(&db_pool_clone, &messages_clone).await;

                // WebSocketでブロードキャスト
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

/// ポーリング状態の有効期限（24時間）
const POLLING_STATE_EXPIRY_HOURS: i64 = 24;

/// 保存されたポーリング状態をDBから読み込む
/// 有効期限（24時間）を超えた状態は無効として削除し、Noneを返す
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

        // 有効期限チェック
        if let Ok(saved_time) = chrono::DateTime::parse_from_rfc3339(&data.saved_at) {
            let now = chrono::Utc::now();
            let elapsed = now.signed_duration_since(saved_time);

            if elapsed.num_hours() >= POLLING_STATE_EXPIRY_HOURS {
                log::info!(
                    "Polling state expired (saved {} hours ago, limit {} hours). Clearing state.",
                    elapsed.num_hours(),
                    POLLING_STATE_EXPIRY_HOURS
                );

                // 期限切れの状態を削除
                sqlx::query("DELETE FROM settings WHERE key = 'polling_state'")
                    .execute(pool)
                    .await
                    .map_err(|e| format!("DB error while clearing expired state: {}", e))?;

                return Ok(None);
            }

            log::debug!(
                "Polling state is valid (saved {} hours ago)",
                elapsed.num_hours()
            );
        } else {
            log::warn!("Failed to parse saved_at timestamp: {}", data.saved_at);
            // パース失敗時は安全のため状態を削除
            sqlx::query("DELETE FROM settings WHERE key = 'polling_state'")
                .execute(pool)
                .await
                .map_err(|e| format!("DB error while clearing invalid state: {}", e))?;
            return Ok(None);
        }

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
    /// ポーリング間隔（ミリ秒）
    /// 後方互換性のためOption<u64>として定義。
    /// v0.1.0以前の保存データにはこのフィールドが存在しないため、
    /// デシリアライズ時にNoneとして扱う。
    #[serde(default)]
    pub polling_interval_millis: Option<u64>,
    pub saved_at: String,
}

/// テストモード: ダミーコメントを送信
/// message_type_name: "text" | "superChat" | "superSticker" | "membership" | "membershipGift"
#[tauri::command]
pub async fn send_test_comment(
    comment_text: String,
    author_name: String,
    message_type_name: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    use crate::youtube::types::MessageType;
    use chrono::Utc;

    // メッセージタイプを決定
    let message_type = match message_type_name.as_deref() {
        Some("superChat") => MessageType::SuperChat {
            amount: "¥1,000".to_string(),
            currency: "JPY".to_string(),
        },
        Some("superSticker") => MessageType::SuperSticker {
            sticker_id: "test-sticker".to_string(),
        },
        Some("membership") => MessageType::Membership {
            level: "New Member".to_string(),
        },
        Some("membershipGift") => MessageType::MembershipGift { count: 5 },
        _ => MessageType::Text,
    };

    // バッジ設定（メンバーシップ系はis_memberをtrueに）
    let is_member = matches!(
        message_type,
        MessageType::Membership { .. } | MessageType::MembershipGift { .. }
    );

    // ダミーコメント作成
    let test_message = ChatMessage {
        id: format!("test-{}", Utc::now().timestamp_millis()),
        message: comment_text,
        author_name,
        author_channel_id: "test-channel".to_string(),
        // シンプルなSVGプレースホルダー（オフライン対応）
        author_image_url: "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='48' height='48' viewBox='0 0 48 48'%3E%3Ccircle cx='24' cy='24' r='24' fill='%236366f1'/%3E%3Ctext x='24' y='30' text-anchor='middle' fill='white' font-size='20'%3E%F0%9F%A7%AA%3C/text%3E%3C/svg%3E".to_string(),
        published_at: Utc::now(),
        is_owner: false,
        is_moderator: false,
        is_member,
        is_verified: false,
        message_type,
        message_runs: None,
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

// ================================
// InnerTube API 関連コマンド
// ================================

/// API取得モードの列挙型
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ApiMode {
    /// 公式API ポーリング（BYOK必須）
    Official,
    /// InnerTube API（非公式、APIキー不要）
    InnerTube,
    /// 公式API gRPCストリーミング（推奨、同梱キー使用可）
    Grpc,
}

impl Default for ApiMode {
    fn default() -> Self {
        ApiMode::Official
    }
}

/// APIモードを保存
#[tauri::command]
pub async fn save_api_mode(
    mode: ApiMode,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let pool = &state.db;
    let now = chrono::Utc::now().to_rfc3339();

    let data = serde_json::json!({
        "api_mode": mode,
        "saved_at": now
    });
    let data_str =
        serde_json::to_string(&data).map_err(|e| format!("JSON serialize error: {}", e))?;

    sqlx::query(
        r#"
        INSERT INTO settings (key, value, updated_at)
        VALUES ('api_mode', ?, ?)
        ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at
        "#,
    )
    .bind(&data_str)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    log::info!("Saved API mode: {:?}", mode);
    Ok(())
}

/// APIモードを読み込み
#[tauri::command]
pub async fn load_api_mode(state: tauri::State<'_, AppState>) -> Result<ApiMode, String> {
    let pool = &state.db;

    let result: Option<String> =
        sqlx::query_scalar("SELECT value FROM settings WHERE key = 'api_mode'")
            .fetch_optional(pool)
            .await
            .map_err(|e| format!("DB error: {}", e))?;

    if let Some(json_str) = result {
        #[derive(serde::Deserialize)]
        struct ApiModeData {
            api_mode: ApiMode,
        }
        let data: ApiModeData =
            serde_json::from_str(&json_str).map_err(|e| format!("JSON parse error: {}", e))?;
        Ok(data.api_mode)
    } else {
        Ok(ApiMode::default())
    }
}

/// InnerTube API接続テスト（開発ビルドのみ有効）
#[cfg(debug_assertions)]
#[tauri::command]
pub async fn test_innertube_connection(video_id: String) -> Result<String, String> {
    use crate::youtube::innertube::{parse_chat_response, InnerTubeClient};

    log::info!("Testing InnerTube connection for video: {}", video_id);

    // クライアント初期化
    let mut client = InnerTubeClient::new(video_id.clone()).map_err(|e| {
        log::error!("InnerTube client creation failed: {}", e);
        format!("クライアント作成に失敗しました: {}", e)
    })?;
    client.initialize().await.map_err(|e| {
        log::error!("InnerTube initialization failed: {}", e);
        format!("初期化に失敗しました: {}", e)
    })?;

    log::info!("InnerTube client initialized");

    // メッセージ取得
    let response = client.get_chat_messages().await.map_err(|e| {
        log::error!("InnerTube message fetch failed: {}", e);
        format!("メッセージ取得に失敗しました: {}", e)
    })?;

    // パース
    let messages = parse_chat_response(response);

    // 統計情報を返す
    let emoji_count = messages
        .iter()
        .filter(|m| m.message_runs.is_some())
        .flat_map(|m| m.message_runs.as_ref().unwrap())
        .filter(|run| matches!(run, crate::youtube::types::MessageRun::Emoji { .. }))
        .count();

    let result = format!(
        "接続成功！\nメッセージ数: {}\nカスタム絵文字数: {}",
        messages.len(),
        emoji_count
    );

    log::info!("{}", result);
    Ok(result)
}

// ================================
// InnerTube ポーリング
// ================================

use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::Mutex as TokioMutex;
use tokio::task::JoinHandle;

// グローバルなInnerTubeポーリング状態
static INNERTUBE_RUNNING: std::sync::OnceLock<Arc<AtomicBool>> = std::sync::OnceLock::new();
static INNERTUBE_CLIENT: std::sync::OnceLock<Arc<TokioMutex<Option<innertube::InnerTubeClient>>>> =
    std::sync::OnceLock::new();
static INNERTUBE_HANDLE: std::sync::OnceLock<Arc<TokioMutex<Option<JoinHandle<()>>>>> =
    std::sync::OnceLock::new();

fn get_innertube_running() -> &'static Arc<AtomicBool> {
    INNERTUBE_RUNNING.get_or_init(|| Arc::new(AtomicBool::new(false)))
}

fn get_innertube_client(
) -> &'static Arc<TokioMutex<Option<innertube::InnerTubeClient>>> {
    INNERTUBE_CLIENT.get_or_init(|| Arc::new(TokioMutex::new(None)))
}

fn get_innertube_handle() -> &'static Arc<TokioMutex<Option<JoinHandle<()>>>> {
    INNERTUBE_HANDLE.get_or_init(|| Arc::new(TokioMutex::new(None)))
}

/// InnerTube APIを使用したポーリングを開始
///
/// 公式APIとは異なり、video_idのみで開始可能。
/// カスタム絵文字の画像URLを含むメッセージを取得可能。
#[tauri::command]
pub async fn start_polling_innertube(
    video_id: String,
    app: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    log::info!(
        "Starting InnerTube polling for video: {}",
        video_id
    );

    // 相互排他: 公式ポーリングが動いていたら停止してUI通知
    {
        let poller_lock = state
            .poller
            .lock()
            .map_err(|e| format!("Failed to acquire poller lock: {}", e))?;
        if let Some(poller) = poller_lock.as_ref() {
            if poller.is_running() {
                log::info!("Stopping official polling (mutual exclusion)");
                poller.stop();
                // UI更新のためStopped通知を送信
                if let Err(e) = app.emit("polling-event", PollingEvent::Stopped {
                    reason: "InnerTubeポーリングに切り替え".to_string(),
                }) {
                    log::error!("Failed to emit stopped event: {}", e);
                }
            }
        }
    }

    // 相互排他: 統合ポーラーが動いていたら停止してUI通知
    {
        let poller = get_unified_poller().lock().await;
        if poller.is_running() {
            log::info!("Stopping unified polling (mutual exclusion)");
            poller.stop().await;
            // UI更新のためStopped通知を送信
            if let Err(e) = app.emit("polling-event", PollingEvent::Stopped {
                reason: "InnerTubeポーリング（旧経路）に切り替え".to_string(),
            }) {
                log::error!("Failed to emit stopped event: {}", e);
            }
        }
    }

    // 既存のInnerTubeポーリングを停止（JoinHandleをabort）
    {
        let mut handle_lock = get_innertube_handle().lock().await;
        if let Some(handle) = handle_lock.take() {
            log::info!("Aborting existing InnerTube polling task");
            handle.abort();
        }
    }
    get_innertube_running().store(false, Ordering::SeqCst);

    // 動画切替のため絵文字キャッシュをクリア
    innertube::clear_emoji_cache();
    log::info!("Cleared emoji cache for new video");

    // クライアントを初期化
    let mut client = innertube::InnerTubeClient::new(video_id.clone()).map_err(|e| {
        log::error!("InnerTube client creation failed: {}", e);
        format!("InnerTubeクライアント作成に失敗しました: {}", e)
    })?;

    client.initialize().await.map_err(|e| {
        log::error!("InnerTube initialization failed: {}", e);
        format!("InnerTube初期化に失敗しました: {}", e)
    })?;

    log::info!("InnerTube client initialized successfully");

    // クライアントを保存
    {
        let mut client_lock = get_innertube_client().lock().await;
        *client_lock = Some(client);
    }

    // ポーリング開始フラグを設定
    get_innertube_running().store(true, Ordering::SeqCst);

    // WebSocketサーバー状態を取得
    let server_state = Arc::clone(&state.server);

    // DBプールを取得（コメントログ保存用）
    let db_pool = state.db.clone();

    // ポーリングループを開始（JoinHandleを保持）
    let running = Arc::clone(get_innertube_running());
    let client_mutex = Arc::clone(get_innertube_client());

    let handle = tokio::spawn(async move {
        log::info!("InnerTube polling loop started");

        // 重複排除用のメッセージIDセット（挿入順を維持）
        let mut seen_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut seen_order: std::collections::VecDeque<String> = std::collections::VecDeque::new();
        const MAX_SEEN_IDS: usize = 10000;

        while running.load(Ordering::SeqCst) {
            let timeout_ms;

            // メッセージ取得
            let messages = {
                let mut client_lock = client_mutex.lock().await;
                if let Some(client) = client_lock.as_mut() {
                    match client.get_chat_messages().await {
                        Ok(response) => {
                            timeout_ms = client.get_timeout_ms();
                            innertube::parse_chat_response(response)
                        }
                        Err(e) => {
                            log::error!("InnerTube fetch error: {}", e);
                            timeout_ms = 5000;
                            vec![]
                        }
                    }
                } else {
                    log::error!("InnerTube client not initialized");
                    break;
                }
            };

            // 新しいメッセージのみフィルタリング
            let new_messages: Vec<ChatMessage> = messages
                .into_iter()
                .filter(|m| !seen_ids.contains(&m.id))
                .collect();

            // 見たIDを記録（順序も維持、重複はスキップ）
            for message in &new_messages {
                // insertはtrueを返す＝新規追加、falseは既存（同一レスポンス内重複）
                if seen_ids.insert(message.id.clone()) {
                    seen_order.push_back(message.id.clone());
                }
            }

            // メモリリーク防止: 古いIDから削除（FIFO方式で一定数に保つ）
            while seen_ids.len() > MAX_SEEN_IDS {
                if let Some(oldest_id) = seen_order.pop_front() {
                    seen_ids.remove(&oldest_id);
                } else {
                    break;
                }
            }

            if !new_messages.is_empty() {
                log::debug!(
                    "InnerTube: {} new messages (total seen: {})",
                    new_messages.len(),
                    seen_ids.len()
                );

                // Tauriアプリへのイベント送信
                let event = PollingEvent::Messages {
                    messages: new_messages.clone(),
                };
                if let Err(e) = app.emit("polling-event", &event) {
                    log::error!("Failed to emit polling event: {}", e);
                }

                // DBに保存
                save_comments_to_db(&db_pool, &new_messages).await;

                // WebSocketでブロードキャスト
                let server_state_clone = Arc::clone(&server_state);
                for message in new_messages {
                    let state_lock = server_state_clone.read().await;
                    state_lock
                        .broadcast(WsMessage::CommentAdd {
                            payload: message,
                        })
                        .await;
                }
            }

            // 次のポーリングまで待機
            let wait_ms = std::cmp::max(timeout_ms, 3000); // 最低3秒
            tokio::time::sleep(tokio::time::Duration::from_millis(wait_ms)).await;
        }

        log::info!("InnerTube polling loop ended");
    });

    // JoinHandleを保存
    {
        let mut handle_lock = get_innertube_handle().lock().await;
        *handle_lock = Some(handle);
    }

    Ok(())
}

/// InnerTubeポーリングを停止
#[tauri::command]
pub async fn stop_polling_innertube() -> Result<(), String> {
    log::info!("Stopping InnerTube polling");
    get_innertube_running().store(false, Ordering::SeqCst);

    // JoinHandleをabort
    {
        let mut handle_lock = get_innertube_handle().lock().await;
        if let Some(handle) = handle_lock.take() {
            handle.abort();
        }
    }

    // クライアントをクリア
    {
        let mut client_lock = get_innertube_client().lock().await;
        *client_lock = None;
    }

    Ok(())
}

/// InnerTubeポーリングが実行中かどうかを確認
#[tauri::command]
pub async fn is_polling_innertube_running() -> Result<bool, String> {
    Ok(get_innertube_running().load(Ordering::SeqCst))
}

// ================================
// APIキー管理コマンド
// ================================

/// APIキー状態の情報
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ApiKeyStatus {
    /// 同梱キーが利用可能か
    pub has_bundled_key: bool,
    /// BYOKが設定されているか
    pub has_user_key: bool,
    /// 現在Secondaryキーを使用中か
    pub using_secondary: bool,
    /// ステータスサマリー（デバッグ用）
    pub summary: String,
}

/// APIキー状態を取得
#[tauri::command]
pub async fn get_api_key_status() -> Result<ApiKeyStatus, String> {
    let manager = get_api_key_manager()
        .read()
        .map_err(|e| format!("Failed to read API key manager: {}", e))?;

    Ok(ApiKeyStatus {
        has_bundled_key: manager.has_bundled_key(),
        has_user_key: manager.has_user_key(),
        using_secondary: manager.is_using_secondary(),
        summary: manager.status_summary(),
    })
}

/// 同梱APIキーが利用可能かどうかを確認
#[tauri::command]
pub async fn has_bundled_api_key() -> Result<bool, String> {
    let manager = get_api_key_manager()
        .read()
        .map_err(|e| format!("Failed to read API key manager: {}", e))?;

    Ok(manager.has_bundled_key())
}

/// BYOKキーを設定
#[tauri::command]
pub async fn set_byok_key(api_key: Option<String>) -> Result<(), String> {
    let mut manager = get_api_key_manager()
        .write()
        .map_err(|e| format!("Failed to write API key manager: {}", e))?;

    manager.set_user_key(api_key.clone());

    if api_key.is_some() {
        log::info!("BYOK key has been set");
    } else {
        log::info!("BYOK key has been cleared");
    }

    Ok(())
}

/// 有効なAPIキーを取得（内部使用）
/// prefer_bundled: true=同梱キー優先、false=BYOK優先
#[tauri::command]
pub async fn get_active_api_key(prefer_bundled: bool) -> Result<Option<String>, String> {
    let manager = get_api_key_manager()
        .read()
        .map_err(|e| format!("Failed to read API key manager: {}", e))?;

    Ok(manager.get_active_key(prefer_bundled).map(|s| s.to_string()))
}

/// Secondaryキーにフォールバック
#[tauri::command]
pub async fn switch_to_secondary_key() -> Result<(), String> {
    let manager = get_api_key_manager()
        .read()
        .map_err(|e| format!("Failed to read API key manager: {}", e))?;

    manager.switch_to_secondary();
    Ok(())
}

/// Primaryキーにリセット
#[tauri::command]
pub async fn reset_to_primary_key() -> Result<(), String> {
    let manager = get_api_key_manager()
        .read()
        .map_err(|e| format!("Failed to read API key manager: {}", e))?;

    manager.reset_to_primary();
    Ok(())
}

// ================================
// 統合ポーラーコマンド
// ================================
//
// NOTE: 将来的にはこの統合ポーラー（UNIFIED_POLLER）に既存の個別ポーラー
// （INNERTUBE_RUNNING, INNERTUBE_CLIENT, INNERTUBE_HANDLE, AppState.poller）を
// 統合することを検討してください。現在は並行して存在していますが、
// 一元管理することでコードの保守性が向上します。
// @see docs/900_tasks.md

use crate::youtube::unified_poller::UnifiedPoller;

// グローバルな統合ポーラー状態
static UNIFIED_POLLER: std::sync::OnceLock<Arc<TokioMutex<UnifiedPoller>>> = std::sync::OnceLock::new();

fn get_unified_poller() -> &'static Arc<TokioMutex<UnifiedPoller>> {
    UNIFIED_POLLER.get_or_init(|| Arc::new(TokioMutex::new(UnifiedPoller::new())))
}

/// 統合ポーリングを開始
///
/// 3つのモード（InnerTube / Official / gRPC）のいずれかでポーリングを開始する。
/// 既存のポーリングは自動的に停止される。
///
/// ## WS/DB連携
/// 取得したコメントは以下の経路で配信される：
/// 1. Tauriイベント（chat-messages）→ フロントエンドUI
/// 2. WebSocketブロードキャスト（comment:add）→ OBSオーバーレイ
/// 3. SQLite保存 → コメントログ
#[tauri::command]
pub async fn start_unified_polling(
    video_id: String,
    mode: ApiMode,
    use_bundled_key: bool,
    user_api_key: Option<String>,
    app: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    log::info!(
        "Starting unified polling: mode={:?}, video_id={}, use_bundled_key={}",
        mode,
        video_id,
        use_bundled_key
    );

    // 旧ポーラーを停止（二重ポーリング防止）
    // 1. 公式APIポーラー（ChatPoller）を停止
    if let Ok(poller_lock) = state.poller.lock() {
        if let Some(poller) = poller_lock.as_ref() {
            poller.stop();
            log::info!("Stopped legacy Official API poller before starting unified polling");
        }
    }

    // 2. InnerTubeポーラーを停止
    get_innertube_running().store(false, Ordering::SeqCst);
    {
        let mut handle_lock = get_innertube_handle().lock().await;
        if let Some(handle) = handle_lock.take() {
            handle.abort();
            log::info!("Stopped legacy InnerTube poller before starting unified polling");
        }
    }

    let poller = get_unified_poller().lock().await;

    // AppStateからDBプールとWebSocketサーバー状態を取得
    let db_pool = state.db.clone();
    let server_state = std::sync::Arc::clone(&state.server);

    poller
        .start(video_id, mode, use_bundled_key, user_api_key, app, db_pool, server_state)
        .await
        .map_err(|e| format!("{}", e))
}

/// 統合ポーリングを停止
#[tauri::command]
pub async fn stop_unified_polling() -> Result<(), String> {
    log::info!("Stopping unified polling");

    let poller = get_unified_poller().lock().await;
    poller.stop().await;

    Ok(())
}

/// 統合ポーリングが実行中かどうかを確認
#[tauri::command]
pub async fn is_unified_polling_running() -> Result<bool, String> {
    let poller = get_unified_poller().lock().await;
    Ok(poller.is_running())
}

/// 現在のAPIモードを取得
#[tauri::command]
pub async fn get_unified_polling_mode() -> Result<Option<ApiMode>, String> {
    let poller = get_unified_poller().lock().await;
    Ok(poller.current_mode().await)
}

// ================================
// KPI（視聴者数等）コマンド
// ================================

use crate::server::types::KpiUpdatePayload;
use crate::youtube::types::LiveStreamStats;

/// ライブ配信の統計情報を取得
///
/// 視聴者数、高評価数、再生回数を取得。
/// APIキーが必要（同梱キーまたはBYOK）。
/// クォータ消費: 約3 units
#[tauri::command]
pub async fn get_live_stream_stats(
    video_id: String,
    use_bundled_key: bool,
) -> Result<LiveStreamStats, String> {
    log::debug!(
        "Fetching live stream stats: video_id={}, use_bundled_key={}",
        video_id,
        use_bundled_key
    );

    // APIキーを取得
    let api_key = {
        let manager = get_api_key_manager()
            .read()
            .map_err(|e| format!("Failed to read API key manager: {}", e))?;

        manager
            .get_active_key(use_bundled_key)
            .map(|s| s.to_string())
            .ok_or_else(|| "APIキーが設定されていません".to_string())?
    };

    let client = YouTubeClient::new(api_key);
    client
        .get_live_stream_stats(&video_id)
        .await
        .map_err(|e| e.to_string())
}

/// KPI情報をWebSocketでブロードキャスト
///
/// 視聴者数と高評価数をオーバーレイに配信
#[tauri::command]
pub async fn broadcast_kpi_update(
    main: Option<i64>,
    label: Option<String>,
    sub: Option<i64>,
    sub_label: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let payload = KpiUpdatePayload {
        main,
        label,
        sub,
        sub_label,
    };

    let ws_state = state.server.read().await;
    ws_state
        .broadcast(crate::server::types::WsMessage::KpiUpdate { payload })
        .await;

    log::debug!("KPI update broadcasted");
    Ok(())
}

