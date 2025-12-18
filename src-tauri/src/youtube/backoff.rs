use std::time::Duration;

/// 指数バックオフの最大試行回数
/// これを超えるとリトライを停止する
const MAX_ATTEMPTS: u32 = 10;

/// 指数バックオフを管理する構造体
/// エラー時のリトライ間隔を指数的に増加させる（1s→2s→4s→8s→16s...）
pub struct ExponentialBackoff {
    base_delay: Duration,
    max_delay: Duration,
    max_attempts: u32,
    current_attempt: u32,
}

impl ExponentialBackoff {
    /// 新しいExponentialBackoffインスタンスを作成
    ///
    /// デフォルト設定:
    /// - base_delay: 1秒
    /// - max_delay: 60秒
    /// - max_attempts: 10回
    pub fn new() -> Self {
        Self {
            base_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            max_attempts: MAX_ATTEMPTS,
            current_attempt: 0,
        }
    }

    /// カスタム設定でExponentialBackoffインスタンスを作成
    pub fn with_config(base_delay: Duration, max_delay: Duration, max_attempts: u32) -> Self {
        Self {
            base_delay,
            max_delay,
            max_attempts,
            current_attempt: 0,
        }
    }

    /// 次のリトライまでの待機時間を計算して返す
    ///
    /// 計算式: base_delay * 2^current_attempt
    /// max_delayを超える場合はmax_delayが返される
    pub fn next_delay(&mut self) -> Duration {
        let delay = self.base_delay * 2u32.pow(self.current_attempt);
        self.current_attempt += 1;
        delay.min(self.max_delay)
    }

    /// バックオフカウンターをリセット（成功時に呼び出す）
    pub fn reset(&mut self) {
        self.current_attempt = 0;
    }

    /// 現在の試行回数を取得
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
}
