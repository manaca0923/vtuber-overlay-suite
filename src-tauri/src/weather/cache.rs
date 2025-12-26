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
    /// キャッシュ作成時刻
    created_at: Instant,
}

impl CacheEntry {
    fn new(data: WeatherData) -> Self {
        Self {
            data,
            created_at: Instant::now(),
        }
    }

    /// 指定されたTTLに対して期限切れかどうかを判定
    fn is_expired(&self, ttl_secs: u64) -> bool {
        self.created_at.elapsed() > Duration::from_secs(ttl_secs)
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
    /// キャッシュがない、または期限切れの場合はNoneを返す
    pub async fn get(&self) -> Option<WeatherData> {
        let entry = self.entry.read().await;
        match entry.as_ref() {
            Some(e) if !e.is_expired(self.ttl_secs) => {
                log::debug!("Weather cache hit");
                Some(e.data.clone())
            }
            Some(_) => {
                log::debug!("Weather cache expired");
                None
            }
            None => {
                log::debug!("Weather cache miss");
                None
            }
        }
    }

    /// キャッシュに天気データを保存
    pub async fn set(&self, data: WeatherData) {
        let mut entry = self.entry.write().await;
        *entry = Some(CacheEntry::new(data));
        log::debug!("Weather data cached (TTL: {}s)", self.ttl_secs);
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

        cache.set(data.clone()).await;

        let cached = cache.get().await;
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().location, "Tokyo");
    }

    #[tokio::test]
    async fn test_cache_miss() {
        let cache = WeatherCache::new();
        let cached = cache.get().await;
        assert!(cached.is_none());
    }

    #[tokio::test]
    async fn test_cache_clear() {
        let cache = WeatherCache::new();
        let data = create_test_weather_data();

        cache.set(data).await;
        assert!(cache.get().await.is_some());

        cache.clear().await;
        assert!(cache.get().await.is_none());
    }

    #[tokio::test]
    async fn test_cache_ttl_remaining() {
        let cache = WeatherCache::with_ttl(10);
        let data = create_test_weather_data();

        // キャッシュなしの場合は0
        assert_eq!(cache.ttl_remaining().await, 0);

        cache.set(data).await;

        // キャッシュ直後はTTLに近い値
        let ttl = cache.ttl_remaining().await;
        assert!(ttl > 0 && ttl <= 10);
    }

    #[tokio::test]
    async fn test_cache_expiry_with_short_ttl() {
        // 短いTTL（1秒）でテスト
        let cache = WeatherCache::with_ttl(1);
        let data = create_test_weather_data();

        cache.set(data).await;

        // キャッシュ直後は取得可能
        assert!(cache.get().await.is_some());

        // 1秒以上待機して期限切れを確認
        tokio::time::sleep(tokio::time::Duration::from_millis(1100)).await;

        // 期限切れでNoneを返す
        assert!(cache.get().await.is_none());

        // TTL remainingも0
        assert_eq!(cache.ttl_remaining().await, 0);
    }

    #[tokio::test]
    async fn test_ttl_consistency_between_get_and_ttl_remaining() {
        // TTLの整合性テスト：get()とttl_remaining()が同じTTL値を使用していることを確認
        let cache = WeatherCache::with_ttl(2);
        let data = create_test_weather_data();

        cache.set(data).await;

        // 1.5秒待機（TTLの半分以上経過）
        tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;

        // まだ有効（2秒未満）
        assert!(cache.get().await.is_some());
        assert!(cache.ttl_remaining().await > 0);

        // さらに1秒待機（合計2.5秒経過）
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

        // 期限切れ
        assert!(cache.get().await.is_none());
        assert_eq!(cache.ttl_remaining().await, 0);
    }
}
