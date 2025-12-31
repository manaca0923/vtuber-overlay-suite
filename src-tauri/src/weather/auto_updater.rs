// =============================================================================
// 天気自動更新モジュール
// =============================================================================
// 15分ごとに天気情報を自動取得してWebSocketでブロードキャストする
// =============================================================================

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Notify;

use crate::server::types::{ServerState, WsMessage};

use super::WeatherClient;

/// 自動更新間隔（15分 = 900秒）
const AUTO_UPDATE_INTERVAL_SECS: u64 = 900;

/// 天気自動更新タスク
///
/// アプリ起動時に開始し、15分ごとに天気を取得してWebSocketでブロードキャストする。
/// 手動更新時は `reset_timer()` でタイマーをリセットできる。
pub struct WeatherAutoUpdater {
    /// 実行中フラグ
    is_running: Arc<AtomicBool>,
    /// タイマーリセット通知
    reset_signal: Arc<Notify>,
}

impl WeatherAutoUpdater {
    /// 自動更新タスクを開始する
    ///
    /// # Arguments
    /// * `weather` - 天気クライアント
    /// * `server` - WebSocketサーバー状態
    ///
    /// # Returns
    /// WeatherAutoUpdaterインスタンス（stop/reset_timer用）
    pub fn start(weather: Arc<WeatherClient>, server: ServerState) -> Self {
        let is_running = Arc::new(AtomicBool::new(true));
        let reset_signal = Arc::new(Notify::new());

        let is_running_clone = Arc::clone(&is_running);
        let reset_signal_clone = Arc::clone(&reset_signal);

        tauri::async_runtime::spawn(async move {
            Self::update_loop(weather, server, is_running_clone, reset_signal_clone).await;
        });

        log::info!(
            "Weather auto-updater started (interval: {}s)",
            AUTO_UPDATE_INTERVAL_SECS
        );

        Self {
            is_running,
            reset_signal,
        }
    }

    /// 自動更新ループ
    async fn update_loop(
        weather: Arc<WeatherClient>,
        server: ServerState,
        is_running: Arc<AtomicBool>,
        reset_signal: Arc<Notify>,
    ) {
        while is_running.load(Ordering::SeqCst) {
            // 15分待機（reset_signalで中断可能）
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(AUTO_UPDATE_INTERVAL_SECS)) => {
                    // 15分経過 → 天気を更新
                }
                _ = reset_signal.notified() => {
                    // タイマーリセット → ループ先頭に戻る（待機時間リセット）
                    log::debug!("Weather auto-update timer reset");
                    continue;
                }
            }

            // 天気を取得してブロードキャスト
            if let Err(e) = Self::fetch_and_broadcast(&weather, &server).await {
                log::warn!("Weather auto-update failed: {}", e);
            }
        }

        log::info!("Weather auto-updater stopped");
    }

    /// 天気を取得してWebSocketでブロードキャスト
    async fn fetch_and_broadcast(
        weather: &WeatherClient,
        server: &ServerState,
    ) -> Result<(), String> {
        // キャッシュをクリアして最新データを取得
        weather.clear_cache().await;
        let data = weather.get_weather().await.map_err(|e| e.to_string())?;

        let ws_state = server.read().await;
        ws_state
            .broadcast(WsMessage::WeatherUpdate {
                payload: (&data).into(),
            })
            .await;

        log::info!("Weather auto-update broadcasted: {}°C", data.temp);
        Ok(())
    }

    /// 自動更新を停止する
    pub fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);
        // ループをすぐに終了させるためにリセットシグナルを送る
        self.reset_signal.notify_one();
    }

    /// タイマーをリセットする（手動更新時に呼び出し）
    ///
    /// 次回の自動更新までの時間を15分にリセットする。
    pub fn reset_timer(&self) {
        self.reset_signal.notify_one();
    }

    /// 実行中かどうかを確認
    ///
    /// NOTE: 現在未使用だが、将来の状態確認UI等で使用予定（PR#107: 天気自動更新機能）
    #[allow(dead_code)]
    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }
}

impl Drop for WeatherAutoUpdater {
    fn drop(&mut self) {
        self.stop();
    }
}
