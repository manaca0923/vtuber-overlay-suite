// =============================================================================
// 天気情報キャッシュ
// =============================================================================
// 天気情報を15分間キャッシュしてAPIコールを削減
// =============================================================================

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use super::types::WeatherData;

/// キャッシュのTTL（15分）
const CACHE_TTL_SECS: u64 = 15 * 60;

/// キャッシュエントリ
#[derive(Debug, Clone)]
struct CacheEntry {
    /// キャッシュされたデータ
    data: WeatherData,
    /// キャッシュ対象の都市名
    city: String,
    /// キャッシュ作成時刻
    created_at: Instant,
}

impl CacheEntry {
    fn new(data: WeatherData, city: String) -> Self {
        Self {
            data,
            city,
            created_at: Instant::now(),
        }
    }

    /// 指定されたTTLに対して期限切れかどうかを判定
    fn is_expired(&self, ttl_secs: u64) -> bool {
        self.created_at.elapsed() > Duration::from_secs(ttl_secs)
    }

    /// 指定された都市と一致するかを判定
    fn matches_city(&self, city: &str) -> bool {
        self.city == city
    }
}

/// 天気情報キャッシュ
#[derive(Debug)]
pub struct WeatherCache {
    /// キャッシュエントリ（都市名でキャッシュ）
    entry: Arc<RwLock<Option<CacheEntry>>>,
    /// キャッシュのTTL（秒）
    ttl_secs: u64,
}

impl WeatherCache {
    /// 新しいキャッシュを作成
    pub fn new() -> Self {
        Self {
            entry: Arc::new(RwLock::new(None)),
            ttl_secs: CACHE_TTL_SECS,
        }
    }

    /// カスタムTTLでキャッシュを作成（テスト用）
    #[cfg(test)]
    pub fn with_ttl(ttl_secs: u64) -> Self {
        Self {
            entry: Arc::new(RwLock::new(None)),
            ttl_secs,
        }
    }

    /// キャッシュから天気データを取得
    ///
    /// キャッシュがない、期限切れ、または都市が異なる場合はNoneを返す
    pub async fn get(&self, city: &str) -> Option<WeatherData> {
        let entry = self.entry.read().await;
        match entry.as_ref() {
            Some(e) if !e.is_expired(self.ttl_secs) && e.matches_city(city) => {
                log::debug!("Weather cache hit for city: {}", city);
                Some(e.data.clone())
            }
            Some(e) if !e.matches_city(city) => {
                log::debug!(
                    "Weather cache miss: city mismatch (cached: {}, requested: {})",
                    e.city,
                    city
                );
                None
            }
            Some(_) => {
                log::debug!("Weather cache expired for city: {}", city);
                None
            }
            None => {
                log::debug!("Weather cache miss: no entry");
                None
            }
        }
    }

    /// キャッシュに天気データを保存
    pub async fn set(&self, data: WeatherData, city: String) {
        let mut entry = self.entry.write().await;
        *entry = Some(CacheEntry::new(data, city.clone()));
        log::debug!("Weather data cached for city: {} (TTL: {}s)", city, self.ttl_secs);
    }

    /// キャッシュをクリア
    pub async fn clear(&self) {
        let mut entry = self.entry.write().await;
        *entry = None;
        log::debug!("Weather cache cleared");
    }

    /// キャッシュの残り有効期限（秒）を取得
    ///
    /// キャッシュがない場合は0を返す
    pub async fn ttl_remaining(&self) -> u64 {
        let entry = self.entry.read().await;
        match entry.as_ref() {
            Some(e) => {
                let elapsed = e.created_at.elapsed().as_secs();
                if elapsed >= self.ttl_secs {
                    0
                } else {
                    self.ttl_secs - elapsed
                }
            }
            None => 0,
        }
    }
}

