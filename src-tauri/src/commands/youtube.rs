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

            // snippet.message_typeをパースしてMessageTypeを設定
            let message_type = match item.snippet.message_type.as_str() {
                "textMessageEvent" => crate::youtube::types::MessageType::Text,
                "superChatEvent" => {
                    if let Some(details) = &item.snippet.super_chat_details {
                        crate::youtube::types::MessageType::SuperChat {
                            amount: details.amount_display_string.clone(),
                            currency: details.currency.clone(),
                        }
                    } else {
                        log::warn!("superChatEvent without superChatDetails for message {}", item.id);
                        crate::youtube::types::MessageType::Text
                    }
                }
                "superStickerEvent" => crate::youtube::types::MessageType::SuperSticker {
                    sticker_id: String::new(), // TODO: スーパーステッカーの詳細実装
                },
                "newSponsorEvent" => crate::youtube::types::MessageType::Membership {
                    level: String::new(), // TODO: メンバーシップレベル取得
                },
                "membershipGiftingEvent" => crate::youtube::types::MessageType::MembershipGift {
                    count: 1, // TODO: ギフト数取得
                },
                _ => {
                    log::debug!("Unknown message type: {}", item.snippet.message_type);
                    crate::youtube::types::MessageType::Text
                }
            };

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
        .start(live_chat_id, event_callback)
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

/// セットリスト更新をブロードキャスト（ダミー実装 - T06で実データに置き換え）
#[tauri::command]
pub async fn broadcast_setlist_update(state: tauri::State<'_, AppState>) -> Result<(), String> {
    use crate::server::types::{SetlistUpdatePayload, SongItem, SongStatus, WsMessage};

    // ダミーデータ
    let dummy_setlist = SetlistUpdatePayload {
        current_index: 1,
        songs: vec![
            SongItem {
                id: "1".to_string(),
                title: "前の曲".to_string(),
                artist: "アーティスト".to_string(),
                status: SongStatus::Done,
            },
            SongItem {
                id: "2".to_string(),
                title: "現在の曲".to_string(),
                artist: "アーティスト".to_string(),
                status: SongStatus::Current,
            },
            SongItem {
                id: "3".to_string(),
                title: "次の曲".to_string(),
                artist: "アーティスト".to_string(),
                status: SongStatus::Pending,
            },
        ],
    };

    // WebSocketでブロードキャスト
    let server_state = Arc::clone(&state.server);
    tokio::spawn(async move {
        let state_lock = server_state.read().await;
        state_lock
            .broadcast(WsMessage::SetlistUpdate {
                payload: dummy_setlist,
            })
            .await;
    });

    log::info!("Broadcasted dummy setlist update");
    Ok(())
}
