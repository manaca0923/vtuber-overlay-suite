//! 統合ポーラー
//!
//! 3つのAPIモード（InnerTube / Official / gRPC）を統一管理する。
//! モードに応じて適切なポーラーを起動し、メッセージをブロードキャストする。

use super::api_key_manager::get_api_key_manager;
use super::errors::YouTubeError;
use super::grpc::GrpcPoller;
use super::innertube::InnerTubeClient;
use super::poller::{ChatPoller, PollingEvent};
use super::types::ChatMessage;
use crate::commands::youtube::ApiMode;
use std::collections::{HashSet, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::async_runtime::JoinHandle;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;

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
    ) -> Result<(), YouTubeError> {
        self.stop().await;

        *self.mode.lock().await = Some(ApiMode::InnerTube);
        self.running.store(true, Ordering::SeqCst);

        let running = Arc::clone(&self.running);

        let handle = tauri::async_runtime::spawn(async move {
            if let Err(e) = run_innertube_loop(video_id, running.clone(), app_handle).await {
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
    ) -> Result<(), YouTubeError> {
        self.stop().await;

        *self.mode.lock().await = Some(ApiMode::Official);
        self.running.store(true, Ordering::SeqCst);

        let poller = ChatPoller::new(api_key);
        let handle = app_handle.clone();

        poller
            .start(live_chat_id, move |event: PollingEvent| {
                if let PollingEvent::Messages { messages } = event {
                    let _ = handle.emit("chat-messages", &messages);
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
    ) -> Result<(), YouTubeError> {
        self.stop().await;

        *self.mode.lock().await = Some(ApiMode::Grpc);
        self.running.store(true, Ordering::SeqCst);

        let mut poller = GrpcPoller::new();
        poller.start(live_chat_id, api_key, app_handle).await?;

        *self.grpc_poller.lock().await = Some(poller);

        log::info!("Started gRPC streaming");
        Ok(())
    }

    /// モードに応じてポーリングを開始（統一インターフェース）
    pub async fn start(
        &self,
        video_id: String,
        mode: ApiMode,
        use_bundled_key: bool,
        user_api_key: Option<String>,
        app_handle: AppHandle,
    ) -> Result<(), YouTubeError> {
        // APIキーを取得（Official/Grpcモードの場合）
        let api_key = if mode != ApiMode::InnerTube {
            // BYOKが指定されている場合は設定
            if let Some(key) = &user_api_key {
                if let Ok(mut guard) = get_api_key_manager().write() {
                    guard.set_user_key(Some(key.clone()));
                }
            }

            // アクティブなキーを取得
            let key = get_api_key_manager()
                .read()
                .ok()
                .and_then(|guard| guard.get_active_key(use_bundled_key).map(|s| s.to_string()));

            match key {
                Some(k) if !k.is_empty() => k,
                _ => return Err(YouTubeError::InvalidApiKey),
            }
        } else {
            String::new()
        };

        match mode {
            ApiMode::InnerTube => {
                self.start_innertube(video_id, app_handle).await
            }
            ApiMode::Official => {
                // video_idからlive_chat_idを取得する必要がある
                // 既存のロジックを使用
                let client = super::client::YouTubeClient::new(api_key.clone());
                let live_chat_id = client.get_live_chat_id(&video_id).await?;
                self.start_official(live_chat_id, api_key, app_handle).await
            }
            ApiMode::Grpc => {
                // video_idからlive_chat_idを取得
                let client = super::client::YouTubeClient::new(api_key.clone());
                let live_chat_id = client.get_live_chat_id(&video_id).await?;
                self.start_grpc(live_chat_id, api_key, app_handle).await
            }
        }
    }
}

impl Default for UnifiedPoller {
    fn default() -> Self {
        Self::new()
    }
}

/// InnerTubeポーリングループ
async fn run_innertube_loop(
    video_id: String,
    running: Arc<AtomicBool>,
    app_handle: AppHandle,
) -> Result<(), YouTubeError> {
    use super::innertube::parse_chat_response;

    let mut client = InnerTubeClient::new(video_id)?;
    client.initialize().await?;

    let mut seen_ids: HashSet<String> = HashSet::new();
    let mut seen_order: VecDeque<String> = VecDeque::new();

    log::info!("InnerTube polling loop started");

    while running.load(Ordering::SeqCst) {
        match client.get_chat_messages().await {
            Ok(response) => {
                let messages = parse_chat_response(response);

                // 重複排除
                let new_messages: Vec<ChatMessage> = messages
                    .into_iter()
                    .filter(|msg| {
                        if seen_ids.contains(&msg.id) {
                            false
                        } else {
                            if seen_ids.insert(msg.id.clone()) {
                                seen_order.push_back(msg.id.clone());
                            }
                            true
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
                    let _ = app_handle.emit("chat-messages", &new_messages);
                    log::debug!("InnerTube: {} new messages", new_messages.len());
                }

                // 次のポーリングまで待機
                let timeout_ms = client.get_timeout_ms();
                tokio::time::sleep(std::time::Duration::from_millis(timeout_ms)).await;
            }
            Err(e) => {
                log::error!("InnerTube fetch error: {:?}", e);
                // エラー時は少し待ってから再試行
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        }
    }

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
