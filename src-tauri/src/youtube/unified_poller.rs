//! 統合ポーラー
//!
//! 3つのAPIモード（InnerTube / Official / gRPC）を統一管理する。
//! モードに応じて適切なポーラーを起動し、メッセージをブロードキャストする。
//!
//! ## WS/DB連携
//! 取得したコメントは以下の経路で配信される：
//! 1. Tauriイベント（chat-messages）→ フロントエンドUI
//! 2. WebSocketブロードキャスト（comment:add）→ OBSオーバーレイ
//! 3. SQLite保存 → コメントログ

use super::api_key_manager::get_api_key_manager;
use super::backoff::ExponentialBackoff;
use super::db::save_comments_to_db;
use super::errors::YouTubeError;
use super::grpc::GrpcPoller;
use super::innertube::InnerTubeClient;
use super::poller::{ChatPoller, PollingEvent};
use super::types::ChatMessage;
use crate::commands::youtube::ApiMode;
use crate::server::types::WsMessage;
use crate::server::WebSocketState;
use sqlx::SqlitePool;
use std::collections::{HashSet, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::async_runtime::JoinHandle;
use tauri::{AppHandle, Emitter};
use tokio::sync::{Mutex, RwLock};

/// 重複排除用のメッセージIDの最大保持数
const MAX_SEEN_IDS: usize = 10000;

/// 統合ポーラー
///
/// 3つのモード（InnerTube / Official / gRPC）のいずれかでポーリングを実行し、
/// 共通のイベント形式でメッセージを配信する。
pub struct UnifiedPoller {
    /// 現在のモード
    mode: Arc<Mutex<Option<ApiMode>>>,
    /// 実行中フラグ
    running: Arc<AtomicBool>,
    /// ポーリングタスクハンドル
    task_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
    /// gRPCポーラー（gRPCモード時のみ使用）
    grpc_poller: Arc<Mutex<Option<GrpcPoller>>>,
    /// 公式APIポーラー（Officialモード時のみ使用）
    official_poller: Arc<Mutex<Option<ChatPoller>>>,
}

impl UnifiedPoller {
    /// 新しい統合ポーラーを作成
    pub fn new() -> Self {
        Self {
            mode: Arc::new(Mutex::new(None)),
            running: Arc::new(AtomicBool::new(false)),
            task_handle: Arc::new(Mutex::new(None)),
            grpc_poller: Arc::new(Mutex::new(None)),
            official_poller: Arc::new(Mutex::new(None)),
        }
    }

    /// 現在のモードを取得
    pub async fn current_mode(&self) -> Option<ApiMode> {
        *self.mode.lock().await
    }

    /// ポーリング中かどうか
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// ポーリングを停止
    pub async fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);

        // gRPCポーラーを停止
        if let Some(mut poller) = self.grpc_poller.lock().await.take() {
            poller.stop().await;
        }

        // 公式APIポーラーを停止
        if let Some(poller) = self.official_poller.lock().await.take() {
            poller.stop();
        }

        // タスクハンドルをabort
        if let Some(handle) = self.task_handle.lock().await.take() {
            handle.abort();
            let _ = handle.await;
        }

        // モードをリセット
        *self.mode.lock().await = None;

        log::info!("Unified poller stopped");
    }

    /// InnerTubeモードでポーリングを開始
    pub async fn start_innertube(
        &self,
        video_id: String,
        app_handle: AppHandle,
        db_pool: SqlitePool,
        server_state: Arc<RwLock<WebSocketState>>,
    ) -> Result<(), YouTubeError> {
        self.stop().await;

        *self.mode.lock().await = Some(ApiMode::InnerTube);
        self.running.store(true, Ordering::SeqCst);

        let running = Arc::clone(&self.running);

        let handle = tauri::async_runtime::spawn(async move {
            if let Err(e) = run_innertube_loop(video_id, running.clone(), app_handle, db_pool, server_state).await {
                log::error!("InnerTube polling error: {:?}", e);
            }
            running.store(false, Ordering::SeqCst);
        });

        *self.task_handle.lock().await = Some(handle);

        log::info!("Started InnerTube polling");
        Ok(())
    }

    /// 公式APIモード（ポーリング）で開始
    pub async fn start_official(
        &self,
        live_chat_id: String,
        api_key: String,
        app_handle: AppHandle,
        db_pool: SqlitePool,
        server_state: Arc<RwLock<WebSocketState>>,
    ) -> Result<(), YouTubeError> {
        self.stop().await;

        *self.mode.lock().await = Some(ApiMode::Official);
        self.running.store(true, Ordering::SeqCst);

        let poller = ChatPoller::new(api_key);
        let handle = app_handle.clone();
        let db_pool_for_callback = db_pool.clone();
        let server_state_for_callback: Arc<RwLock<WebSocketState>> = Arc::clone(&server_state);

        poller
            .start(live_chat_id, move |event: PollingEvent| {
                let db_pool = db_pool_for_callback.clone();
                let server_state = Arc::clone(&server_state_for_callback);

                match event {
                    PollingEvent::Messages { messages } => {
                        // フロントエンドへのイベント発火
                        let _ = handle.emit("chat-messages", &messages);

                        // WS/DB連携（非同期タスクで処理）
                        let messages_clone = messages.clone();
                        tokio::spawn(async move {
                            // DBに保存
                            let save_result = save_comments_to_db(&db_pool, &messages_clone).await;
                            if save_result.failed > 0 || save_result.skipped > 0 {
                                log::warn!(
                                    "save_comments_to_db: {} saved, {} failed, {} skipped",
                                    save_result.saved, save_result.failed, save_result.skipped
                                );
                            }

                            // WebSocketでブロードキャスト（公式APIはバッファリング表示、デフォルト5秒）
                            let state_lock = server_state.read().await;
                            for msg in messages_clone {
                                state_lock.broadcast(WsMessage::CommentAdd { payload: msg, instant: false, buffer_interval_ms: None }).await;
                            }
                        });
                    }
                    PollingEvent::Started { live_chat_id } => {
                        let _ = handle.emit("official-status", serde_json::json!({
                            "connected": true,
                            "liveChatId": live_chat_id
                        }));
                    }
                    PollingEvent::Stopped { reason } => {
                        let _ = handle.emit("official-status", serde_json::json!({
                            "connected": false,
                            "stopped": true,
                            "reason": reason
                        }));
                    }
                    PollingEvent::Error { message, retrying } => {
                        let _ = handle.emit("official-status", serde_json::json!({
                            "connected": false,
                            "error": message,
                            "retrying": retrying
                        }));
                    }
                    PollingEvent::QuotaExceeded => {
                        let _ = handle.emit("official-status", serde_json::json!({
                            "connected": false,
                            "error": "クォータ超過",
                            "quotaExceeded": true
                        }));
                    }
                    PollingEvent::StreamEnded => {
                        let _ = handle.emit("official-status", serde_json::json!({
                            "connected": false,
                            "streamEnded": true
                        }));
                    }
                    PollingEvent::StateUpdate { quota_used, remaining_quota, poll_count, .. } => {
                        let _ = handle.emit("official-status", serde_json::json!({
                            "connected": true,
                            "quotaUsed": quota_used,
                            "remainingQuota": remaining_quota,
                            "pollCount": poll_count
                        }));
                    }
                }
            })
            .await?;

        *self.official_poller.lock().await = Some(poller);

        log::info!("Started Official API polling");
        Ok(())
    }

    /// gRPCモードで開始
    pub async fn start_grpc(
        &self,
        live_chat_id: String,
        api_key: String,
        app_handle: AppHandle,
        db_pool: SqlitePool,
        server_state: Arc<RwLock<WebSocketState>>,
    ) -> Result<(), YouTubeError> {
        self.stop().await;

        *self.mode.lock().await = Some(ApiMode::Grpc);
        self.running.store(true, Ordering::SeqCst);

        let mut poller = GrpcPoller::new();
        poller.start(live_chat_id, api_key, app_handle, db_pool, server_state).await?;

        *self.grpc_poller.lock().await = Some(poller);

        log::info!("Started gRPC streaming");
        Ok(())
    }

    /// モードに応じてポーリングを開始（統一インターフェース）
    ///
    /// ## 引数
    /// - `video_id`: YouTube動画ID
    /// - `mode`: APIモード（InnerTube / Official / Grpc）
    /// - `use_bundled_key`: 同梱APIキーを使用するか
    /// - `user_api_key`: ユーザー指定のAPIキー（BYOK）
    /// - `app_handle`: Tauriアプリケーションハンドル
    /// - `db_pool`: SQLiteデータベースプール（コメントログ保存用）
    /// - `server_state`: WebSocketサーバー状態（オーバーレイへのブロードキャスト用）
    pub async fn start(
        &self,
        video_id: String,
        mode: ApiMode,
        use_bundled_key: bool,
        user_api_key: Option<String>,
        app_handle: AppHandle,
        db_pool: SqlitePool,
        server_state: Arc<RwLock<WebSocketState>>,
    ) -> Result<(), YouTubeError> {
        match mode {
            ApiMode::InnerTube => {
                // InnerTubeモードはAPIキー不要
                self.start_innertube(video_id, app_handle, db_pool, server_state).await
            }
            ApiMode::Official => {
                // APIキーを取得
                let api_key = get_api_key_for_mode(use_bundled_key, user_api_key.as_ref())?;
                // video_idからlive_chat_idを取得
                let client = super::client::YouTubeClient::new(api_key.clone());
                let live_chat_id = client.get_live_chat_id(&video_id).await?;
                self.start_official(live_chat_id, api_key, app_handle, db_pool, server_state).await
            }
            ApiMode::Grpc => {
                // APIキーを取得
                let api_key = get_api_key_for_mode(use_bundled_key, user_api_key.as_ref())?;
                // video_idからlive_chat_idを取得
                let client = super::client::YouTubeClient::new(api_key.clone());
                let live_chat_id = client.get_live_chat_id(&video_id).await?;
                self.start_grpc(live_chat_id, api_key, app_handle, db_pool, server_state).await
            }
        }
    }
}

