use rand::Rng;
use std::time::Duration;

/// 指数バックオフの最大試行回数
/// u32::MAXで事実上無制限に設定（長時間配信対応）
/// ユーザーは手動でポーリングを停止可能
const MAX_ATTEMPTS: u32 = u32::MAX;

/// ジッタの最大割合（±25%）
const JITTER_FACTOR: f64 = 0.25;

/// 指数バックオフを管理する構造体
/// エラー時のリトライ間隔を指数的に増加させる（1s→2s→4s→8s→16s...）
/// ジッタを追加して複数クライアントの再接続衝突を防止
pub struct ExponentialBackoff {
    base_delay: Duration,
    max_delay: Duration,
    max_attempts: u32,
    current_attempt: u32,
    use_jitter: bool,
}

impl ExponentialBackoff {
    /// 新しいExponentialBackoffインスタンスを作成
    ///
    /// デフォルト設定:
    /// - base_delay: 1秒
    /// - max_delay: 60秒
    /// - max_attempts: 無制限（u32::MAX）
    /// - use_jitter: false（互換性のため）
    pub fn new() -> Self {
        Self {
            base_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            max_attempts: MAX_ATTEMPTS,
            current_attempt: 0,
            use_jitter: false,
        }
    }

    /// ジッタ付きでExponentialBackoffインスタンスを作成
    ///
    /// gRPCストリーミングなど、複数クライアントの再接続衝突を
    /// 避けたい場合に使用
    pub fn with_jitter() -> Self {
        Self {
            base_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            max_attempts: MAX_ATTEMPTS,
            current_attempt: 0,
            use_jitter: true,
        }
    }

    /// カスタム設定でExponentialBackoffインスタンスを作成
    #[allow(dead_code)]
    pub fn with_config(base_delay: Duration, max_delay: Duration, max_attempts: u32) -> Self {
        Self {
            base_delay,
            max_delay,
            max_attempts,
            current_attempt: 0,
            use_jitter: false,
        }
    }

    /// 次のリトライまでの待機時間を計算して返す
    ///
    /// 計算式: base_delay * 2^current_attempt (± jitter)
    /// max_delayを超える場合はmax_delayが返される
    pub fn next_delay(&mut self) -> Duration {
        // saturating_powでオーバーフローを防止
        let multiplier = 2u32.saturating_pow(self.current_attempt);
        let delay = self.base_delay.saturating_mul(multiplier);
        self.current_attempt = self.current_attempt.saturating_add(1);
        let capped_delay = delay.min(self.max_delay);

        if self.use_jitter {
            self.apply_jitter(capped_delay)
        } else {
            capped_delay
        }
    }

    /// 遅延にジッタを適用（±JITTER_FACTOR）
    fn apply_jitter(&self, delay: Duration) -> Duration {
        let delay_ms = delay.as_millis() as f64;
        let jitter_range = delay_ms * JITTER_FACTOR;
        let mut rng = rand::thread_rng();
        let jitter = rng.gen_range(-jitter_range..=jitter_range);
        let jittered_ms = (delay_ms + jitter).max(0.0) as u64;
        Duration::from_millis(jittered_ms)
    }

    /// バックオフカウンターをリセット（成功時に呼び出す）
    pub fn reset(&mut self) {
        self.current_attempt = 0;
    }

    /// 現在の試行回数を取得
    #[allow(dead_code)]
    pub fn attempt_count(&self) -> u32 {
        self.current_attempt
    }

    /// 最大試行回数に達したかどうかを確認
    pub fn has_exceeded_max_attempts(&self) -> bool {
        self.current_attempt >= self.max_attempts
    }

    /// リトライを続行すべきかどうかを確認
    pub fn should_retry(&self) -> bool {
        !self.has_exceeded_max_attempts()
    }
}

impl Default for ExponentialBackoff {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backoff_progression() {
        let mut backoff = ExponentialBackoff::new();

        assert_eq!(backoff.next_delay(), Duration::from_secs(1)); // 2^0 = 1
        assert_eq!(backoff.next_delay(), Duration::from_secs(2)); // 2^1 = 2
        assert_eq!(backoff.next_delay(), Duration::from_secs(4)); // 2^2 = 4
        assert_eq!(backoff.next_delay(), Duration::from_secs(8)); // 2^3 = 8
        assert_eq!(backoff.next_delay(), Duration::from_secs(16)); // 2^4 = 16
        assert_eq!(backoff.next_delay(), Duration::from_secs(32)); // 2^5 = 32
        assert_eq!(backoff.next_delay(), Duration::from_secs(60)); // 2^6 = 64 -> max 60
    }

    #[test]
    fn test_backoff_reset() {
        let mut backoff = ExponentialBackoff::new();

        assert_eq!(backoff.next_delay(), Duration::from_secs(1));
        assert_eq!(backoff.next_delay(), Duration::from_secs(2));

        backoff.reset();

        assert_eq!(backoff.next_delay(), Duration::from_secs(1));
    }

    #[test]
    fn test_custom_config() {
        let mut backoff = ExponentialBackoff::with_config(
            Duration::from_millis(500),
            Duration::from_secs(10),
            5,
        );

        assert_eq!(backoff.next_delay(), Duration::from_millis(500));
        assert_eq!(backoff.next_delay(), Duration::from_secs(1));
        assert_eq!(backoff.next_delay(), Duration::from_secs(2));
    }

    #[test]
    fn test_max_attempts() {
        let mut backoff = ExponentialBackoff::with_config(
            Duration::from_secs(1),
            Duration::from_secs(60),
            3,
        );

        assert!(backoff.should_retry());
        backoff.next_delay();
        assert!(backoff.should_retry());
        backoff.next_delay();
        assert!(backoff.should_retry());
        backoff.next_delay();
        assert!(!backoff.should_retry());
        assert!(backoff.has_exceeded_max_attempts());
    }

    #[test]
    fn test_reset_attempts() {
        let mut backoff = ExponentialBackoff::with_config(
            Duration::from_secs(1),
            Duration::from_secs(60),
            3,
        );

        backoff.next_delay();
        backoff.next_delay();
        backoff.next_delay();
        assert!(!backoff.should_retry());

        backoff.reset();
        assert!(backoff.should_retry());
    }

    #[test]
    fn test_jitter_within_bounds() {
        let mut backoff = ExponentialBackoff::with_jitter();

        // Run multiple times to test randomness
        for _ in 0..10 {
            backoff.reset();
            let delay = backoff.next_delay();
            // Base delay is 1s, jitter is ±25%, so 750ms to 1250ms
            assert!(delay >= Duration::from_millis(750));
            assert!(delay <= Duration::from_millis(1250));
        }
    }

    #[test]
    fn test_jitter_progression() {
        let mut backoff = ExponentialBackoff::with_jitter();

        let delay1 = backoff.next_delay();
        let delay2 = backoff.next_delay();
        let delay3 = backoff.next_delay();

        // Each delay should roughly double (within jitter bounds)
        // delay1 ≈ 1s (750-1250ms)
        // delay2 ≈ 2s (1500-2500ms)
        // delay3 ≈ 4s (3000-5000ms)
        assert!(delay1 < delay2);
        assert!(delay2 < delay3);
    }
}
