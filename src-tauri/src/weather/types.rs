// =============================================================================
// 天気API型定義
// =============================================================================
// Open-Meteo APIのレスポンス型とアプリ内部で使用する天気データ型を定義
// =============================================================================

use serde::{Deserialize, Serialize};

// =============================================================================
// Open-Meteo Geocoding API
// =============================================================================

/// Geocoding APIレスポンス
#[derive(Debug, Clone, Deserialize)]
pub struct GeocodingResponse {
    /// 検索結果の都市リスト
    pub results: Option<Vec<GeocodingResult>>,
}

/// Geocoding検索結果
#[derive(Debug, Clone, Deserialize)]
pub struct GeocodingResult {
    /// 都市ID（APIレスポンスに含まれるがアプリ内では未使用）
    #[allow(dead_code)]
    pub id: i64,
    /// 都市名
    pub name: String,
    /// 緯度
    pub latitude: f64,
    /// 経度
    pub longitude: f64,
    /// 国名
    pub country: Option<String>,
    /// 行政区画（都道府県・州）
    pub admin1: Option<String>,
}

// =============================================================================
// Open-Meteo Weather API
// =============================================================================

/// Weather APIレスポンス
#[derive(Debug, Clone, Deserialize)]
pub struct OpenMeteoResponse {
    /// 現在の天気データ
    pub current: CurrentWeather,
}

/// 現在の天気データ
#[derive(Debug, Clone, Deserialize)]
pub struct CurrentWeather {
    /// 気温（摂氏）
    pub temperature_2m: f64,
    /// 湿度（%）
    pub relative_humidity_2m: i32,
    /// WMO天気コード
    pub weather_code: i32,
    /// 昼夜判定（0=夜, 1=昼）
    pub is_day: i32,
}

// =============================================================================
// アプリ内部データ型
// =============================================================================

/// アプリ内部で使用する天気データ
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeatherData {
    /// 天気アイコン（絵文字）
    pub icon: String,
    /// 気温（摂氏、小数点1桁）
    pub temp: f64,
    /// 天気の説明
    pub description: String,
    /// 地域名
    pub location: String,
    /// 湿度（%）
    pub humidity: i32,
    /// 天気コード（WMO）
    pub weather_code: i32,
    /// 取得時刻（UNIX timestamp）
    pub fetched_at: i64,
}

impl WeatherData {
    /// Open-MeteoレスポンスからWeatherDataを生成
    pub fn from_open_meteo(response: OpenMeteoResponse, location: String) -> Self {
        let current = response.current;
        let is_day = current.is_day == 1;

        Self {
            icon: Self::wmo_code_to_emoji(current.weather_code, is_day),
            temp: (current.temperature_2m * 10.0).round() / 10.0,
            description: Self::wmo_code_to_description(current.weather_code),
            location,
            humidity: current.relative_humidity_2m,
            weather_code: current.weather_code,
            fetched_at: chrono::Utc::now().timestamp(),
        }
    }

    /// WMOコードから絵文字に変換
    ///
    /// WMO天気コード: https://open-meteo.com/en/docs
    pub fn wmo_code_to_emoji(code: i32, is_day: bool) -> String {
        match code {
            // 晴天
            0 => if is_day { "☀️" } else { "🌙" }.to_string(),
            // 曇り (おおむね晴れ〜曇り)
            1..=3 => if is_day { "⛅" } else { "🌙" }.to_string(),
            // 霧
            45 | 48 => "🌫️".to_string(),
            // 霧雨
            51..=57 => "🌧️".to_string(),
            // 雨
            61..=67 => "🌧️".to_string(),
            // 雪
            71..=77 => "❄️".to_string(),
            // しゅう雨
            80..=82 => "🌧️".to_string(),
            // にわか雪
            85 | 86 => "🌨️".to_string(),
            // 雷雨
            95..=99 => "⛈️".to_string(),
            // 不明
            _ => "🌡️".to_string(),
        }
    }

    /// WMOコードから日本語説明に変換
    pub fn wmo_code_to_description(code: i32) -> String {
        match code {
            0 => "晴天",
            1 => "おおむね晴れ",
            2 => "一部曇り",
            3 => "曇り",
            45 => "霧",
            48 => "着氷性の霧",
            51 => "弱い霧雨",
            53 => "霧雨",
            55 => "強い霧雨",
            56 => "弱い着氷性霧雨",
            57 => "強い着氷性霧雨",
            61 => "弱い雨",
            63 => "雨",
            65 => "強い雨",
            66 => "弱い着氷性の雨",
            67 => "強い着氷性の雨",
            71 => "弱い雪",
            73 => "雪",
            75 => "強い雪",
            77 => "霧雪",
            80 => "弱いにわか雨",
            81 => "にわか雨",
            82 => "激しいにわか雨",
            85 => "弱いにわか雪",
            86 => "激しいにわか雪",
            95 => "雷雨",
            96 => "雹を伴う弱い雷雨",
            99 => "雹を伴う激しい雷雨",
            _ => "不明",
        }
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // GeocodingResponse パーステスト
    // =========================================================================

    #[test]
    fn test_geocoding_response_with_results() {
        let json = r#"{
            "results": [
                {
                    "id": 1850147,
                    "name": "Tokyo",
                    "latitude": 35.6895,
                    "longitude": 139.6917,
                    "country": "Japan",
                    "admin1": "Tokyo"
                }
            ]
        }"#;

        let response: GeocodingResponse = serde_json::from_str(json).unwrap();
        assert!(response.results.is_some());
        let results = response.results.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Tokyo");
        assert_eq!(results[0].latitude, 35.6895);
        assert_eq!(results[0].longitude, 139.6917);
    }