impl Default for WeatherCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_weather_data() -> WeatherData {
        WeatherData {
            icon: "☀️".to_string(),
            temp: 25.5,
            description: "晴天".to_string(),
            location: "Tokyo".to_string(),
            humidity: 60,
            weather_code: 800,
            fetched_at: chrono::Utc::now().timestamp(),
        }
    }

    #[tokio::test]
    async fn test_cache_set_and_get() {
        let cache = WeatherCache::new();
        let data = create_test_weather_data();

        cache.set(data.clone(), "Tokyo".to_string()).await;

        let cached = cache.get("Tokyo").await;
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().location, "Tokyo");
    }

    #[tokio::test]
    async fn test_cache_miss() {
        let cache = WeatherCache::new();
        let cached = cache.get("Tokyo").await;
        assert!(cached.is_none());
    }

    #[tokio::test]
    async fn test_cache_clear() {
        let cache = WeatherCache::new();
        let data = create_test_weather_data();

        cache.set(data, "Tokyo".to_string()).await;
        assert!(cache.get("Tokyo").await.is_some());

        cache.clear().await;
        assert!(cache.get("Tokyo").await.is_none());
    }

    #[tokio::test]
    async fn test_cache_ttl_remaining() {
        let cache = WeatherCache::with_ttl(10);
        let data = create_test_weather_data();

        // キャッシュなしの場合は0
        assert_eq!(cache.ttl_remaining().await, 0);

        cache.set(data, "Tokyo".to_string()).await;

        // キャッシュ直後はTTLに近い値
        let ttl = cache.ttl_remaining().await;
        assert!(ttl > 0 && ttl <= 10);
    }

    #[tokio::test]
    async fn test_cache_expiry_with_short_ttl() {
        // 短いTTL（1秒）でテスト
        let cache = WeatherCache::with_ttl(1);
        let data = create_test_weather_data();

        cache.set(data, "Tokyo".to_string()).await;

        // キャッシュ直後は取得可能
        assert!(cache.get("Tokyo").await.is_some());

        // 1秒以上待機して期限切れを確認
        tokio::time::sleep(tokio::time::Duration::from_millis(1100)).await;

        // 期限切れでNoneを返す
        assert!(cache.get("Tokyo").await.is_none());

        // TTL remainingも0
        assert_eq!(cache.ttl_remaining().await, 0);
    }

    #[tokio::test]
    async fn test_ttl_consistency_between_get_and_ttl_remaining() {
        // TTLの整合性テスト：get()とttl_remaining()が同じTTL値を使用していることを確認
        let cache = WeatherCache::with_ttl(2);
        let data = create_test_weather_data();

        cache.set(data, "Tokyo".to_string()).await;

        // 1.5秒待機（TTLの半分以上経過）
        tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;

        // まだ有効（2秒未満）
        assert!(cache.get("Tokyo").await.is_some());
        assert!(cache.ttl_remaining().await > 0);

        // さらに1秒待機（合計2.5秒経過）
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

        // 期限切れ
        assert!(cache.get("Tokyo").await.is_none());
        assert_eq!(cache.ttl_remaining().await, 0);
    }

    #[tokio::test]
    async fn test_cache_city_mismatch() {
        // 都市名が異なる場合はキャッシュミスになることを確認
        let cache = WeatherCache::new();
        let data = create_test_weather_data();

        // Tokyoでキャッシュ
        cache.set(data, "Tokyo".to_string()).await;

        // Tokyoでは取得可能
        assert!(cache.get("Tokyo").await.is_some());

        // Osakaでは取得不可（都市不一致）
        assert!(cache.get("Osaka").await.is_none());
    }

    #[tokio::test]
    async fn test_cache_city_change_invalidates_old_city() {
        // 都市変更後に古い都市のキャッシュが無効になることを確認
        let cache = WeatherCache::new();

        let tokyo_data = WeatherData {
            icon: "☀️".to_string(),
            temp: 25.5,
            description: "晴天".to_string(),
            location: "Tokyo".to_string(),
            humidity: 60,
            weather_code: 800,
            fetched_at: chrono::Utc::now().timestamp(),
        };

        let osaka_data = WeatherData {
            icon: "☁️".to_string(),
            temp: 22.0,
            description: "曇り".to_string(),
            location: "Osaka".to_string(),
            humidity: 70,
            weather_code: 803,
            fetched_at: chrono::Utc::now().timestamp(),
        };

        // Tokyoでキャッシュ
        cache.set(tokyo_data.clone(), "Tokyo".to_string()).await;
        assert!(cache.get("Tokyo").await.is_some());

        // Osakaで上書き
        cache.set(osaka_data.clone(), "Osaka".to_string()).await;

        // Tokyoは取得不可（上書きされた）
        assert!(cache.get("Tokyo").await.is_none());

        // Osakaは取得可能
        let cached = cache.get("Osaka").await;
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().location, "Osaka");
    }
}
