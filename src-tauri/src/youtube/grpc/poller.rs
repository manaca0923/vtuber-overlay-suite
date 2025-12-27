//! gRPC Streaming Poller
//!
//! Manages gRPC streaming connection lifecycle and broadcasts messages via WebSocket.
//!
//! ## WS/DB連携
//! 取得したコメントは以下の経路で配信される：
//! 1. Tauriイベント（chat-messages）→ フロントエンドUI
//! 2. WebSocketブロードキャスト（comment:add）→ OBSオーバーレイ
//! 3. SQLite保存 → コメントログ

use super::client::GrpcChatClient;
use crate::server::types::WsMessage;
use crate::server::WebSocketState;
use crate::youtube::api_key_manager::get_api_key_manager;
use crate::youtube::backoff::ExponentialBackoff;
use crate::youtube::db::save_comments_to_db;
use crate::youtube::errors::YouTubeError;
use sqlx::SqlitePool;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::async_runtime::JoinHandle;
use tauri::{AppHandle, Emitter};
use tokio::sync::RwLock;
use tokio_stream::StreamExt;

/// gRPC polling state
pub struct GrpcPoller {
    /// Handle to the streaming task
    task_handle: Option<JoinHandle<()>>,
    /// Stop signal
    stop_signal: Arc<AtomicBool>,
}

