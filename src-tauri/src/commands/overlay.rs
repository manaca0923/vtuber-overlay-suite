use serde::{Deserialize, Serialize};

use crate::server::types::{CommentPosition, LayoutPreset, SetlistPosition};
use crate::AppState;

/// HEXカラーコードのバリデーション (#RRGGBB形式)
fn is_valid_hex_color(color: &str) -> bool {
    color.len() == 7
        && color.starts_with('#')
        && color[1..].chars().all(|c| c.is_ascii_hexdigit())
}

/// オーバーレイ設定のバリデーション
fn validate_overlay_settings(settings: &OverlaySettings) -> Result<(), String> {
    // primaryColorの検証
    if !is_valid_hex_color(&settings.common.primary_color) {
        return Err(format!(
            "Invalid primaryColor: {}. Expected #RRGGBB format.",
            settings.common.primary_color
        ));
    }

    // borderRadiusの検証 (0-32)
    if settings.common.border_radius > 32 {
        return Err(format!(
            "Invalid borderRadius: {}. Expected 0-32.",
            settings.common.border_radius
        ));
    }

    // コメント設定の検証
    // NOTE: maxCountは画面高さベースの自動調整に統一したため削除
    if settings.comment.font_size < 8 || settings.comment.font_size > 72 {
        return Err(format!(
            "Invalid comment fontSize: {}. Expected 8-72.",
            settings.comment.font_size
        ));
    }

    // セットリスト設定の検証
    if settings.setlist.font_size < 8 || settings.setlist.font_size > 72 {
        return Err(format!(
            "Invalid setlist fontSize: {}. Expected 8-72.",
            settings.setlist.font_size
        ));
    }

    Ok(())
}

/// 共通設定
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommonSettings {
    pub primary_color: String,
    pub font_family: String,
    pub border_radius: u32,
}

/// コメントオーバーレイ設定
/// NOTE: maxCountは画面高さベースの自動調整に統一したため削除
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentSettings {
    pub enabled: bool,
    pub position: CommentPosition,
    pub show_avatar: bool,
    pub font_size: u32,
}

/// セットリストオーバーレイ設定
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetlistSettings {
    pub enabled: bool,
    pub position: SetlistPosition,
    pub show_artist: bool,
    pub font_size: u32,
}

/// オーバーレイ設定全体
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OverlaySettings {
    pub theme: String,
    pub layout: LayoutPreset,
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
    // サーバーサイドバリデーション
    validate_overlay_settings(&settings)?;

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
    // サーバーサイドバリデーション
    validate_overlay_settings(&settings)?;

    use crate::server::types::{
        CommentSettingsPayload, SetlistSettingsPayload, SettingsUpdatePayload, WsMessage,
    };

    let payload = SettingsUpdatePayload {
        theme: settings.theme.clone(),
        layout: settings.layout,
        primary_color: settings.common.primary_color.clone(),
        font_family: settings.common.font_family.clone(),
        border_radius: settings.common.border_radius,
        comment: CommentSettingsPayload {
            enabled: settings.comment.enabled,
            position: settings.comment.position, // Copy trait実装済みのため.clone()不要
            show_avatar: settings.comment.show_avatar,
            font_size: settings.comment.font_size,
        },
        setlist: SetlistSettingsPayload {
            enabled: settings.setlist.enabled,
            position: settings.setlist.position, // Copy trait実装済みのため.clone()不要
            show_artist: settings.setlist.show_artist,
            font_size: settings.setlist.font_size,
        },
    };

    let server_state = state.server.read().await;
    server_state
        .broadcast(WsMessage::SettingsUpdate { payload })
        .await;

    log::info!("Settings update broadcasted");
    Ok(())
}
