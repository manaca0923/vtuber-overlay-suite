// =============================================================================
// 天気自動更新モジュール
// =============================================================================
// 15分ごとに天気情報を自動取得してWebSocketでブロードキャストする
// マルチシティモードにも対応
// =============================================================================

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Notify, RwLock};

use crate::server::types::{CityWeatherData, ServerState, WeatherMultiUpdatePayload, WsMessage};

use super::WeatherClient;

/// 自動更新間隔（15分 = 900秒）
const AUTO_UPDATE_INTERVAL_SECS: u64 = 900;

/// マルチシティ設定
#[derive(Debug, Clone)]
pub struct MultiCityConfig {
    /// マルチシティモード有効
    pub enabled: bool,
    /// 都市リスト: (id, name, displayName)
    pub cities: Vec<(String, String, String)>,
    /// ローテーション間隔（秒）
    pub rotation_interval_sec: u32,
}

/// 天気自動更新タスク
///
/// アプリ起動時に開始し、15分ごとに天気を取得してWebSocketでブロードキャストする。
/// 手動更新時は `reset_timer()` でタイマーをリセットできる。
/// マルチシティモードにも対応。
pub struct WeatherAutoUpdater {
    /// 実行中フラグ
    is_running: Arc<AtomicBool>,
    /// タイマーリセット通知
    reset_signal: Arc<Notify>,
    /// マルチシティ設定
    multi_city_config: Arc<RwLock<MultiCityConfig>>,
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
        let multi_city_config = Arc::new(RwLock::new(MultiCityConfig {
            enabled: false,
            cities: Vec::new(),
            rotation_interval_sec: 5,
        }));

        let is_running_clone = Arc::clone(&is_running);
        let reset_signal_clone = Arc::clone(&reset_signal);
        let multi_city_config_clone = Arc::clone(&multi_city_config);

        tauri::async_runtime::spawn(async move {
            Self::update_loop(
                weather,
                server,
                is_running_clone,
                reset_signal_clone,
                multi_city_config_clone,
            )
            .await;
        });

        log::info!(
            "Weather auto-updater started (interval: {}s)",
            AUTO_UPDATE_INTERVAL_SECS
        );

