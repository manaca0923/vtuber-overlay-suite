// =============================================================================
// 天気API連携モジュール
// =============================================================================
// Open-Meteo APIを使用して天気情報を取得（APIキー不要）
//
// 機能:
// - 都市名で天気情報を取得（Geocoding API経由）
// - 15分間のキャッシュでAPIコールを削減
// - WMOコードから絵文字への変換
//
// 使用API:
// - Open-Meteo Geocoding API: https://open-meteo.com/en/docs/geocoding-api
// - Open-Meteo Weather API: https://open-meteo.com/en/docs
// =============================================================================

mod auto_updater;
mod cache;
mod types;

pub use auto_updater::WeatherAutoUpdater;
pub use cache::WeatherCache;
pub use types::{GeocodingResponse, OpenMeteoResponse, WeatherData};

use crate::config::{http_timeout, HTTP_TIMEOUT_SECS};
use reqwest::Client;
use std::sync::Arc;
#[cfg(test)]
use std::time::Duration;
use thiserror::Error;
use tokio::sync::RwLock;

/// Open-Meteo Geocoding APIのベースURL
const GEOCODING_API_URL: &str = "https://geocoding-api.open-meteo.com/v1/search";

/// Open-Meteo Weather APIのベースURL
const WEATHER_API_URL: &str = "https://api.open-meteo.com/v1/forecast";

/// 天気APIエラー
#[derive(Debug, Error)]
pub enum WeatherError {
    #[error("City not configured")]
    CityNotConfigured,

    #[error("City not found: {0}")]
    CityNotFound(String),

    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("API error: {status} - {message}")]
    ApiError { status: u16, message: String },

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Request timeout: API応答がありません")]
    Timeout,
}

/// 緯度経度キャッシュエントリ
#[derive(Debug, Clone)]
struct CoordsCache {
    city: String,
    latitude: f64,
    longitude: f64,
    display_name: String,
}

/// 天気APIクライアント
#[derive(Debug)]
pub struct WeatherClient {
    /// HTTPクライアント
    client: Client,
    /// 天気情報キャッシュ
    cache: WeatherCache,
    /// 都市名（デフォルト: Tokyo）
    city: Arc<RwLock<String>>,
    /// 緯度経度キャッシュ
    coords_cache: Arc<RwLock<Option<CoordsCache>>>,
    /// テスト用: GeocodingベースURL
    #[cfg(test)]
    geocoding_base_url: String,
    /// テスト用: WeatherベースURL
    #[cfg(test)]
    weather_base_url: String,
}

impl WeatherClient {
    /// 新しい天気クライアントを作成
    pub fn new() -> Self {
        // タイムアウト付きのHTTPクライアントを構築
        let client = Client::builder()
            .timeout(http_timeout())
            .build()
            .expect("Failed to build HTTP client with timeout - this should never fail");

        Self {
            client,
            cache: WeatherCache::new(),
            city: Arc::new(RwLock::new("Tokyo".to_string())),
            coords_cache: Arc::new(RwLock::new(None)),
            #[cfg(test)]
            geocoding_base_url: GEOCODING_API_URL.to_string(),
            #[cfg(test)]
            weather_base_url: WEATHER_API_URL.to_string(),
        }
    }

    /// テスト用: カスタムベースURLで天気クライアントを作成
    #[cfg(test)]
    pub fn new_with_base_urls(geocoding_base_url: String, weather_base_url: String) -> Self {
        Self::new_with_base_urls_and_timeout(geocoding_base_url, weather_base_url, http_timeout())
    }

    /// テスト用: カスタムベースURLとタイムアウトで天気クライアントを作成
    #[cfg(test)]
    pub fn new_with_base_urls_and_timeout(
        geocoding_base_url: String,
        weather_base_url: String,
        timeout: Duration,
    ) -> Self {
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .expect("Failed to build HTTP client with timeout");

        Self {
            client,
            cache: WeatherCache::new(),
            city: Arc::new(RwLock::new("Tokyo".to_string())),
            coords_cache: Arc::new(RwLock::new(None)),
            geocoding_base_url,
            weather_base_url,
        }
    }

    /// GeocodingベースURLを取得
    #[inline]
    fn get_geocoding_base_url(&self) -> &str {
        #[cfg(test)]
        {
            &self.geocoding_base_url
        }
        #[cfg(not(test))]
        {
            GEOCODING_API_URL
        }
    }

    /// WeatherベースURLを取得
    #[inline]
    fn get_weather_base_url(&self) -> &str {
        #[cfg(test)]
        {
            &self.weather_base_url
        }
        #[cfg(not(test))]
        {
            WEATHER_API_URL
        }
    }

