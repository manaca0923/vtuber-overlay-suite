// =============================================================================
// 天気API Tauriコマンド
// =============================================================================
// フロントエンドから天気情報を取得・設定するためのコマンド
// =============================================================================

use tauri::State;

use crate::keyring;
use crate::server::types::{WeatherUpdatePayload, WsMessage};
use crate::weather::WeatherData;
use crate::AppState;

/// 天気APIキーを設定（keyringに保存 + メモリにセット）
#[tauri::command]
pub async fn set_weather_api_key(
    state: State<'_, AppState>,
    api_key: String,
) -> Result<(), String> {
    // keyringに保存（永続化）
    keyring::save_weather_api_key(&api_key).map_err(|e| e.to_string())?;

    // メモリにもセット（即時利用可能に）
    state.weather.set_api_key(api_key).await;
    Ok(())
}

/// 天気APIキーが設定されているか確認（keyringから確認 + メモリと同期）
///
/// keyringにキーがある場合はメモリにもロードして状態を同期する。
/// これにより、起動時のkeyring復元が失敗した場合でも、
/// UIが「設定済み」を表示するタイミングでメモリにもキーがロードされる。
#[tauri::command]
pub async fn has_weather_api_key(state: State<'_, AppState>) -> Result<bool, String> {
    // keyringから確認
    match keyring::get_weather_api_key() {
        Ok(api_key) => {
            // keyringにキーがある場合、メモリにもセット（まだ無い場合）
            if !state.weather.has_api_key().await {
                state.weather.set_api_key(api_key).await;
                log::info!("Weather API key synced from keyring to memory");
            }
            Ok(true)
        }
        Err(keyring::KeyringError::NotFound) => Ok(false),
        Err(e) => Err(e.to_string()),
    }
}

/// 都市名を設定
#[tauri::command]
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
    // キャッシュをクリアしてから取得（get_weatherが新規取得＆キャッシュ保存を行う）
    state.weather.clear_cache().await;
    state.weather.get_weather().await.map_err(|e| e.to_string())
}

/// 天気情報をWebSocketでブロードキャスト
#[tauri::command]
pub async fn broadcast_weather_update(state: State<'_, AppState>) -> Result<(), String> {
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

    log::info!("Weather update broadcasted");
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