impl Default for UnifiedPoller {
    fn default() -> Self {
        Self::new()
    }
}

/// APIキーを取得する（Official/Grpcモード用）
///
/// BYOKが指定されている場合は設定し、その後アクティブなキーを返す。
fn get_api_key_for_mode(
    use_bundled_key: bool,
    user_api_key: Option<&String>,
) -> Result<String, YouTubeError> {
    // BYOKが指定されている場合は設定
    if let Some(key) = user_api_key {
        match get_api_key_manager().write() {
            Ok(mut guard) => {
                guard.set_user_key(Some(key.clone()));
            }
            Err(poison_error) => {
                log::error!(
                    "API key manager write lock is poisoned: {}",
                    poison_error
                );
                return Err(YouTubeError::ApiError(
                    "API key manager lock poisoned".to_string(),
                ));
            }
        }
    }

    // アクティブなキーを取得
    let key = match get_api_key_manager().read() {
        Ok(guard) => guard.get_active_key(use_bundled_key).map(|s| s.to_string()),
        Err(poison_error) => {
            log::error!(
                "API key manager read lock is poisoned: {}",
                poison_error
            );
            return Err(YouTubeError::ApiError(
                "API key manager lock poisoned".to_string(),
            ));
        }
    };

    match key {
        Some(k) if !k.is_empty() => Ok(k),
        _ => Err(YouTubeError::InvalidApiKey),
    }
}