        Self {
            is_running,
            reset_signal,
            multi_city_config,
        }
    }

    /// 自動更新ループ
    async fn update_loop(
        weather: Arc<WeatherClient>,
        server: ServerState,
        is_running: Arc<AtomicBool>,
        reset_signal: Arc<Notify>,
        multi_city_config: Arc<RwLock<MultiCityConfig>>,
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

            // 天気を取得してブロードキャスト（モードに応じて）
            let config = multi_city_config.read().await.clone();
            if config.enabled && !config.cities.is_empty() {
                // マルチシティモード
                if let Err(e) =
                    Self::fetch_and_broadcast_multi(&weather, &server, &config).await
                {
                    log::warn!("Weather multi-city auto-update failed: {}", e);
                }
            } else {
                // 単一都市モード
                if let Err(e) = Self::fetch_and_broadcast_single(&weather, &server).await {
                    log::warn!("Weather auto-update failed: {}", e);
                }
            }
        }

        log::info!("Weather auto-updater stopped");
    }

    /// 単一都市モード: 天気を取得してWebSocketでブロードキャスト
    ///
    /// ## 設計ノート
    /// - Fire-and-forgetパターン: ブロードキャストは`tokio::spawn`でバックグラウンド実行
    /// - RwLockガードをawait境界をまたいで保持しないようにtokio::spawnで分離
    async fn fetch_and_broadcast_single(
        weather: &WeatherClient,
        server: &ServerState,
    ) -> Result<(), String> {
        // キャッシュをクリアして最新データを取得
        weather.clear_cache().await;
        let data = weather.get_weather().await.map_err(|e| e.to_string())?;
        let temp = data.temp;

        // WebSocketでブロードキャスト（Fire-and-forget）
        let server = Arc::clone(server);
        let message = WsMessage::WeatherUpdate {
            payload: (&data).into(),
        };
        tokio::spawn(async move {
            let peers_arc = {
                let ws_state = server.read().await;
                ws_state.get_peers_arc()
            };
            let peers_guard = peers_arc.read().await;
            let peers: Vec<_> = peers_guard
                .iter()
                .map(|(id, tx)| (*id, tx.clone()))
                .collect();
            drop(peers_guard);
            crate::server::websocket::WebSocketState::send_to_peers(&peers, &message);
            log::debug!("Weather auto-update broadcasted: {}°C", temp);
        });

        Ok(())
    }

    /// マルチシティモード: 複数都市の天気を取得してWebSocketでブロードキャスト
    ///
    /// ## 設計ノート
    /// - Fire-and-forgetパターン: ブロードキャストは`tokio::spawn`でバックグラウンド実行
    /// - RwLockガードをawait境界をまたいで保持しないようにtokio::spawnで分離
    async fn fetch_and_broadcast_multi(
        weather: &WeatherClient,
        server: &ServerState,
        config: &MultiCityConfig,
    ) -> Result<(), String> {
        // 都市リストを準備
        let city_pairs: Vec<(String, String)> = config
            .cities
            .iter()
            .map(|(id, name, _)| (id.clone(), name.clone()))
            .collect();

        // 複数都市の天気を取得
        let results = weather.get_weather_multi(&city_pairs).await;

        // displayNameマップを作成
        let display_name_map: std::collections::HashMap<String, String> = config
            .cities
            .iter()
            .map(|(id, _, display_name)| (id.clone(), display_name.clone()))
            .collect();

        // 成功した都市のみ抽出
        let weather_data: Vec<CityWeatherData> = results
            .into_iter()
            .filter_map(|(id, _name, result)| {
                result.ok().map(|data| {
                    let display_name = display_name_map
                        .get(&id)
                        .cloned()
                        .unwrap_or(data.location.clone());
                    CityWeatherData {
                        city_id: id,
                        city_name: display_name,
                        icon: data.icon,
                        temp: data.temp,
                        description: data.description,
                        location: data.location,
                        humidity: Some(data.humidity),
                    }
                })
            })
            .collect();

        if weather_data.is_empty() {
            return Err("No weather data available for any city".to_string());
        }

        // WebSocketでブロードキャスト（Fire-and-forget）
        let server = Arc::clone(server);
        let city_count = weather_data.len();
        let rotation_interval = config.rotation_interval_sec;
        let message = WsMessage::WeatherMultiUpdate {
            payload: WeatherMultiUpdatePayload {
                cities: weather_data,
                rotation_interval_sec: rotation_interval,
            },
        };
        tokio::spawn(async move {
            let peers_arc = {
                let ws_state = server.read().await;
                ws_state.get_peers_arc()
            };
            let peers_guard = peers_arc.read().await;
            let peers: Vec<_> = peers_guard
                .iter()
                .map(|(id, tx)| (*id, tx.clone()))
                .collect();
            drop(peers_guard);
            crate::server::websocket::WebSocketState::send_to_peers(&peers, &message);
            log::debug!(
                "Weather multi-city auto-update broadcasted: {} cities (interval: {}s)",
                city_count,
                rotation_interval
            );
        });

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

    /// マルチシティ設定を更新する
    ///
    /// 設定保存時や手動配信時に呼び出し、次回の自動更新からこの設定が使用される。
    pub fn set_multi_city_config(
        &self,
        enabled: bool,
        cities: Vec<(String, String, String)>,
        rotation_interval_sec: u32,
    ) {
        // blockingでロックを取得（非async関数から呼び出せるように）
        let config = MultiCityConfig {
            enabled,
            cities,
            rotation_interval_sec,
        };

        // tokio runtime内で実行
        let multi_city_config = Arc::clone(&self.multi_city_config);
        tauri::async_runtime::spawn(async move {
            let mut guard = multi_city_config.write().await;
            *guard = config;
            log::info!(
                "Multi-city config updated: enabled={}, cities={}",
                guard.enabled,
                guard.cities.len()
            );
        });
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