    /// 都市名を設定
    /// 空白のみの入力は空文字列に正規化される
    pub async fn set_city(&self, city: String) {
        // 前後の空白を除去して正規化
        let normalized_city = city.trim().to_string();

        let old_city = {
            let mut c = self.city.write().await;
            let old = c.clone();
            *c = normalized_city.clone();
            old
        };

        // 都市名変更時はキャッシュをクリア
        if old_city != normalized_city {
            self.cache.clear().await;
            // 緯度経度キャッシュもクリア
            let mut coords = self.coords_cache.write().await;
            *coords = None;
            log::info!("Weather city changed: {} -> {}", old_city, normalized_city);
        }
    }

    /// 現在の都市名を取得
    pub async fn get_city(&self) -> String {
        self.city.read().await.clone()
    }

    /// 表示用の地名を構築（都市名, 行政区画, 国）
    fn build_display_name(
        name: &str,
        admin1: &Option<String>,
        country: &Option<String>,
    ) -> String {
        let mut parts = vec![name.to_string()];

        if let Some(a) = admin1 {
            if !a.is_empty() && a != name {
                parts.push(a.clone());
            }
        }

        if let Some(c) = country {
            if !c.is_empty() {
                parts.push(c.clone());
            }
        }

        parts.join(", ")
    }

