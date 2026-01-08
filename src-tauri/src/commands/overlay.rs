use serde::{Deserialize, Serialize};
use std::sync::Arc;

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

    // テーマ設定の検証
    if let Some(ref theme) = settings.theme_settings {
        // カスタムカラーの上限チェック（最大3件）
        const MAX_CUSTOM_COLORS: usize = 3;
        if theme.custom_colors.len() > MAX_CUSTOM_COLORS {
            return Err(format!(
                "Too many custom colors: {}. Maximum is {}.",
                theme.custom_colors.len(),
                MAX_CUSTOM_COLORS
            ));
        }

        // グローバルプライマリカラーの検証
        if !is_valid_hex_color(&theme.global_primary_color) {
            return Err(format!(
                "Invalid globalPrimaryColor: {}. Expected #RRGGBB format.",
                theme.global_primary_color
            ));
        }

        // カスタムカラーの各色を検証
        for (i, color_entry) in theme.custom_colors.iter().enumerate() {
            if !is_valid_hex_color(&color_entry.color) {
                return Err(format!(
                    "Invalid custom color at index {}: {}. Expected #RRGGBB format.",
                    i, color_entry.color
                ));
            }
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
///
/// ## JSON破損時のフォールバック
/// 保存されているJSONが破損している場合:
/// 1. 破損データを`overlay_settings_backup_{timestamp}`キーに退避保存
/// 2. 破損した`overlay_settings`キーを削除（次回以降のフォールバックを防止）
/// 3. Noneを返却（フロントエンドがデフォルト設定を使用）
/// 4. 警告ログを出力
///
/// これにより、UIが復旧不能な状態になることを防止する。
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
        match serde_json::from_str::<OverlaySettings>(&json_str) {
            Ok(settings) => Ok(Some(settings)),
            Err(e) => {
                // JSON破損時: 破損データを退避してNoneを返す
                log::warn!(
                    "Overlay settings JSON corrupted, falling back to None. Error: {}",
                    e
                );

                // 破損データをバックアップキーに退避（復旧調査用）
                // バックアップ成功時のみ元キーを削除（データ損失防止）
                let now = chrono::Utc::now().to_rfc3339();
                let backup_result = sqlx::query(
                    r#"
                    INSERT INTO settings (key, value, updated_at)
                    VALUES (?, ?, ?)
                    ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at
                    "#,
                )
                .bind(format!("overlay_settings_backup_{}", now))
                .bind(&json_str)
                .bind(&now)
                .execute(pool)
                .await;

                match backup_result {
                    Ok(_) => {
                        log::info!("Corrupted overlay settings backed up successfully");
                        // バックアップ成功時のみ破損キーを削除
                        if let Err(delete_err) =
                            sqlx::query("DELETE FROM settings WHERE key = 'overlay_settings'")
                                .execute(pool)
                                .await
                        {
                            log::error!(
                                "Failed to delete corrupted overlay settings: {}",
                                delete_err
                            );
                        } else {
                            log::info!(
                                "Deleted corrupted overlay_settings key to prevent repeated fallback"
                            );
                        }
                    }
                    Err(backup_err) => {
                        // バックアップ失敗時は元キーを保持（データ損失防止）
                        log::error!(
                            "Failed to backup corrupted overlay settings, keeping original key: {}",
                            backup_err
                        );
                    }
                }

                Ok(None)
            }
        }
    } else {
        Ok(None)
    }
}

/// 設定変更をWebSocketでブロードキャスト
///
/// ## 設計ノート
/// - Fire-and-forgetパターン: ブロードキャストは`tokio::spawn`でバックグラウンド実行
/// - 呼び出し元はブロードキャスト完了を待たずに即座に`Ok(())`を返す
/// - ブロードキャスト失敗はログ出力のみで、呼び出し元のコマンド成功には影響しない
/// - RwLockガードをawait境界をまたいで保持しないようにtokio::spawnで分離
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

    // WebSocketでブロードキャスト（Fire-and-forget）
    //
    // ## 設計根拠
    // - `tokio::spawn`で独立したタスクとして実行
    // - RwLockガードをawait境界をまたいで保持しないため、2段階で処理:
    //   1. serverのガードを取得→peersのArcをクローン→ガード解放
    //   2. ガード解放後にpeersのRwLockをawait
    // - これにより「ガード保持中にawait」を完全に回避
    let server = Arc::clone(&state.server);
    let message = WsMessage::SettingsUpdate { payload };
    tokio::spawn(async move {
        // ステップ1: serverのガードを取得してpeersのArcをクローン、即座にガード解放
        let peers_arc = {
            let ws_state = server.read().await;
            ws_state.get_peers_arc()
        }; // ここでws_stateのガード解放

        // ステップ2: ガード解放後にpeersをawait（ガード保持中にawaitしていない）
        let peers_guard = peers_arc.read().await;
        let peers: Vec<_> = peers_guard
            .iter()
            .map(|(id, tx)| (*id, tx.clone()))
            .collect();
        drop(peers_guard); // 明示的にガード解放

        // ステップ3: ガード解放後に送信（awaitなし）
        crate::server::websocket::WebSocketState::send_to_peers(&peers, &message);
        log::debug!("Settings update broadcasted");
    });

    Ok(())
}
