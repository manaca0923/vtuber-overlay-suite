// =============================================================================
// 共通設定・定数モジュール
// =============================================================================
// アプリケーション全体で使用する共通の設定値・定数を定義
// =============================================================================

use std::time::Duration;

/// HTTPリクエストのデフォルトタイムアウト（秒）
///
/// YouTube API、Weather API など外部APIへのリクエストで使用。
/// ネットワーク状況が悪い場合でも適切にタイムアウトし、
/// ユーザーを長時間待たせないようにする。
pub const HTTP_TIMEOUT_SECS: u64 = 10;

/// HTTPリクエストのデフォルトタイムアウト（Duration）
///
/// HTTPクライアント構築時に直接使用可能
pub fn http_timeout() -> Duration {
    Duration::from_secs(HTTP_TIMEOUT_SECS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_timeout_secs() {
        assert_eq!(HTTP_TIMEOUT_SECS, 10);
    }

    #[test]
    fn test_http_timeout_duration() {
        assert_eq!(http_timeout(), Duration::from_secs(10));
    }
}
