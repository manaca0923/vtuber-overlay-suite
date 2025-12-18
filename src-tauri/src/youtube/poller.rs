use super::{
    backoff::ExponentialBackoff, client::YouTubeClient, errors::YouTubeError, state::PollingState,
    types::ChatMessage,
};
use serde::{Deserialize, Serialize};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use tokio::time::sleep;

/// ポーリングイベント
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum PollingEvent {
    /// 新しいメッセージを受信
    #[serde(rename = "messages")]
    Messages { messages: Vec<ChatMessage> },

    /// ポーリング開始
    #[serde(rename = "started")]
    Started { live_chat_id: String },

    /// ポーリング停止
    #[serde(rename = "stopped")]
    Stopped { reason: String },

    /// エラー発生（自動再試行する）
    #[serde(rename = "error")]
    Error { message: String, retrying: bool },

    /// クォータ不足（停止）
    #[serde(rename = "quotaExceeded")]
    QuotaExceeded,

    /// 配信終了検出
    #[serde(rename = "streamEnded")]
    StreamEnded,

    /// 状態更新
    #[serde(rename = "stateUpdate")]
    StateUpdate {
        quota_used: u64,
        remaining_quota: i64,
        poll_count: u64,
    },
}

/// YouTubeコメントポーリングマネージャー
pub struct ChatPoller {
    client: YouTubeClient,
    state: Arc<Mutex<Option<PollingState>>>,
    is_running: Arc<AtomicBool>,
    backoff: Arc<Mutex<ExponentialBackoff>>,
}

impl ChatPoller {
    /// 新しいポーラーを作成
    pub fn new(api_key: String) -> Self {
        Self {
            client: YouTubeClient::new(api_key),
            state: Arc::new(Mutex::new(None)),
            is_running: Arc::new(AtomicBool::new(false)),
            backoff: Arc::new(Mutex::new(ExponentialBackoff::new())),
        }
    }

    /// ポーリングを開始
    ///
    /// # 引数
    /// - `live_chat_id`: ライブチャットID
    /// - `event_callback`: イベント発生時に呼ばれるコールバック
    pub async fn start<F>(
        &self,
        live_chat_id: String,
        event_callback: F,
    ) -> Result<(), YouTubeError>
    where
        F: Fn(PollingEvent) + Send + Sync + 'static,
    {
        // 既に実行中の場合はエラー
        if self.is_running.load(Ordering::SeqCst) {
            return Err(YouTubeError::PollerAlreadyRunning);
        }

        // 状態を初期化
        {
            let mut state = self.state.lock().map_err(|e| {
                log::error!("Failed to acquire state lock: {}", e);
                YouTubeError::ParseError("Failed to initialize poller state".to_string())
            })?;
            *state = Some(PollingState::new(live_chat_id.clone()));
        }

        self.is_running.store(true, Ordering::SeqCst);

        // 開始イベントを送信
        event_callback(PollingEvent::Started {
            live_chat_id: live_chat_id.clone(),
        });

        // ポーリングループを別タスクで開始
        let client = self.client.clone();
        let state = Arc::clone(&self.state);
        let is_running = Arc::clone(&self.is_running);
        let backoff = Arc::clone(&self.backoff);

        tokio::spawn(async move {
            Self::polling_loop(client, state, is_running, backoff, event_callback).await;
        });

        Ok(())
    }