    /// 都市名から緯度経度を取得（Geocoding API）
    async fn geocode_city(&self, city: &str) -> Result<(f64, f64, String), WeatherError> {
        // キャッシュをチェック
        {
            let cache = self.coords_cache.read().await;
            if let Some(cached) = cache.as_ref() {
                if cached.city == city {
                    return Ok((cached.latitude, cached.longitude, cached.display_name.clone()));
                }
            }
        }

        log::debug!("Geocoding city: {}", city);

        let response = self
            .client
            .get(self.get_geocoding_base_url())
            .query(&[("name", city), ("count", "1"), ("language", "ja")])
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    log::warn!("Geocoding API request timed out after {}s", HTTP_TIMEOUT_SECS);
                    WeatherError::Timeout
                } else {
                    WeatherError::HttpError(e)
                }
            })?;

        let status = response.status();
        if !status.is_success() {
            let message = response.text().await.unwrap_or_default();
            log::error!("Geocoding API error: {} - {}", status, message);
            return Err(WeatherError::ApiError {
                status: status.as_u16(),
                message,
            });
        }

        let geo_response: GeocodingResponse = response.json().await.map_err(|e| {
            WeatherError::ParseError(format!("Failed to parse geocoding response: {}", e))
        })?;

        let result = geo_response
            .results
            .and_then(|r| r.into_iter().next())
            .ok_or_else(|| WeatherError::CityNotFound(city.to_string()))?;

        // 表示名を構築: "都市名, 行政区画, 国" の形式で同名都市の混乱を避ける
        let display_name = Self::build_display_name(&result.name, &result.admin1, &result.country);

        // キャッシュに保存
        {
            let mut cache = self.coords_cache.write().await;
            *cache = Some(CoordsCache {
                city: city.to_string(),
                latitude: result.latitude,
                longitude: result.longitude,
                display_name: display_name.clone(),
            });
        }

        log::debug!(
            "Geocoded: {} -> ({}, {}) as {}",
            city,
            result.latitude,
            result.longitude,
            display_name
        );

        Ok((result.latitude, result.longitude, display_name))
    }

    /// 天気情報を取得（キャッシュ優先）
    pub async fn get_weather(&self) -> Result<WeatherData, WeatherError> {
        // 一度だけ都市を読み取り、同じ値をリクエストとキャッシュキーに使用
        let city = self.city.read().await.clone();

        // キャッシュをチェック（都市名も検証）
        if let Some(cached) = self.cache.get(&city).await {
            return Ok(cached);
        }

        // APIから取得
        let data = self.fetch_weather_for_city(&city).await?;

        // キャッシュに保存
        self.cache.set(data.clone(), city).await;

        Ok(data)
    }

    /// 天気情報を強制的に取得（キャッシュ無視）
    pub async fn fetch_weather(&self) -> Result<WeatherData, WeatherError> {
        let city = self.city.read().await.clone();
        self.fetch_weather_for_city(&city).await
    }

    /// 指定された都市の天気情報を取得（内部用）
    async fn fetch_weather_for_city(&self, city: &str) -> Result<WeatherData, WeatherError> {
        if city.is_empty() {
            return Err(WeatherError::CityNotConfigured);
        }

        // 都市名から緯度経度を取得
        let (lat, lon, location_name) = self.geocode_city(city).await?;

        log::debug!(
            "Fetching weather for: {} ({}, {})",
            location_name,
            lat,
            lon
        );

        let response = self
            .client
            .get(self.get_weather_base_url())
            .query(&[
                ("latitude", lat.to_string()),
                ("longitude", lon.to_string()),
                (
                    "current",
                    "temperature_2m,relative_humidity_2m,weather_code,is_day".to_string(),
                ),
            ])
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    log::warn!("Weather API request timed out after {}s", HTTP_TIMEOUT_SECS);
                    WeatherError::Timeout
                } else {
                    WeatherError::HttpError(e)
                }
            })?;

        let status = response.status();
        if !status.is_success() {
            let message = response.text().await.unwrap_or_default();
            log::error!("Weather API error: {} - {}", status, message);
            return Err(WeatherError::ApiError {
                status: status.as_u16(),
                message,
            });
        }

        let api_response: OpenMeteoResponse = response.json().await.map_err(|e| {
            WeatherError::ParseError(format!("Failed to parse weather response: {}", e))
        })?;

        Ok(WeatherData::from_open_meteo(api_response, location_name))
    }

    /// キャッシュをクリア
    pub async fn clear_cache(&self) {
        self.cache.clear().await;
    }

    /// キャッシュの残りTTLを取得（秒）
    pub async fn cache_ttl_remaining(&self) -> u64 {
        let city = self.city.read().await.clone();
        self.cache.ttl_remaining(&city).await
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
        assert_eq!(client.get_city().await, "Tokyo");
    }

    #[tokio::test]
    async fn test_set_city() {
        let client = WeatherClient::new();
        client.set_city("Osaka".to_string()).await;
        assert_eq!(client.get_city().await, "Osaka");
    }

    #[tokio::test]
    async fn test_fetch_with_empty_city() {
        let client = WeatherClient::new();
        client.set_city("".to_string()).await;
        let result = client.fetch_weather().await;
        assert!(matches!(result, Err(WeatherError::CityNotConfigured)));
    }

    #[tokio::test]
    async fn test_city_change_clears_coords_cache() {
        let client = WeatherClient::new();

        // 初期状態ではキャッシュは空
        {
            let cache = client.coords_cache.read().await;
            assert!(cache.is_none());
        }

        // 都市を変更
        client.set_city("Osaka".to_string()).await;

        // キャッシュはまだ空（APIを呼んでいない）
        {
            let cache = client.coords_cache.read().await;
            assert!(cache.is_none());
        }
    }

    #[tokio::test]
    async fn test_set_city_trims_whitespace() {
        let client = WeatherClient::new();
        client.set_city("  Osaka  ".to_string()).await;
        assert_eq!(client.get_city().await, "Osaka");
    }

    #[tokio::test]
    async fn test_set_city_whitespace_only_becomes_empty() {
        let client = WeatherClient::new();
        client.set_city("   ".to_string()).await;
        assert_eq!(client.get_city().await, "");
    }

    #[test]
    fn test_build_display_name_city_only() {
        let name = WeatherClient::build_display_name("Tokyo", &None, &None);
        assert_eq!(name, "Tokyo");
    }

    #[test]
    fn test_build_display_name_with_country() {
        let name = WeatherClient::build_display_name(
            "Tokyo",
            &None,
            &Some("Japan".to_string()),
        );
        assert_eq!(name, "Tokyo, Japan");
    }

    #[test]
    fn test_build_display_name_with_admin1_and_country() {
        let name = WeatherClient::build_display_name(
            "Shibuya",
            &Some("Tokyo".to_string()),
            &Some("Japan".to_string()),
        );
        assert_eq!(name, "Shibuya, Tokyo, Japan");
    }

    #[test]
    fn test_build_display_name_skips_duplicate_admin1() {
        // admin1が都市名と同じ場合は重複を避ける
        let name = WeatherClient::build_display_name(
            "Tokyo",
            &Some("Tokyo".to_string()),
            &Some("Japan".to_string()),
        );
        assert_eq!(name, "Tokyo, Japan");
    }

    #[test]
    fn test_build_display_name_skips_empty_parts() {
        let name = WeatherClient::build_display_name(
            "Tokyo",
            &Some("".to_string()),
            &Some("Japan".to_string()),
        );
        assert_eq!(name, "Tokyo, Japan");
    }

    // =========================================================================
    // WeatherError テスト
    // =========================================================================

    #[test]
    fn test_weather_error_city_not_configured() {
        let err = WeatherError::CityNotConfigured;
        assert_eq!(format!("{}", err), "City not configured");
    }

    #[test]
    fn test_weather_error_city_not_found() {
        let err = WeatherError::CityNotFound("UnknownCity".to_string());
        assert_eq!(format!("{}", err), "City not found: UnknownCity");
    }

    #[test]
    fn test_weather_error_api_error() {
        let err = WeatherError::ApiError {
            status: 404,
            message: "Not Found".to_string(),
        };
        assert_eq!(format!("{}", err), "API error: 404 - Not Found");
    }

    #[test]
    fn test_weather_error_parse_error() {
        let err = WeatherError::ParseError("Invalid JSON".to_string());
        assert_eq!(format!("{}", err), "Parse error: Invalid JSON");
    }

    #[test]
    fn test_weather_error_timeout() {
        let err = WeatherError::Timeout;
        assert_eq!(format!("{}", err), "Request timeout: API応答がありません");
    }

    // =========================================================================
    // CityNotFound判定ロジックのテスト
    // =========================================================================

    #[test]
    fn test_geocoding_results_none_produces_city_not_found() {
        // geocode_city内のロジックをシミュレート
        // results.and_then(|r| r.into_iter().next()).ok_or_else(...)
        let response = GeocodingResponse { results: None };
        let result = response
            .results
            .and_then(|r| r.into_iter().next())
            .ok_or_else(|| WeatherError::CityNotFound("TestCity".to_string()));

        assert!(matches!(result, Err(WeatherError::CityNotFound(city)) if city == "TestCity"));
    }

    #[test]
    fn test_geocoding_results_empty_produces_city_not_found() {
        // 空配列の場合
        let response = GeocodingResponse { results: Some(vec![]) };
        let result = response
            .results
            .and_then(|r| r.into_iter().next())
            .ok_or_else(|| WeatherError::CityNotFound("EmptyCity".to_string()));

        assert!(matches!(result, Err(WeatherError::CityNotFound(city)) if city == "EmptyCity"));
    }

    #[test]
    fn test_geocoding_results_with_data_produces_success() {
        use types::GeocodingResult;

        let response = GeocodingResponse {
            results: Some(vec![GeocodingResult {
                id: 1,
                name: "Tokyo".to_string(),
                latitude: 35.6895,
                longitude: 139.6917,
                country: Some("Japan".to_string()),
                admin1: Some("Tokyo".to_string()),
            }]),
        };
        let result = response
            .results
            .and_then(|r| r.into_iter().next())
            .ok_or_else(|| WeatherError::CityNotFound("TestCity".to_string()));

        assert!(result.is_ok());
        let geo_result = result.unwrap();
        assert_eq!(geo_result.name, "Tokyo");
        assert_eq!(geo_result.latitude, 35.6895);
    }

    // =========================================================================
    // HTTPモックテスト（mockito使用）
    // =========================================================================

    use mockito::Server;

    /// テスト用のセットアップを行い、(ServerGuard, WeatherClient)を返す
    async fn setup_test_client() -> (mockito::ServerGuard, WeatherClient) {
        let server = Server::new_async().await;
        let client = WeatherClient::new_with_base_urls(
            format!("{}/v1/search", server.url()),
            format!("{}/v1/forecast", server.url()),
        );
        (server, client)
    }

    /// Geocoding APIの成功レスポンスをモックする
    async fn mock_geocoding_success(server: &mut mockito::ServerGuard) -> mockito::Mock {
        server
            .mock("GET", "/v1/search")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"results": [{"id": 1, "name": "Tokyo", "latitude": 35.6895, "longitude": 139.6917, "country": "Japan", "admin1": "Tokyo"}]}"#)
            .create_async()
            .await
    }

    #[tokio::test]
    async fn test_geocoding_api_500_error() {
        let (mut server, client) = setup_test_client().await;

        let _mock = server
            .mock("GET", "/v1/search")
            .match_query(mockito::Matcher::Any)
            .with_status(500)
            .with_body("Internal Server Error")
            .create_async()
            .await;

        client.set_city("Tokyo".to_string()).await;

        let result = client.fetch_weather().await;
        match result {
            Err(WeatherError::ApiError { status, .. }) => {
                assert_eq!(status, 500);
            }
            _ => panic!("Expected ApiError with status 500, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_geocoding_api_400_error() {
        let (mut server, client) = setup_test_client().await;

        let _mock = server
            .mock("GET", "/v1/search")
            .match_query(mockito::Matcher::Any)
            .with_status(400)
            .with_body("Bad Request")
            .create_async()
            .await;

        client.set_city("InvalidCity!@#".to_string()).await;

        let result = client.fetch_weather().await;
        match result {
            Err(WeatherError::ApiError { status, .. }) => {
                assert_eq!(status, 400);
            }
            _ => panic!("Expected ApiError with status 400, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_geocoding_city_not_found() {
        let (mut server, client) = setup_test_client().await;

        // Geocoding APIは成功するが、結果が空
        let _mock = server
            .mock("GET", "/v1/search")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"results": []}"#)
            .create_async()
            .await;

        client.set_city("NonexistentCity123".to_string()).await;

        let result = client.fetch_weather().await;
        assert!(matches!(result, Err(WeatherError::CityNotFound(_))));
    }

    #[tokio::test]
    async fn test_weather_api_500_error() {
        let (mut server, client) = setup_test_client().await;

        // Geocoding APIは成功
        let _geocoding_mock = mock_geocoding_success(&mut server).await;

        // Weather APIは500エラー
        let _weather_mock = server
            .mock("GET", "/v1/forecast")
            .match_query(mockito::Matcher::Any)
            .with_status(500)
            .with_body("Internal Server Error")
            .create_async()
            .await;

        client.set_city("Tokyo".to_string()).await;

        let result = client.fetch_weather().await;
        match result {
            Err(WeatherError::ApiError { status, .. }) => {
                assert_eq!(status, 500);
            }
            _ => panic!("Expected ApiError with status 500, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_weather_api_503_service_unavailable() {
        let (mut server, client) = setup_test_client().await;

        // Geocoding APIは成功
        let _geocoding_mock = mock_geocoding_success(&mut server).await;

        // Weather APIは503エラー
        let _weather_mock = server
            .mock("GET", "/v1/forecast")
            .match_query(mockito::Matcher::Any)
            .with_status(503)
            .with_body("Service Unavailable")
            .create_async()
            .await;

        client.set_city("Tokyo".to_string()).await;

        let result = client.fetch_weather().await;
        match result {
            Err(WeatherError::ApiError { status, .. }) => {
                assert_eq!(status, 503);
            }
            _ => panic!("Expected ApiError with status 503, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_weather_fetch_success() {
        let (mut server, client) = setup_test_client().await;

        // Geocoding APIは成功
        let _geocoding_mock = mock_geocoding_success(&mut server).await;

        // Weather APIも成功（緯度経度はmock_geocoding_successの値に合わせる）
        let _weather_mock = server
            .mock("GET", "/v1/forecast")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("latitude".into(), "35.6895".into()),
                mockito::Matcher::UrlEncoded("longitude".into(), "139.6917".into()),
                mockito::Matcher::UrlEncoded("current".into(), "temperature_2m,relative_humidity_2m,weather_code,is_day".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "current": {
                    "temperature_2m": 25.5,
                    "relative_humidity_2m": 60,
                    "weather_code": 0,
                    "is_day": 1
                }
            }"#)
            .create_async()
            .await;

        client.set_city("Tokyo".to_string()).await;

        let result = client.fetch_weather().await;
        assert!(result.is_ok());

        let weather = result.unwrap();
        assert_eq!(weather.temp, 25.5);
        assert_eq!(weather.humidity, 60);
        assert_eq!(weather.location, "Tokyo, Japan");
    }

    #[tokio::test]
    async fn test_weather_api_invalid_json() {
        let (mut server, client) = setup_test_client().await;

        // Geocoding APIは成功
        let _geocoding_mock = mock_geocoding_success(&mut server).await;

        // Weather APIは不正なJSON
        let _weather_mock = server
            .mock("GET", "/v1/forecast")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("not valid json")
            .create_async()
            .await;

        client.set_city("Tokyo".to_string()).await;

        let result = client.fetch_weather().await;
        assert!(matches!(result, Err(WeatherError::ParseError(_))));
    }

    // =============================================================================
    // タイムアウト関連
    // =============================================================================
    //
    // 注: mockitoではタイムアウト動作の完全なシミュレーションが困難なため、
    // 実際のタイムアウト動作テストは除外しています。
    // タイムアウト機能自体は以下のように実装されています:
    // - HTTPクライアントに10秒のタイムアウトを設定 (HTTP_TIMEOUT_SECS)
    // - タイムアウト発生時は WeatherError::Timeout を返す
    // - is_timeout() でタイムアウトエラーを判別
    //
    // test_weather_error_timeout() でエラーメッセージのフォーマットを検証済み
}