    #[test]
    fn test_geocoding_response_empty_results() {
        // 存在しない都市名の場合、APIは空のresults配列を返す
        let json = r#"{"results": []}"#;

        let response: GeocodingResponse = serde_json::from_str(json).unwrap();
        assert!(response.results.is_some());
        let results = response.results.unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_geocoding_response_no_results_field() {
        // 一致する都市がない場合、APIはresultsフィールド自体を省略することがある
        let json = r#"{}"#;

        let response: GeocodingResponse = serde_json::from_str(json).unwrap();
        assert!(response.results.is_none());
    }

    #[test]
    fn test_geocoding_response_null_results() {
        // resultsがnullの場合
        let json = r#"{"results": null}"#;

        let response: GeocodingResponse = serde_json::from_str(json).unwrap();
        assert!(response.results.is_none());
    }

    #[test]
    fn test_geocoding_response_optional_fields() {
        // countryとadmin1がオプションの場合
        let json = r#"{
            "results": [
                {
                    "id": 12345,
                    "name": "SomePlace",
                    "latitude": 10.0,
                    "longitude": 20.0
                }
            ]
        }"#;

        let response: GeocodingResponse = serde_json::from_str(json).unwrap();
        let results = response.results.unwrap();
        assert_eq!(results[0].name, "SomePlace");
        assert!(results[0].country.is_none());
        assert!(results[0].admin1.is_none());
    }

    // =========================================================================
    // WMO Code テスト
    // =========================================================================

    #[test]
    fn test_wmo_code_to_emoji_clear_day() {
        assert_eq!(WeatherData::wmo_code_to_emoji(0, true), "☀️");
    }

    #[test]
    fn test_wmo_code_to_emoji_clear_night() {
        assert_eq!(WeatherData::wmo_code_to_emoji(0, false), "🌙");
    }

    #[test]
    fn test_wmo_code_to_emoji_cloudy() {
        assert_eq!(WeatherData::wmo_code_to_emoji(3, true), "⛅");
    }

    #[test]
    fn test_wmo_code_to_emoji_rain() {
        assert_eq!(WeatherData::wmo_code_to_emoji(63, true), "🌧️");
    }

    #[test]
    fn test_wmo_code_to_emoji_snow() {
        assert_eq!(WeatherData::wmo_code_to_emoji(73, true), "❄️");
    }

    #[test]
    fn test_wmo_code_to_emoji_thunderstorm() {
        assert_eq!(WeatherData::wmo_code_to_emoji(95, true), "⛈️");
    }

    #[test]
    fn test_wmo_code_to_emoji_fog() {
        assert_eq!(WeatherData::wmo_code_to_emoji(45, true), "🌫️");
    }

    #[test]
    fn test_wmo_code_to_description() {
        assert_eq!(WeatherData::wmo_code_to_description(0), "晴天");
        assert_eq!(WeatherData::wmo_code_to_description(63), "雨");
        assert_eq!(WeatherData::wmo_code_to_description(73), "雪");
        assert_eq!(WeatherData::wmo_code_to_description(95), "雷雨");
    }

    #[test]
    fn test_from_open_meteo() {
        let response = OpenMeteoResponse {
            current: CurrentWeather {
                temperature_2m: 25.456,
                relative_humidity_2m: 60,
                weather_code: 0,
                is_day: 1,
            },
        };

        let data = WeatherData::from_open_meteo(response, "Tokyo".to_string());

        assert_eq!(data.icon, "☀️");
        assert_eq!(data.temp, 25.5); // 小数点1桁に丸め
        assert_eq!(data.description, "晴天");
        assert_eq!(data.location, "Tokyo");
        assert_eq!(data.humidity, 60);
        assert_eq!(data.weather_code, 0);
        assert!(data.fetched_at > 0);
    }

    #[test]
    fn test_from_open_meteo_negative_temp() {
        let response = OpenMeteoResponse {
            current: CurrentWeather {
                temperature_2m: -5.7,
                relative_humidity_2m: 85,
                weather_code: 73,
                is_day: 1,
            },
        };

        let data = WeatherData::from_open_meteo(response, "Sapporo".to_string());

        assert_eq!(data.temp, -5.7);
        assert_eq!(data.icon, "❄️");
        assert_eq!(data.description, "雪");
    }

    #[test]
    fn test_from_open_meteo_night() {
        let response = OpenMeteoResponse {
            current: CurrentWeather {
                temperature_2m: 18.0,
                relative_humidity_2m: 70,
                weather_code: 0,
                is_day: 0, // 夜
            },
        };

        let data = WeatherData::from_open_meteo(response, "Osaka".to_string());

        assert_eq!(data.icon, "🌙");
    }
}