/// InnerTubeポーリングループ
async fn run_innertube_loop(
    video_id: String,
    running: Arc<AtomicBool>,
    app_handle: AppHandle,
    db_pool: SqlitePool,
    server_state: Arc<RwLock<WebSocketState>>,
) -> Result<(), YouTubeError> {
    use super::innertube::parse_chat_response;

    let mut client = InnerTubeClient::new(video_id)?;
    client.initialize().await?;

    let mut seen_ids: HashSet<String> = HashSet::new();
    let mut seen_order: VecDeque<String> = VecDeque::new();
    // エラー時の指数バックオフ（ジッタ付き）
    let mut error_backoff = ExponentialBackoff::with_jitter();

    log::info!("InnerTube polling loop started");

    // 接続成功を通知
    let _ = app_handle.emit("innertube-status", serde_json::json!({
        "connected": true
    }));

    while running.load(Ordering::SeqCst) {
        match client.get_chat_messages().await {
            Ok(response) => {
                // 成功時はバックオフをリセット
                error_backoff.reset();

                let messages = parse_chat_response(response);

                // 重複排除（HashSet::insertの戻り値を利用して簡素化）
                let new_messages: Vec<ChatMessage> = messages
                    .into_iter()
                    .filter(|msg| {
                        if seen_ids.insert(msg.id.clone()) {
                            seen_order.push_back(msg.id.clone());
                            true
                        } else {
                            false
                        }
                    })
                    .collect();

                // FIFO eviction
                while seen_ids.len() > MAX_SEEN_IDS {
                    if let Some(oldest_id) = seen_order.pop_front() {
                        seen_ids.remove(&oldest_id);
                    } else {
                        break;
                    }
                }

                if !new_messages.is_empty() {
                    // フロントエンドへのイベント発火
                    let _ = app_handle.emit("chat-messages", &new_messages);
                    log::debug!("InnerTube: {} new messages", new_messages.len());

                    // WS/DB連携
                    // DBに保存
                    let save_result = save_comments_to_db(&db_pool, &new_messages).await;
                    if save_result.failed > 0 || save_result.skipped > 0 {
                        log::warn!(
                            "save_comments_to_db: {} saved, {} failed, {} skipped",
                            save_result.saved, save_result.failed, save_result.skipped
                        );
                    }

                    // WebSocketでブロードキャスト（InnerTubeはバッファリング表示、1秒間隔）
                    let state_lock = server_state.read().await;
                    for msg in &new_messages {
                        state_lock.broadcast(WsMessage::CommentAdd {
                            payload: msg.clone(),
                            instant: false,
                            buffer_interval_ms: Some(1000), // InnerTubeは1秒バッファ
                        }).await;
                    }
                }

                // 次のポーリングまで待機
                // Continuation種別に応じてポーリング間隔を制御
                let api_timeout = client.get_timeout_ms();
                let cont_type = client.get_continuation_type();
                let timeout_ms = cont_type.effective_timeout_ms(api_timeout);
                log::debug!(
                    "InnerTube: next poll in {}ms (API: {}ms, type: {:?})",
                    timeout_ms,
                    api_timeout,
                    cont_type
                );
                tokio::time::sleep(std::time::Duration::from_millis(timeout_ms)).await;
            }
            Err(e) => {
                log::error!("InnerTube fetch error: {:?}", e);

                // エラーをフロントエンドに通知
                let _ = app_handle.emit("innertube-status", serde_json::json!({
                    "connected": false,
                    "error": format!("{:?}", e)
                }));

                // 指数バックオフで待機
                let delay = error_backoff.next_delay();
                log::info!("InnerTube: retrying in {:?}", delay);
                tokio::time::sleep(delay).await;
            }
        }
    }

    // 終了を通知
    let _ = app_handle.emit("innertube-status", serde_json::json!({
        "connected": false,
        "stopped": true
    }));

    log::info!("InnerTube polling loop ended");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unified_poller_new() {
        let poller = UnifiedPoller::new();
        assert!(!poller.is_running());
    }
}
