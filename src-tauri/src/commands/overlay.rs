use serde::{Deserialize, Serialize};

use crate::server::types::{
    CommentSettings, LayoutPreset, SetlistSettings, SettingsUpdatePayload, SuperchatSettings,
    ThemeSettings, WeatherSettings, WidgetVisibilitySettings, WsMessage,
};
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

    // スパチャ設定の検証
    if let Some(ref superchat) = settings.superchat {
        if superchat.max_display < 1 || superchat.max_display > 3 {
            return Err(format!(
                "Invalid superchat maxDisplay: {}. Expected 1-3.",
                superchat.max_display
            ));
        }
        if superchat.display_duration_sec < 10 || superchat.display_duration_sec > 120 {
            return Err(format!(
                "Invalid superchat displayDurationSec: {}. Expected 10-120.",
                superchat.display_duration_sec
            ));
        }
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

/// オーバーレイ設定全体
/// NOTE: WidgetVisibilitySettingsはcrate::server::typesから再利用
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OverlaySettings {
    pub theme: String,
    pub layout: LayoutPreset,
    pub common: CommonSettings,
    pub comment: CommentSettings,
    pub setlist: SetlistSettings,
    #[serde(default)]
    pub weather: Option<WeatherSettings>,
    #[serde(default)]
    pub widget: Option<WidgetVisibilitySettings>,
    #[serde(default)]
    pub superchat: Option<SuperchatSettings>,
    /// テーマ設定（カラー・フォント統合）
    #[serde(default)]
    pub theme_settings: Option<ThemeSettings>,
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

    // 型が統一されたため、直接設定を渡せる
    let payload = SettingsUpdatePayload {
        theme: settings.theme.clone(),
        layout: settings.layout,
        primary_color: settings.common.primary_color.clone(),
        font_family: settings.common.font_family.clone(),
        border_radius: settings.common.border_radius,
        comment: settings.comment,
        setlist: settings.setlist,
        weather: settings.weather,
        widget: settings.widget,
        superchat: settings.superchat,
        theme_settings: settings.theme_settings,
    };

    let server_state = state.server.read().await;
    server_state
        .broadcast(WsMessage::SettingsUpdate { payload })
        .await;

    log::info!("Settings update broadcasted");
    Ok(())
}
