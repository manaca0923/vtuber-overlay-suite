// =============================================================================
// 天気API型定義
// =============================================================================
// OpenWeatherMap APIのレスポンス型とアプリ内部で使用する天気データ型を定義
// =============================================================================

use serde::{Deserialize, Serialize};

/// OpenWeatherMap APIレスポンス
#[derive(Debug, Clone, Deserialize)]
pub struct OpenWeatherMapResponse {
    /// 天気情報配列
    pub weather: Vec<WeatherCondition>,
    /// 気温情報
    pub main: MainData,
    /// 都市名
    pub name: String,
    /// 国コード情報
    pub sys: Option<SysData>,
}

/// 天気状態
#[derive(Debug, Clone, Deserialize)]
pub struct WeatherCondition {
    /// 天気コード（800=晴れ、801=曇りなど）
    pub id: i32,
    /// 天気グループ（Clear, Clouds, Rain等）
    pub main: String,
    /// 天気の説明（日本語）
    pub description: String,
    /// アイコンコード（01d, 02n等）
    pub icon: String,
}

/// 気温・湿度データ
#[derive(Debug, Clone, Deserialize)]
pub struct MainData {
    /// 現在気温（摂氏）
    pub temp: f64,
    /// 体感気温
    pub feels_like: Option<f64>,
    /// 湿度（%）
    pub humidity: i32,
}

/// 国情報
#[derive(Debug, Clone, Deserialize)]
pub struct SysData {
    /// 国コード
    pub country: Option<String>,
}

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
    /// 天気コード（OpenWeatherMap）
    pub weather_code: i32,
    /// 取得時刻（UNIX timestamp）
    pub fetched_at: i64,
}

impl WeatherData {
    /// OpenWeatherMapレスポンスからWeatherDataを生成
    pub fn from_openweathermap(response: OpenWeatherMapResponse) -> Option<Self> {
        let weather = response.weather.first()?;

        Some(Self {
            icon: Self::code_to_emoji(weather.id, &weather.icon),
            temp: (response.main.temp * 10.0).round() / 10.0, // 小数点1桁に丸め
            description: weather.description.clone(),
            location: response.name,
            humidity: response.main.humidity,
            weather_code: weather.id,
            fetched_at: chrono::Utc::now().timestamp(),
        })
    }

    /// OpenWeatherMap天気コードから絵文字に変換
    ///
    /// 天気コード一覧: https://openweathermap.org/weather-conditions
    fn code_to_emoji(code: i32, icon: &str) -> String {
        // 昼夜判定（iconの末尾がd=昼、n=夜）
        let is_night = icon.ends_with('n');

        match code {
            // Thunderstorm（雷雨）
            200..=232 => "⛈️".to_string(),

            // Drizzle（霧雨）
            300..=321 => "🌧️".to_string(),

            // Rain（雨）
            500..=504 => "🌧️".to_string(),
            511 => "🌨️".to_string(), // 凍雨
            520..=531 => "🌧️".to_string(),

            // Snow（雪）
            600..=622 => "❄️".to_string(),

            // Atmosphere（大気現象）
            701 => "🌫️".to_string(), // 霧
            711 => "💨".to_string(), // 煙
            721 => "🌫️".to_string(), // もや
            731 | 761 => "💨".to_string(), // 砂塵
            741 => "🌫️".to_string(), // 霧
            751 => "💨".to_string(), // 砂
            762 => "🌋".to_string(), // 火山灰
            771 => "💨".to_string(), // スコール
            781 => "🌪️".to_string(), // 竜巻

            // Clear（晴れ）
            800 => if is_night { "🌙".to_string() } else { "☀️".to_string() },

            // Clouds（曇り）
            801 => if is_night { "🌙".to_string() } else { "⛅".to_string() }, // 少し曇り
            802 => "⛅".to_string(), // 散らばった雲
            803 | 804 => "☁️".to_string(), // 曇り

            _ => "🌡️".to_string(), // 不明
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_to_emoji_clear_day() {
        assert_eq!(WeatherData::code_to_emoji(800, "01d"), "☀️");
    }

    #[test]
    fn test_code_to_emoji_clear_night() {
        assert_eq!(WeatherData::code_to_emoji(800, "01n"), "🌙");
    }

    #[test]
    fn test_code_to_emoji_rain() {
        assert_eq!(WeatherData::code_to_emoji(500, "10d"), "🌧️");
    }

    #[test]
    fn test_code_to_emoji_snow() {
        assert_eq!(WeatherData::code_to_emoji(600, "13d"), "❄️");
    }

    #[test]
    fn test_code_to_emoji_thunderstorm() {
        assert_eq!(WeatherData::code_to_emoji(200, "11d"), "⛈️");
    }

    #[test]
    fn test_temp_rounding() {
        let response = OpenWeatherMapResponse {
            weather: vec![WeatherCondition {
                id: 800,
                main: "Clear".to_string(),
                description: "晴天".to_string(),
                icon: "01d".to_string(),
            }],
            main: MainData {
                temp: 25.456,
                feels_like: Some(26.0),
                humidity: 60,
            },
            name: "Tokyo".to_string(),
            sys: None,
        };

        let data = WeatherData::from_openweathermap(response).unwrap();
        assert_eq!(data.temp, 25.5); // 小数点1桁に丸め
    }
}
