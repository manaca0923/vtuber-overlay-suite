// =============================================================================
// 天気API連携モジュール
// =============================================================================
// OpenWeatherMap APIを使用して天気情報を取得
//
// 機能:
// - 都市名または緯度経度で天気情報を取得
// - 15分間のキャッシュでAPIコールを削減
// - 天気コードから絵文字への変換
//
// 使用API: OpenWeatherMap Current Weather Data
// https://openweathermap.org/current
// =============================================================================

mod cache;
mod types;

pub use cache::WeatherCache;
pub use types::{OpenWeatherMapResponse, WeatherData};

use reqwest::Client;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

/// OpenWeatherMap APIのベースURL
const OPENWEATHERMAP_API_URL: &str = "https://api.openweathermap.org/data/2.5/weather";

/// 天気APIエラー
#[derive(Debug, Error)]
pub enum WeatherError {
    #[error("API key not configured")]
    ApiKeyNotConfigured,

    #[error("City not configured")]
    CityNotConfigured,

    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("API error: {status} - {message}")]
    ApiError { status: u16, message: String },

    #[error("Parse error: {0}")]
    ParseError(String),
}

/// 天気APIクライアント
#[derive(Debug)]
pub struct WeatherClient {
    /// HTTPクライアント
    client: Client,
    /// 天気情報キャッシュ
    cache: WeatherCache,
    /// APIキー
    api_key: Arc<RwLock<Option<String>>>,
    /// 都市名（デフォルト: Tokyo）
    city: Arc<RwLock<String>>,
}

impl WeatherClient {
    /// 新しい天気クライアントを作成
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            cache: WeatherCache::new(),
            api_key: Arc::new(RwLock::new(None)),
            city: Arc::new(RwLock::new("Tokyo".to_string())),
        }
    }

    /// APIキーを設定
    pub async fn set_api_key(&self, api_key: String) {
        let mut key = self.api_key.write().await;
        *key = Some(api_key);
        log::info!("Weather API key configured");

        // APIキー変更時はキャッシュをクリア
        self.cache.clear().await;
    }

    /// APIキーが設定されているか確認
    pub async fn has_api_key(&self) -> bool {
        self.api_key.read().await.is_some()
    }

    /// 都市名を設定
    pub async fn set_city(&self, city: String) {
        let old_city = {
            let mut c = self.city.write().await;
            let old = c.clone();
            *c = city.clone();
            old
        };

        // 都市名変更時はキャッシュをクリア
        if old_city != city {
            self.cache.clear().await;
            log::info!("Weather city changed: {} -> {}", old_city, city);
        }
    }

    /// 現在の都市名を取得
    pub async fn get_city(&self) -> String {
        self.city.read().await.clone()
    }

    /// 天気情報を取得（キャッシュ優先）
    pub async fn get_weather(&self) -> Result<WeatherData, WeatherError> {
        // キャッシュをチェック
        if let Some(cached) = self.cache.get().await {
            return Ok(cached);
        }

        // APIから取得
        let data = self.fetch_weather().await?;

        // キャッシュに保存
        self.cache.set(data.clone()).await;

        Ok(data)
    }

    /// 天気情報を強制的に取得（キャッシュ無視）
    pub async fn fetch_weather(&self) -> Result<WeatherData, WeatherError> {
        let api_key = self.api_key.read().await;
        let api_key = api_key.as_ref().ok_or(WeatherError::ApiKeyNotConfigured)?;

        let city = self.city.read().await;
        if city.is_empty() {
            return Err(WeatherError::CityNotConfigured);
        }

        log::debug!("Fetching weather for city: {}", city);

        let response = self
            .client
            .get(OPENWEATHERMAP_API_URL)
            .query(&[
                ("q", city.as_str()),
                ("appid", api_key),
                ("units", "metric"), // 摂氏
                ("lang", "ja"),      // 日本語
            ])
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let message = response.text().await.unwrap_or_default();
            log::error!("Weather API error: {} - {}", status, message);
            return Err(WeatherError::ApiError {
                status: status.as_u16(),
                message,
            });
        }

        let api_response: OpenWeatherMapResponse = response.json().await?;

        WeatherData::from_openweathermap(api_response)
            .ok_or_else(|| WeatherError::ParseError("Empty weather data".to_string()))
    }

    /// キャッシュをクリア
    pub async fn clear_cache(&self) {
        self.cache.clear().await;
    }

    /// キャッシュの残りTTLを取得（秒）
    pub async fn cache_ttl_remaining(&self) -> u64 {
        self.cache.ttl_remaining().await
    }
}

impl Default for WeatherClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        let client = WeatherClient::new();
        assert!(!client.has_api_key().await);
        assert_eq!(client.get_city().await, "Tokyo");
    }

    #[tokio::test]
    async fn test_set_api_key() {
        let client = WeatherClient::new();
        client.set_api_key("test_key".to_string()).await;
        assert!(client.has_api_key().await);
    }

    #[tokio::test]
    async fn test_set_city() {
        let client = WeatherClient::new();
        client.set_city("Osaka".to_string()).await;
        assert_eq!(client.get_city().await, "Osaka");
    }

    #[tokio::test]
    async fn test_fetch_without_api_key() {
        let client = WeatherClient::new();
        let result = client.fetch_weather().await;
        assert!(matches!(result, Err(WeatherError::ApiKeyNotConfigured)));
    }

    #[tokio::test]
    async fn test_fetch_with_empty_city() {
        let client = WeatherClient::new();
        client.set_api_key("test_key".to_string()).await;
        client.set_city("".to_string()).await;
        let result = client.fetch_weather().await;
        assert!(matches!(result, Err(WeatherError::CityNotConfigured)));
    }
}
