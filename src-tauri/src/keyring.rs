use keyring::Entry;
use thiserror::Error;

/// アプリケーション識別子（サービス名として使用）
const SERVICE_NAME: &str = "com.vtuber-overlay-suite.desktop";

/// APIキー用のエントリ名
const API_KEY_ENTRY: &str = "youtube_api_key";

#[derive(Debug, Error)]
pub enum KeyringError {
    #[error("Keyring error: {0}")]
    KeyringError(#[from] keyring::Error),

    #[error("API key not found")]
    NotFound,
}

/// APIキーをOSのセキュアストレージに保存
///
/// - macOS: Keychain
/// - Windows: Credential Manager
/// - Linux: Secret Service API
pub fn save_api_key(api_key: &str) -> Result<(), KeyringError> {
    let entry = Entry::new(SERVICE_NAME, API_KEY_ENTRY)?;
    entry.set_password(api_key)?;
    log::info!("API key saved to secure storage");
    Ok(())
}

/// APIキーをセキュアストレージから取得
pub fn get_api_key() -> Result<String, KeyringError> {
    let entry = Entry::new(SERVICE_NAME, API_KEY_ENTRY)?;
    match entry.get_password() {
        Ok(password) => {
            log::debug!("API key retrieved from secure storage");
            Ok(password)
        }
        Err(keyring::Error::NoEntry) => Err(KeyringError::NotFound),
        Err(e) => Err(KeyringError::KeyringError(e)),
    }
}

/// APIキーをセキュアストレージから削除
pub fn delete_api_key() -> Result<(), KeyringError> {
    let entry = Entry::new(SERVICE_NAME, API_KEY_ENTRY)?;
    match entry.delete_credential() {
        Ok(()) => {
            log::info!("API key deleted from secure storage");
            Ok(())
        }
        Err(keyring::Error::NoEntry) => {
            log::warn!("Attempted to delete non-existent API key");
            Ok(()) // 既に存在しない場合も成功扱い
        }
        Err(e) => Err(KeyringError::KeyringError(e)),
    }
}

/// APIキーが保存されているかチェック
pub fn has_api_key() -> Result<bool, KeyringError> {
    match get_api_key() {
        Ok(_) => Ok(true),
        Err(KeyringError::NotFound) => Ok(false),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_and_get_api_key() {
        let test_key = "test_api_key_12345";

        // 保存
        save_api_key(test_key).unwrap();

        // 取得
        let retrieved = get_api_key().unwrap();
        assert_eq!(retrieved, test_key);

        // クリーンアップ
        delete_api_key().unwrap();
    }

    #[test]
    fn test_delete_api_key() {
        let test_key = "test_api_key_delete";

        // 保存
        save_api_key(test_key).unwrap();

        // 削除
        delete_api_key().unwrap();

        // 取得できないことを確認
        assert!(matches!(get_api_key(), Err(KeyringError::NotFound)));
    }

    #[test]
    fn test_has_api_key() {
        let test_key = "test_api_key_has";

        // 保存前はfalse
        delete_api_key().ok(); // 既存のものがあれば削除
        assert_eq!(has_api_key().unwrap(), false);

        // 保存
        save_api_key(test_key).unwrap();

        // 保存後はtrue
        assert_eq!(has_api_key().unwrap(), true);

        // クリーンアップ
        delete_api_key().unwrap();
    }
}
