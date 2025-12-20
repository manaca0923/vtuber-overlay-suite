use crate::keyring;

/// APIキーをセキュアストレージに保存
#[tauri::command]
pub async fn save_api_key(api_key: String) -> Result<(), String> {
    keyring::save_api_key(&api_key).map_err(|e| e.to_string())
}

/// APIキーをセキュアストレージから取得
#[tauri::command]
pub async fn get_api_key() -> Result<Option<String>, String> {
    match keyring::get_api_key() {
        Ok(key) => Ok(Some(key)),
        Err(keyring::KeyringError::NotFound) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

/// APIキーをセキュアストレージから削除
#[tauri::command]
pub async fn delete_api_key() -> Result<(), String> {
    keyring::delete_api_key().map_err(|e| e.to_string())
}

/// APIキーが保存されているかチェック
#[tauri::command]
pub async fn has_api_key() -> Result<bool, String> {
    keyring::has_api_key().map_err(|e| e.to_string())
}