impl GrpcPoller {
    /// Create a new gRPC poller
    pub fn new() -> Self {
        Self {
            task_handle: None,
            stop_signal: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Start polling with gRPC streaming
    pub async fn start(
        &mut self,
        live_chat_id: String,
        api_key: String,
        app_handle: AppHandle,
        db_pool: SqlitePool,
        server_state: Arc<RwLock<WebSocketState>>,
    ) -> Result<(), YouTubeError> {
        // Stop any existing polling
        self.stop().await;

        // Reset stop signal
        self.stop_signal.store(false, Ordering::SeqCst);

        let stop_signal = self.stop_signal.clone();

        // Spawn streaming task
        let handle = tauri::async_runtime::spawn(async move {
            if let Err(e) = run_grpc_stream(live_chat_id, api_key, stop_signal, app_handle, db_pool, server_state).await {
                log::error!("gRPC streaming error: {:?}", e);
            }
        });

        self.task_handle = Some(handle);

        log::info!("gRPC polling started");
        Ok(())
    }

    /// Stop polling
    ///
    /// この関数は冪等であり、複数回呼び出しても安全です。
    pub async fn stop(&mut self) {
        // 既に停止済みならスキップ
        if self.task_handle.is_none() {
            log::debug!("gRPC polling already stopped");
            return;
        }

        self.stop_signal.store(true, Ordering::SeqCst);

        if let Some(handle) = self.task_handle.take() {
            handle.abort();
            let _ = handle.await;
        }

        log::info!("gRPC polling stopped");
    }

    /// Check if polling is active
    pub fn is_running(&self) -> bool {
        self.task_handle.is_some() && !self.stop_signal.load(Ordering::SeqCst)
    }
}

impl Default for GrpcPoller {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for GrpcPoller {
    /// ドロップ時にタスクを確実に停止する
    fn drop(&mut self) {
        self.stop_signal.store(true, Ordering::SeqCst);
        if let Some(handle) = self.task_handle.take() {
            handle.abort();
        }
    }
}

/// Run the gRPC streaming loop
///
/// # バックオフ戦略
/// - `connection_backoff`: gRPCエンドポイントへの接続失敗時に使用
/// - `client.get_backoff_delay()`: ストリーム開始失敗・切断後の再接続時に使用
///   （クライアント内部で成功時にリセットされる）
async fn run_grpc_stream(
    live_chat_id: String,
    api_key: String,
    stop_signal: Arc<AtomicBool>,
    app_handle: AppHandle,
    db_pool: SqlitePool,
    server_state: Arc<RwLock<WebSocketState>>,
) -> Result<(), YouTubeError> {
    let mut current_api_key = api_key;
    let mut retry_with_secondary = false;
    // gRPCエンドポイントへの接続失敗時のバックオフ（ジッタ付き）
    // ストリーム開始・再接続のバックオフはclient.get_backoff_delay()を使用
    let mut connection_backoff = ExponentialBackoff::with_jitter();

    loop {
        if stop_signal.load(Ordering::SeqCst) {
            log::info!("gRPC stream stopped by signal");
            break;
        }

        // Try secondary key if primary failed with auth error
        if retry_with_secondary {
            let manager = get_api_key_manager();
            match manager.read() {
                Ok(guard) => {
                    guard.switch_to_secondary();
                    if let Some(secondary) = guard.get_active_key(true) {
                        log::info!("Switching to secondary API key");
                        current_api_key = secondary.to_string();
                        retry_with_secondary = false;
                    } else {
                        log::error!("No secondary API key available");
                        return Err(YouTubeError::InvalidApiKey);
                    }
                }
                Err(poison_error) => {
                    // RwLock is poisoned - a thread panicked while holding the lock
                    log::error!(
                        "API key manager lock is poisoned (a thread panicked): {}",
                        poison_error
                    );
                    return Err(YouTubeError::ApiError(
                        "API key manager lock poisoned".to_string(),
                    ));
                }
            }
        }

        // Connect to gRPC endpoint
        let mut client = match GrpcChatClient::connect(
            current_api_key.clone(),
            live_chat_id.clone(),
        )
        .await
        {
            Ok(c) => {
                // 接続成功時はバックオフをリセット
                connection_backoff.reset();
                c
            }
            Err(YouTubeError::InvalidApiKey) => {
                retry_with_secondary = true;
                continue;
            }
            Err(e) => {
                log::error!("Failed to connect to gRPC: {:?}", e);
                // 指数バックオフで待機
                let delay = connection_backoff.next_delay();
                log::info!("Retrying connection in {:?}", delay);
                tokio::time::sleep(delay).await;
                continue;
            }
        };

        // Start streaming
        let mut stream = match client.stream().await {
            Ok(s) => s,
            Err(YouTubeError::InvalidApiKey) => {
                retry_with_secondary = true;
                continue;
            }
            Err(e) => {
                log::error!("Failed to start gRPC stream: {:?}", e);
                let delay = client.get_backoff_delay();
                log::info!("Retrying in {:?}", delay);
                tokio::time::sleep(delay).await;
                continue;
            }
        };

        log::info!("gRPC stream connected, waiting for messages...");

        // Emit connection status
        let _ = app_handle.emit("grpc-status", serde_json::json!({
            "connected": true,
            "liveChatId": &live_chat_id
        }));

        // Track message count for debugging
        let mut message_count = 0u64;
        let mut response_count = 0u64;

        // Process stream
        loop {
            if stop_signal.load(Ordering::SeqCst) {
                log::info!("gRPC stream stopped by signal (received {} responses, {} messages)", response_count, message_count);
                break;
            }

            log::debug!("Waiting for next gRPC response...");

            match stream.next().await {
                Some(Ok(response)) => {
                    response_count += 1;
                    // Reset backoff on successful message
                    client.reset_backoff();

                    // Log response details
                    let item_count = response.items.len();
                    let has_next_page = response.next_page_token.as_ref().map_or(false, |t| !t.is_empty());
                    log::info!(
                        "gRPC response #{}: {} items, next_page_token={}",
                        response_count,
                        item_count,
                        if has_next_page { "present" } else { "empty" }
                    );

                    // Parse and broadcast messages
                    let messages = client.parse_response(response);
                    message_count += messages.len() as u64;

                    if !messages.is_empty() {
                        let broadcast_count = messages.len();

                        // Emit to frontend via Tauri event
                        let _ = app_handle.emit("chat-messages", &messages);

                        // DBに保存
                        save_comments_to_db(&db_pool, &messages).await;

                        // Broadcast to WebSocket clients (for overlays) - gRPCは即時表示
                        let state_lock = server_state.read().await;
                        for msg in &messages {
                            state_lock.broadcast(WsMessage::CommentAdd { payload: msg.clone(), instant: true }).await;
                        }

                        log::info!("Broadcast {} chat messages to WebSocket (total: {})", broadcast_count, message_count);
                    }
                }
                Some(Err(status)) => {
                    log::error!(
                        "gRPC stream error: code={}, message={}, details={:?}",
                        status.code(),
                        status.message(),
                        status.details()
                    );

                    // Check if auth error
                    if status.code() == tonic::Code::Unauthenticated {
                        retry_with_secondary = true;
                    }

                    // Emit disconnection status
                    let _ = app_handle.emit("grpc-status", serde_json::json!({
                        "connected": false,
                        "error": status.message()
                    }));

                    break;
                }
                None => {
                    // Stream ended - this could mean:
                    // 1. Server closed the stream (normal for server-streaming)
                    // 2. Network issue
                    // 3. Invalid request
                    log::warn!(
                        "gRPC stream ended after {} responses, {} messages. This may indicate the stream was closed by server.",
                        response_count,
                        message_count
                    );
                    break;
                }
            }
        }

        // Wait before reconnecting
        if !stop_signal.load(Ordering::SeqCst) {
            let delay = client.get_backoff_delay();
            log::info!("Reconnecting in {:?}", delay);
            tokio::time::sleep(delay).await;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grpc_poller_new() {
        let poller = GrpcPoller::new();
        assert!(!poller.is_running());
    }
}
