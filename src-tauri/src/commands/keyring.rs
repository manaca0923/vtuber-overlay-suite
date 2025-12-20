use crate::AppState;
use sqlx::Row;

// =============================================================================
// TODO: 本番リリース前の対応事項
// =============================================================================
// 現在の実装ではAPIキーをSQLiteに平文で保存しています。
// 本番リリース前に以下の対応が必要です：
//
// 1. src-tauri/src/keyring.rs のkeyringクレートを使用した実装に移行
//    - macOS: Keychain
//    - Windows: Credential Manager
//    - Linux: Secret Service API
//
// 2. この実装（DB保存）を削除するか、keyringが使用できない環境用の
//    フォールバックとして残す場合は暗号化を検討
//
// 参照: CLAUDE.md の「セキュリティ」セクション
// =============================================================================

/// APIキーをDBに保存
///
/// ⚠️ 注意: 開発用の実装です。本番ではkeyringクレートを使用してください。
/// DBファイルは平文で保存されるため、セキュリティリスクがあります。
#[tauri::command]
pub async fn save_api_key(api_key: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let pool = &state.db;
    let now = chrono::Utc::now().to_rfc3339();
    
    sqlx::query(
        r#"
        INSERT INTO settings (key, value, updated_at)
        VALUES ('api_key', ?, ?)
        ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at
        "#
    )
    .bind(&api_key)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(|e| format!("DB error: {}", e))?;
    
    log::info!("API key saved to database");
    Ok(())
}

/// APIキーをDBから取得
#[tauri::command]
pub async fn get_api_key(state: tauri::State<'_, AppState>) -> Result<Option<String>, String> {
    let pool = &state.db;
    
    let result = sqlx::query("SELECT value FROM settings WHERE key = 'api_key'")
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("DB error: {}", e))?;
    
    if let Some(row) = result {
        let value: String = row.get("value");
        log::info!("API key retrieved from database");
        Ok(Some(value))
    } else {
        Ok(None)
    }
}

/// APIキーをDBから削除
#[tauri::command]
pub async fn delete_api_key(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let pool = &state.db;
    
    sqlx::query("DELETE FROM settings WHERE key = 'api_key'")
        .execute(pool)
        .await
        .map_err(|e| format!("DB error: {}", e))?;
    
    log::info!("API key deleted from database");
    Ok(())
}

/// APIキーが保存されているかチェック
#[tauri::command]
pub async fn has_api_key(state: tauri::State<'_, AppState>) -> Result<bool, String> {
    let pool = &state.db;
    
    let result = sqlx::query("SELECT value FROM settings WHERE key = 'api_key'")
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("DB error: {}", e))?;
    
    let has_key = result.is_some();
    log::info!("has_api_key check: {}", has_key);
    Ok(has_key)
}