    /// ポーリングを停止
    pub fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);
    }

    /// ポーリング中かどうかを確認
    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }

    /// 現在の状態を取得
    pub fn get_state(&self) -> Option<PollingState> {
        self.state
            .lock()
            .ok()
            .and_then(|state| state.clone())
    }

    /// ポーリングループ（内部実装）
    async fn polling_loop<F>(
        client: YouTubeClient,
        state: Arc<Mutex<Option<PollingState>>>,
        is_running: Arc<AtomicBool>,
        backoff: Arc<Mutex<ExponentialBackoff>>,
        event_callback: F,
    ) where
        F: Fn(PollingEvent) + Send + Sync + 'static,
    {
        while is_running.load(Ordering::SeqCst) {
            // 現在の状態を取得
            let (live_chat_id, page_token, polling_interval) = {
                let state_lock = match state.lock() {
                    Ok(lock) => lock,
                    Err(e) => {
                        log::error!("Failed to acquire state lock: {}", e);
                        event_callback(PollingEvent::Error {
                            message: "内部エラーが発生しました".to_string(),
                            retrying: false,
                        });
                        break;
                    }
                };
                if let Some(s) = state_lock.as_ref() {
                    (
                        s.live_chat_id.clone(),
                        s.next_page_token.clone(),
                        s.polling_interval(),
                    )
                } else {
                    log::warn!("Polling state is None, stopping poller");
                    break;
                }
            };

            // コメントを取得
            match client
                .get_live_chat_messages(&live_chat_id, page_token.as_deref())
                .await
            {
                Ok(response) => {
                    // 成功: バックオフをリセット
                    if let Ok(mut backoff_lock) = backoff.lock() {
                        backoff_lock.reset();
                    }

                    // メッセージがあればイベント送信
                    if !response.items.is_empty() {
                        let messages: Vec<ChatMessage> = response
                            .items
                            .into_iter()
                            .filter_map(|item| {
                                use chrono::DateTime;

                                let published_at =
                                    match DateTime::parse_from_rfc3339(&item.snippet.published_at)
                                    {
                                        Ok(dt) => dt.with_timezone(&chrono::Utc),
                                        Err(e) => {
                                            log::warn!(
                                                "Failed to parse publishedAt for message {}: {}",
                                                item.id,
                                                e
                                            );
                                            return None;
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
                                    message_type: crate::youtube::types::MessageType::Text,
                                })
                            })
                            .collect();

                        if !messages.is_empty() {
                            event_callback(PollingEvent::Messages { messages });
                        }
                    }

                    // 状態を更新
                    {
                        if let Ok(mut state_lock) = state.lock() {
                            if let Some(s) = state_lock.as_mut() {
                                s.update(
                                    response.next_page_token,
                                    response.polling_interval_millis,
                                );

                                // 定期的に状態更新イベントを送信（10回に1回）
                                if s.poll_count % 10 == 0 {
                                    event_callback(PollingEvent::StateUpdate {
                                        quota_used: s.quota_used,
                                        remaining_quota: s.estimated_remaining_quota(),
                                        poll_count: s.poll_count,
                                    });
                                }
                            }
                        } else {
                            log::error!("Failed to acquire state lock for update");
                        }
                    }

                    // ポーリング間隔を順守
                    sleep(polling_interval).await;
                }
                Err(e) => {
                    log::error!("Polling error: {}", e);

                    match e {
                        YouTubeError::QuotaExceeded => {
                            // クォータ超過: 停止
                            event_callback(PollingEvent::QuotaExceeded);
                            event_callback(PollingEvent::Stopped {
                                reason: "クォータ超過 - 翌日まで待機してください".to_string(),
                            });
                            is_running.store(false, Ordering::SeqCst);
                            break;
                        }
                        YouTubeError::LiveChatNotFound | YouTubeError::LiveChatDisabled => {
                            // 配信終了またはチャット無効: 停止
                            let reason = match e {
                                YouTubeError::LiveChatDisabled => {
                                    "ライブチャットが無効になっています".to_string()
                                }
                                _ => "配信が終了しました".to_string(),
                            };

                            event_callback(PollingEvent::StreamEnded);
                            event_callback(PollingEvent::Stopped { reason });
                            is_running.store(false, Ordering::SeqCst);
                            break;
                        }
                        YouTubeError::InvalidPageToken => {
                            // ページトークン無効: リセットして続行
                            log::warn!("Invalid page token detected, resetting pagination");

                            {
                                if let Ok(mut state_lock) = state.lock() {
                                    if let Some(s) = state_lock.as_mut() {
                                        s.reset_page_token();
                                    }
                                } else {
                                    log::error!("Failed to acquire state lock for page token reset");
                                }
                            }

                            event_callback(PollingEvent::Error {
                                message: "ページトークンが無効です。最初から取得し直します".to_string(),
                                retrying: true,
                            });

                            // 短い待機後に続行
                            sleep(std::time::Duration::from_secs(2)).await;
                        }
                        YouTubeError::RateLimitExceeded => {
                            // レート制限: 指数バックオフで再試行
                            let (delay, should_continue) = {
                                match backoff.lock() {
                                    Ok(mut backoff_lock) => {
                                        let delay = backoff_lock.next_delay();
                                        let should_continue = backoff_lock.should_retry();
                                        (delay, should_continue)
                                    }
                                    Err(e) => {
                                        log::error!("Failed to acquire backoff lock: {}", e);
                                        (std::time::Duration::from_secs(5), false)
                                    }
                                }
                            };

                            if !should_continue {
                                log::error!("Max retry attempts exceeded for rate limit");
                                event_callback(PollingEvent::Error {
                                    message: "最大リトライ回数に達しました".to_string(),
                                    retrying: false,
                                });
                                event_callback(PollingEvent::Stopped {
                                    reason: "レート制限のリトライ上限に達しました".to_string(),
                                });
                                is_running.store(false, Ordering::SeqCst);
                                break;
                            }

                            log::warn!("Rate limit exceeded, retrying in {:?}", delay);

                            event_callback(PollingEvent::Error {
                                message: format!(
                                    "レート制限に達しました。{}秒後に再試行します",
                                    delay.as_secs()
                                ),
                                retrying: true,
                            });

                            sleep(delay).await;
                        }
                        YouTubeError::InvalidApiKey => {
                            // APIキー無効: 停止
                            event_callback(PollingEvent::Error {
                                message: "APIキーが無効です".to_string(),
                                retrying: false,
                            });
                            event_callback(PollingEvent::Stopped {
                                reason: "APIキーが無効です".to_string(),
                            });
                            is_running.store(false, Ordering::SeqCst);
                            break;
                        }
                        _ => {
                            // その他のエラー: 指数バックオフで再試行
                            let (delay, should_continue) = {
                                match backoff.lock() {
                                    Ok(mut backoff_lock) => {
                                        let delay = backoff_lock.next_delay();
                                        let should_continue = backoff_lock.should_retry();
                                        (delay, should_continue)
                                    }
                                    Err(e) => {
                                        log::error!("Failed to acquire backoff lock: {}", e);
                                        (std::time::Duration::from_secs(5), false)
                                    }
                                }
                            };

                            if !should_continue {
                                log::error!("Max retry attempts exceeded for error: {}", e);
                                event_callback(PollingEvent::Error {
                                    message: format!(
                                        "最大リトライ回数に達しました。エラー: {}",
                                        e
                                    ),
                                    retrying: false,
                                });
                                event_callback(PollingEvent::Stopped {
                                    reason: "リトライ上限に達しました".to_string(),
                                });
                                is_running.store(false, Ordering::SeqCst);
                                break;
                            }

                            log::warn!("Polling error, retrying in {:?}: {}", delay, e);

                            event_callback(PollingEvent::Error {
                                message: format!("エラーが発生しました: {}", e),
                                retrying: true,
                            });

                            // ページトークンをリセット
                            {
                                if let Ok(mut state_lock) = state.lock() {
                                    if let Some(s) = state_lock.as_mut() {
                                        s.reset_page_token();
                                    }
                                } else {
                                    log::error!("Failed to acquire state lock for page token reset");
                                }
                            }

                            sleep(delay).await;
                        }
                    }
                }
            }
        }

        log::info!("Polling loop ended");
    }
}

impl Clone for ChatPoller {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            state: Arc::clone(&self.state),
            is_running: Arc::clone(&self.is_running),
            backoff: Arc::clone(&self.backoff),
        }
    }
}
