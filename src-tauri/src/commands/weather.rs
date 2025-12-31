// =============================================================================
// 天気API Tauriコマンド
// =============================================================================
// フロントエンドから天気情報を取得・設定するためのコマンド
// Open-Meteo APIを使用（APIキー不要）
// =============================================================================

use tauri::State;

use crate::server::types::{WeatherUpdatePayload, WsMessage};
use crate::weather::WeatherData;
use crate::AppState;

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

    let payload = WeatherUpdatePayload {
        icon: weather_data.icon,
        temp: weather_data.temp,
        description: weather_data.description,
        location: weather_data.location,
        humidity: Some(weather_data.humidity),
    };

    let ws_state = state.server.read().await;
    ws_state
        .broadcast(WsMessage::WeatherUpdate { payload })
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

    let payload = WeatherUpdatePayload {
        icon: weather_data.icon,
        temp: weather_data.temp,
        description: weather_data.description,
        location: weather_data.location,
        humidity: Some(weather_data.humidity),
    };

    let ws_state = state.server.read().await;
    ws_state
        .broadcast(WsMessage::WeatherUpdate { payload })
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
    let payload = WeatherUpdatePayload {
        icon: weather_data.icon.clone(),
        temp: weather_data.temp,
        description: weather_data.description.clone(),
        location: weather_data.location.clone(),
        humidity: Some(weather_data.humidity),
    };

    let ws_state = state.server.read().await;
    ws_state
        .broadcast(WsMessage::WeatherUpdate { payload })
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
