// =============================================================================
// 天気API Tauriコマンド
// =============================================================================
// フロントエンドから天気情報を取得・設定するためのコマンド
// Open-Meteo APIを使用（APIキー不要）
// =============================================================================

use tauri::State;

use crate::server::types::{
    CityWeatherData, WeatherMultiUpdatePayload, WeatherUpdatePayload, WsMessage,
};
use crate::weather::WeatherData;
use crate::AppState;

/// ローテーション間隔の最小値（秒）
/// UIで最小3秒を設定しているが、0が渡された場合の防御的ガード
const MIN_ROTATION_INTERVAL_SEC: u32 = 1;

/// 都市名を設定
#[tauri::command(rename_all = "snake_case")]
pub async fn set_weather_city(state: State<'_, AppState>, city: String) -> Result<(), String> {
    state.weather.set_city(city).await;
    Ok(())
}

/// 現在の都市名を取得
#[tauri::command]
pub async fn get_weather_city(state: State<'_, AppState>) -> Result<String, String> {
    Ok(state.weather.get_city().await)
}

/// 天気情報を取得（キャッシュ優先）
#[tauri::command]
pub async fn get_weather(state: State<'_, AppState>) -> Result<WeatherData, String> {
    state.weather.get_weather().await.map_err(|e| e.to_string())
}

/// 天気情報を強制取得（キャッシュ無視）
#[tauri::command]
pub async fn fetch_weather(state: State<'_, AppState>) -> Result<WeatherData, String> {
    // キャッシュをクリアしてから取得
    state.weather.clear_cache().await;
    state.weather.get_weather().await.map_err(|e| e.to_string())
}

/// 天気情報をWebSocketでブロードキャスト
///
/// # Arguments
/// * `force_refresh` - trueの場合、キャッシュを無視して最新データを取得してからブロードキャスト
#[tauri::command(rename_all = "snake_case")]
pub async fn broadcast_weather_update(
    state: State<'_, AppState>,
    force_refresh: Option<bool>,
) -> Result<(), String> {
    let weather_data = if force_refresh.unwrap_or(false) {
        // 強制リフレッシュ: キャッシュをクリアしてから取得
        state.weather.clear_cache().await;
        log::info!("Force refresh requested for weather broadcast");
        state.weather.get_weather().await.map_err(|e| e.to_string())?
    } else {
        // 通常: キャッシュ優先
        state.weather.get_weather().await.map_err(|e| e.to_string())?
    };

    let ws_state = state.server.read().await;
    ws_state
        .broadcast(WsMessage::WeatherUpdate {
            payload: WeatherUpdatePayload::from(&weather_data),
        })
        .await;

    log::info!(
        "Weather update broadcasted (force_refresh: {})",
        force_refresh.unwrap_or(false)
    );
    Ok(())
}

/// 天気キャッシュをクリア
#[tauri::command]
pub async fn clear_weather_cache(state: State<'_, AppState>) -> Result<(), String> {
    state.weather.clear_cache().await;
    Ok(())
}

/// 天気キャッシュの残りTTLを取得（秒）
#[tauri::command]
pub async fn get_weather_cache_ttl(state: State<'_, AppState>) -> Result<u64, String> {
    Ok(state.weather.cache_ttl_remaining().await)
}

// =============================================================================
// 新UI用コマンド（2ボタン化対応）
// =============================================================================

/// 天気を手動更新（キャッシュクリア + 取得 + タイマーリセット）
///
/// UIの「更新」ボタン用。最新の天気データを取得し、自動更新タイマーをリセットする。
#[tauri::command]
pub async fn refresh_weather(state: State<'_, AppState>) -> Result<WeatherData, String> {
    state.weather.clear_cache().await;
    let data = state.weather.get_weather().await.map_err(|e| e.to_string())?;
    state.weather_updater.reset_timer();
    log::info!("Weather manually refreshed: {}°C, timer reset", data.temp);
    Ok(data)
}

/// 天気をオーバーレイに配信
///
/// UIの「配信」ボタン用。現在の天気データ（キャッシュ優先）をWebSocketでブロードキャストする。
#[tauri::command]
pub async fn broadcast_weather(state: State<'_, AppState>) -> Result<(), String> {
    let weather_data = state.weather.get_weather().await.map_err(|e| e.to_string())?;

    let ws_state = state.server.read().await;
    ws_state
        .broadcast(WsMessage::WeatherUpdate {
            payload: WeatherUpdatePayload::from(&weather_data),
        })
        .await;

    log::info!("Weather broadcasted to overlay: {}°C", weather_data.temp);
    Ok(())
}

/// 都市名設定 + 更新 + 配信（一括処理）
///
/// 都市名変更時に使用。都市名を保存し、最新の天気を取得してオーバーレイにも配信する。
#[tauri::command(rename_all = "snake_case")]
pub async fn set_weather_city_and_broadcast(
    state: State<'_, AppState>,
    city: String,
) -> Result<WeatherData, String> {
    // 都市名を設定（キャッシュは自動クリアされる）
    state.weather.set_city(city.clone()).await;

    // 最新の天気を取得
    let weather_data = state.weather.get_weather().await.map_err(|e| e.to_string())?;

    // WebSocketでブロードキャスト
    let ws_state = state.server.read().await;
    ws_state
        .broadcast(WsMessage::WeatherUpdate {
            payload: WeatherUpdatePayload::from(&weather_data),
        })
        .await;

    // タイマーリセット
    state.weather_updater.reset_timer();

    log::info!(
        "Weather city set to '{}', fetched and broadcasted: {}°C",
        city,
        weather_data.temp
    );
    Ok(weather_data)
}

