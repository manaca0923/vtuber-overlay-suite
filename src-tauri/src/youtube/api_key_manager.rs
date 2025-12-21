//! APIキー管理モジュール
//!
//! 同梱キー（Primary/Secondary）とBYOK（ユーザー提供キー）の両方をサポートする。
//! - 同梱キー: ビルド時に環境変数から注入
//! - BYOK: ユーザーがUI経由で設定
//!
//! キーの優先順位:
//! 1. BYOKが設定されていて、use_bundled=false の場合 → BYOK
//! 2. それ以外 → Primary → Secondary（フォールバック）

use std::sync::atomic::{AtomicBool, Ordering};

/// 環境変数から同梱キーを取得（ビルド時に設定）
/// 未設定の場合は空文字列として扱う
const BUNDLED_PRIMARY_KEY: Option<&str> = option_env!("YOUTUBE_API_KEY_PRIMARY");
const BUNDLED_SECONDARY_KEY: Option<&str> = option_env!("YOUTUBE_API_KEY_SECONDARY");

/// APIキー管理構造体
#[derive(Debug)]
pub struct ApiKeyManager {
    /// ユーザー提供キー（BYOK）
    user_key: Option<String>,
    /// Primaryキーが失敗してSecondaryにフォールバック中かどうか
    using_secondary: AtomicBool,
}

impl Default for ApiKeyManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ApiKeyManager {
    /// 新しいApiKeyManagerを作成
    pub fn new() -> Self {
        Self {
            user_key: None,
            using_secondary: AtomicBool::new(false),
        }
    }

    /// 同梱キーが利用可能かどうか
    pub fn has_bundled_key(&self) -> bool {
        BUNDLED_PRIMARY_KEY.is_some() && !BUNDLED_PRIMARY_KEY.unwrap().is_empty()
    }

    /// BYOKが設定されているかどうか
    pub fn has_user_key(&self) -> bool {
        self.user_key.is_some() && !self.user_key.as_ref().unwrap().is_empty()
    }

    /// 有効なキーを取得
    ///
    /// # Arguments
    /// * `prefer_bundled` - true: 同梱キーを優先、false: BYOKを優先
    ///
    /// # Returns
    /// * `Some(&str)` - 有効なAPIキー
    /// * `None` - 利用可能なキーがない
    pub fn get_active_key(&self, prefer_bundled: bool) -> Option<&str> {
        if prefer_bundled {
            // 同梱キー優先
            self.get_bundled_key().or(self.user_key.as_deref())
        } else {
            // BYOK優先
            self.user_key
                .as_deref()
                .filter(|k| !k.is_empty())
                .or_else(|| self.get_bundled_key())
        }
    }

    /// 同梱キーを取得（Primary → Secondaryのフォールバック）
    fn get_bundled_key(&self) -> Option<&str> {
        if self.using_secondary.load(Ordering::SeqCst) {
            // Secondaryを使用中
            BUNDLED_SECONDARY_KEY.filter(|k| !k.is_empty())
        } else {
            // Primaryを使用（なければSecondary）
            BUNDLED_PRIMARY_KEY
                .filter(|k| !k.is_empty())
                .or_else(|| BUNDLED_SECONDARY_KEY.filter(|k| !k.is_empty()))
        }
    }

    /// BYOKを設定
    pub fn set_user_key(&mut self, key: Option<String>) {
        self.user_key = key.filter(|k| !k.is_empty());
    }

    /// Secondaryキーにフォールバック
    ///
    /// Primaryキーでエラーが発生した場合に呼び出す。
    /// Secondaryキーが存在しない場合は何もしない。
    pub fn switch_to_secondary(&self) {
        if BUNDLED_SECONDARY_KEY.is_some() {
            log::warn!("Switching to secondary API key due to primary key failure");
            self.using_secondary.store(true, Ordering::SeqCst);
        }
    }

    /// Primaryキーに戻す
    pub fn reset_to_primary(&self) {
        if self.using_secondary.load(Ordering::SeqCst) {
            log::info!("Resetting to primary API key");
            self.using_secondary.store(false, Ordering::SeqCst);
        }
    }

    /// 現在Secondaryを使用中かどうか
    pub fn is_using_secondary(&self) -> bool {
        self.using_secondary.load(Ordering::SeqCst)
    }

    /// キー状態のサマリーを取得（デバッグ/ログ用）
    pub fn status_summary(&self) -> String {
        let bundled_status = if self.has_bundled_key() {
            if self.is_using_secondary() {
                "bundled(secondary)"
            } else {
                "bundled(primary)"
            }
        } else {
            "no bundled key"
        };

        let user_status = if self.has_user_key() {
            "BYOK set"
        } else {
            "no BYOK"
        };

        format!("{}, {}", bundled_status, user_status)
    }
}

/// グローバルなApiKeyManagerインスタンス
static API_KEY_MANAGER: std::sync::OnceLock<std::sync::RwLock<ApiKeyManager>> =
    std::sync::OnceLock::new();

/// グローバルなApiKeyManagerを取得
pub fn get_api_key_manager() -> &'static std::sync::RwLock<ApiKeyManager> {
    API_KEY_MANAGER.get_or_init(|| std::sync::RwLock::new(ApiKeyManager::new()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_manager() {
        let manager = ApiKeyManager::new();
        assert!(!manager.has_user_key());
        assert!(!manager.is_using_secondary());
    }

    #[test]
    fn test_set_user_key() {
        let mut manager = ApiKeyManager::new();

        // 空文字列は設定されない
        manager.set_user_key(Some("".to_string()));
        assert!(!manager.has_user_key());

        // 有効なキーは設定される
        manager.set_user_key(Some("test-key".to_string()));
        assert!(manager.has_user_key());

        // Noneで解除
        manager.set_user_key(None);
        assert!(!manager.has_user_key());
    }

    #[test]
    fn test_secondary_fallback() {
        let manager = ApiKeyManager::new();

        assert!(!manager.is_using_secondary());

        manager.switch_to_secondary();
        // 同梱Secondaryキーがない場合は切り替わらない（テスト環境では通常ない）

        manager.reset_to_primary();
        assert!(!manager.is_using_secondary());
    }
}
