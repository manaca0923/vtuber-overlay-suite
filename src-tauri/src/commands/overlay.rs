use serde::{Deserialize, Serialize};

use crate::AppState;

/// 共通設定
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommonSettings {
    pub primary_color: String,
    pub font_family: String,
    pub border_radius: u32,
}

/// コメントオーバーレイ設定
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentSettings {
    pub enabled: bool,
    pub position: String,
    pub max_count: u32,
    pub show_avatar: bool,
    pub font_size: u32,
}

/// セットリストオーバーレイ設定
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetlistSettings {
    pub enabled: bool,
    pub position: String,
    pub show_artist: bool,
    pub font_size: u32,
}

/// オーバーレイ設定全体
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OverlaySettings {
    pub theme: String,
    pub common: CommonSettings,
    pub comment: CommentSettings,
    pub setlist: SetlistSettings,
}

/// オーバーレイ設定を保存
#[tauri::command]
pub async fn save_overlay_settings(
    settings: OverlaySettings,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let pool = &state.db;
    let now = chrono::Utc::now().to_rfc3339();

    let settings_str =
        serde_json::to_string(&settings).map_err(|e| format!("JSON serialize error: {}", e))?;

    sqlx::query(
        r#"
        INSERT INTO settings (key, value, updated_at)
        VALUES ('overlay_settings', ?, ?)
        ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at
        "#,
    )
    .bind(&settings_str)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    log::info!("Overlay settings saved");
    Ok(())
}

/// オーバーレイ設定を読み込み
#[tauri::command]
pub async fn load_overlay_settings(
    state: tauri::State<'_, AppState>,
) -> Result<Option<OverlaySettings>, String> {
    let pool = &state.db;

    let result: Option<(String,)> =
        sqlx::query_as("SELECT value FROM settings WHERE key = 'overlay_settings'")
            .fetch_optional(pool)
            .await
            .map_err(|e| format!("DB error: {}", e))?;

    if let Some((json_str,)) = result {
        let settings: OverlaySettings =
            serde_json::from_str(&json_str).map_err(|e| format!("JSON parse error: {}", e))?;
        Ok(Some(settings))
    } else {
        Ok(None)
    }
}

/// 設定変更をWebSocketでブロードキャスト
#[tauri::command]
pub async fn broadcast_settings_update(
    settings: OverlaySettings,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    use crate::server::types::{SettingsUpdatePayload, WsMessage};

    let payload = SettingsUpdatePayload {
        theme: settings.theme.clone(),
        primary_color: settings.common.primary_color.clone(),
        position: settings.comment.position.clone(),
        visible: settings.comment.enabled,
    };

    let server_state = state.server.read().await;
    server_state
        .broadcast(WsMessage::SettingsUpdate { payload })
        .await;

    log::info!("Settings update broadcasted");
    Ok(())
}
