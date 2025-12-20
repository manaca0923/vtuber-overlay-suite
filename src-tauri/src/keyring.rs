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

    /// テスト用の一意なエントリ名を生成
    fn get_test_entry_name(test_name: &str) -> String {
        format!("test_youtube_api_key_{}", test_name)
    }

    /// テスト用のAPIキー保存（一意なエントリ名を使用）
    fn save_test_api_key(test_name: &str, api_key: &str) -> Result<(), KeyringError> {
        let entry = Entry::new(SERVICE_NAME, &get_test_entry_name(test_name))?;
        entry.set_password(api_key)?;
        Ok(())
    }

    /// テスト用のAPIキー取得（一意なエントリ名を使用）
    fn get_test_api_key(test_name: &str) -> Result<String, KeyringError> {
        let entry = Entry::new(SERVICE_NAME, &get_test_entry_name(test_name))?;
        match entry.get_password() {
            Ok(password) => Ok(password),
            Err(keyring::Error::NoEntry) => Err(KeyringError::NotFound),
            Err(e) => Err(KeyringError::KeyringError(e)),
        }
    }

    /// テスト用のAPIキー削除（一意なエントリ名を使用）
    fn delete_test_api_key(test_name: &str) -> Result<(), KeyringError> {
        let entry = Entry::new(SERVICE_NAME, &get_test_entry_name(test_name))?;
        match entry.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(KeyringError::KeyringError(e)),
        }
    }

    #[test]
    #[ignore] // CI環境ではセキュアストレージが利用できない可能性があるため、手動実行時のみ
    fn test_save_and_get_api_key() {
        let test_name = "save_and_get";
        let test_key = "test_api_key_12345";

        // 保存
        save_test_api_key(test_name, test_key).unwrap();

        // 取得
        let retrieved = get_test_api_key(test_name).unwrap();
        assert_eq!(retrieved, test_key);

        // クリーンアップ
        delete_test_api_key(test_name).unwrap();
    }

    #[test]
    #[ignore] // CI環境ではセキュアストレージが利用できない可能性があるため、手動実行時のみ
    fn test_delete_api_key() {
        let test_name = "delete";
        let test_key = "test_api_key_delete";

        // 保存
        save_test_api_key(test_name, test_key).unwrap();

        // 削除
        delete_test_api_key(test_name).unwrap();

        // 取得できないことを確認
        assert!(matches!(
            get_test_api_key(test_name),
            Err(KeyringError::NotFound)
        ));
    }

    #[test]
    #[ignore] // CI環境ではセキュアストレージが利用できない可能性があるため、手動実行時のみ
    fn test_has_api_key() {
        let test_name = "has";
        let test_key = "test_api_key_has";

        // 保存前はfalse
        delete_test_api_key(test_name).ok(); // 既存のものがあれば削除
        assert!(matches!(
            get_test_api_key(test_name),
            Err(KeyringError::NotFound)
        ));

        // 保存
        save_test_api_key(test_name, test_key).unwrap();

        // 保存後は取得できる
        assert!(get_test_api_key(test_name).is_ok());

        // クリーンアップ
        delete_test_api_key(test_name).unwrap();
    }
}