// =============================================================================
// マルチシティモード用コマンド
// =============================================================================

/// 複数都市のマルチシティ天気データを取得
///
/// # Arguments
/// * `cities` - 都市リスト [(id, name, displayName), ...]
///
/// # Returns
/// 各都市の天気データ（失敗した都市は含まれない）
#[tauri::command(rename_all = "snake_case")]
pub async fn get_weather_multi(
    state: State<'_, AppState>,
    cities: Vec<(String, String, String)>, // (id, name, displayName)
) -> Result<Vec<CityWeatherData>, String> {
    let city_pairs: Vec<(String, String)> = cities
        .iter()
        .map(|(id, name, _)| (id.clone(), name.clone()))
        .collect();

    let results = state.weather.get_weather_multi(&city_pairs).await;

    // 成功した都市のみ返す（displayNameをマップから取得）
    let display_name_map: std::collections::HashMap<String, String> = cities
        .iter()
        .map(|(id, _, display_name)| (id.clone(), display_name.clone()))
        .collect();

    let total_cities = results.len();
    let mut weather_data: Vec<CityWeatherData> = Vec::new();
    let mut failed_cities: Vec<String> = Vec::new();

    for (id, name, result) in results {
        match result {
            Ok(data) => {
                let display_name = display_name_map
                    .get(&id)
                    .cloned()
                    .unwrap_or(data.location.clone());
                weather_data.push(CityWeatherData {
                    city_id: id,
                    city_name: display_name,
                    icon: data.icon,
                    temp: data.temp,
                    description: data.description,
                    location: data.location,
                    humidity: Some(data.humidity),
                });
            }
            Err(e) => {
                log::warn!("都市 '{}' の天気取得に失敗: {}", name, e);
                failed_cities.push(name);
            }
        }
    }

    // 部分的失敗の場合は警告ログを出力
    if !failed_cities.is_empty() {
        log::warn!(
            "マルチシティ天気: {}/{} 都市成功、失敗した都市: {:?}",
            weather_data.len(),
            total_cities,
            failed_cities
        );
    } else {
        log::info!(
            "マルチシティ天気: すべての都市({})の取得に成功",
            weather_data.len()
        );
    }

    Ok(weather_data)
}

/// 複数都市の天気をWebSocketでブロードキャスト
///
/// # Arguments
/// * `cities` - 都市リスト [(id, name, displayName), ...]
/// * `rotation_interval_sec` - ローテーション間隔（秒）
#[tauri::command(rename_all = "snake_case")]
pub async fn broadcast_weather_multi(
    state: State<'_, AppState>,
    cities: Vec<(String, String, String)>, // (id, name, displayName)
    rotation_interval_sec: u32,
) -> Result<(), String> {
    // 最小値ガード: 0が渡された場合は1秒に
    let rotation_interval_sec = rotation_interval_sec.max(MIN_ROTATION_INTERVAL_SEC);

    // 天気データを取得
    let weather_data = get_weather_multi(state.clone(), cities).await?;

    if weather_data.is_empty() {
        return Err("すべての都市の天気取得に失敗しました".to_string());
    }

    // WebSocketでブロードキャスト
    let ws_state = state.server.read().await;
    ws_state
        .broadcast(WsMessage::WeatherMultiUpdate {
            payload: WeatherMultiUpdatePayload {
                cities: weather_data,
                rotation_interval_sec,
            },
        })
        .await;

    log::info!(
        "Multi-city weather broadcasted (interval: {}s)",
        rotation_interval_sec
    );
    Ok(())
}

/// マルチシティ設定を自動更新に反映する
///
/// 設定保存時や手動配信時に呼び出し、以降の自動更新でこの設定が使用される。
///
/// # Arguments
/// * `enabled` - マルチシティモード有効/無効
/// * `cities` - 都市リスト [(id, name, displayName), ...]
/// * `rotation_interval_sec` - ローテーション間隔（秒）
#[tauri::command(rename_all = "snake_case")]
pub async fn set_multi_city_mode(
    state: State<'_, AppState>,
    enabled: bool,
    cities: Vec<(String, String, String)>,
    rotation_interval_sec: u32,
) -> Result<(), String> {
    // 最小値ガード: 0が渡された場合は1秒に
    let rotation_interval_sec = rotation_interval_sec.max(MIN_ROTATION_INTERVAL_SEC);

    let cities_len = cities.len();
    state
        .weather_updater
        .set_multi_city_config(enabled, cities, rotation_interval_sec);

    log::info!(
        "Multi-city mode set: enabled={}, cities={}, interval={}s",
        enabled,
        cities_len,
        rotation_interval_sec
    );
    Ok(())
}
