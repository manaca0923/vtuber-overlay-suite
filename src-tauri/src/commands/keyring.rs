use crate::keyring as secure_storage;
use crate::AppState;
use sqlx::Row;

// =============================================================================
// セキュアストレージ実装（本番用）
// =============================================================================
// OSのセキュアストレージ（Keychain, Credential Manager等）を使用して
// APIキーを安全に保存します。
//
// 既存のDB保存からの自動移行機能付き：
// - get_api_key時にDBにあってkeyringに無い場合は自動でkeyringに移行
// - 移行後はDBから削除
// =============================================================================

/// APIキーをセキュアストレージに保存
#[tauri::command]
pub async fn save_api_key(api_key: String, _state: tauri::State<'_, AppState>) -> Result<(), String> {
    // 空文字列のバリデーション
    if api_key.trim().is_empty() {
        return Err("API key cannot be empty".to_string());
    }

    // keyringはブロッキング呼び出しなのでspawn_blockingを使用
    tokio::task::spawn_blocking(move || {
        secure_storage::save_api_key(&api_key)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
    .map_err(|e| format!("Keyring error: {}", e))
}

/// APIキーをセキュアストレージから取得
///
/// DBに保存されている場合は自動でセキュアストレージに移行
#[tauri::command]
pub async fn get_api_key(state: tauri::State<'_, AppState>) -> Result<Option<String>, String> {
    // まずkeyringから取得を試みる
    let keyring_result = tokio::task::spawn_blocking(|| {
        secure_storage::get_api_key()
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?;

    match keyring_result {
        Ok(api_key) => {
            // keyringにあればそのまま返す
            Ok(Some(api_key))
        }
        Err(secure_storage::KeyringError::NotFound) => {
            // keyringに無い場合、DBからの移行を試みる
            migrate_from_db_if_exists(state).await
        }
        Err(e) => Err(format!("Keyring error: {}", e)),
    }
}

/// APIキーをセキュアストレージから削除
#[tauri::command]
pub async fn delete_api_key(state: tauri::State<'_, AppState>) -> Result<(), String> {
    // keyringから削除
    tokio::task::spawn_blocking(|| {
        secure_storage::delete_api_key()
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
    .map_err(|e| format!("Keyring error: {}", e))?;

    // DBからも削除（移行残りがあれば）
    let pool = &state.db;
    sqlx::query("DELETE FROM settings WHERE key = 'api_key'")
        .execute(pool)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    Ok(())
}

/// APIキーが保存されているかチェック
#[tauri::command]
pub async fn has_api_key(state: tauri::State<'_, AppState>) -> Result<bool, String> {
    // まずkeyringをチェック
    let keyring_result = tokio::task::spawn_blocking(|| {
        secure_storage::has_api_key()
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?;

    match keyring_result {
        Ok(true) => Ok(true),
        Ok(false) => {
            // keyringに無い場合、DBにあるかチェック（移行対象）
            let pool = &state.db;
            let result = sqlx::query("SELECT value FROM settings WHERE key = 'api_key'")
                .fetch_optional(pool)
                .await
                .map_err(|e| format!("DB error: {}", e))?;
            Ok(result.is_some())
        }
        Err(e) => Err(format!("Keyring error: {}", e)),
    }
}

// =============================================================================
// 移行ヘルパー関数
// =============================================================================

/// DBにAPIキーがあればkeyringに移行して返す
async fn migrate_from_db_if_exists(state: tauri::State<'_, AppState>) -> Result<Option<String>, String> {
    let pool = &state.db;

    // DBから取得
    let result = sqlx::query("SELECT value FROM settings WHERE key = 'api_key'")
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    if let Some(row) = result {
        let api_key: String = row.get("value");

        // keyringに移行
        let api_key_clone = api_key.clone();
        tokio::task::spawn_blocking(move || {
            secure_storage::save_api_key(&api_key_clone)
        })
        .await
        .map_err(|e| format!("Task join error: {}", e))?
        .map_err(|e| format!("Keyring migration error: {}", e))?;

        // DBから削除
        sqlx::query("DELETE FROM settings WHERE key = 'api_key'")
            .execute(pool)
            .await
            .map_err(|e| format!("DB cleanup error: {}", e))?;

        log::info!("API key migrated from DB to secure storage");
        Ok(Some(api_key))
    } else {
        Ok(None)
    }
}
