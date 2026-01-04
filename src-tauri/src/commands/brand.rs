//! ブランド（ロゴ）管理コマンド
//!
//! ロゴ画像URL/テキストの設定・取得、WebSocketブロードキャストを提供する。
//! データはDBのsettingsテーブルに保存される。

use crate::server::types::{BrandSettings, BrandUpdatePayload, WsMessage};
use crate::AppState;

/// ロゴURL最大長（バイト）
const MAX_LOGO_URL_LENGTH: usize = 2048;

/// テキスト最大長（文字）
const MAX_TEXT_LENGTH: usize = 100;

/// ブランド設定を取得
///
/// ## JSON破損時のフォールバック
/// 保存されているJSONが破損している場合はデフォルト状態（空）を返す。
#[tauri::command]
pub async fn get_brand_settings(state: tauri::State<'_, AppState>) -> Result<BrandSettings, String> {
    let pool = &state.db;

    let result: Option<(String,)> =
        sqlx::query_as("SELECT value FROM settings WHERE key = 'brand_settings'")
            .fetch_optional(pool)
            .await
            .map_err(|e| format!("DB error: {}", e))?;

    if let Some((json_str,)) = result {
        match serde_json::from_str::<BrandSettings>(&json_str) {
            Ok(settings) => Ok(settings),
            Err(e) => {
                log::warn!(
                    "Brand settings JSON corrupted, falling back to default. Error: {}",
                    e
                );
                Ok(BrandSettings::default())
            }
        }
    } else {
        Ok(BrandSettings::default())
    }
}

/// ブランド設定を保存
///
/// ## 入力検証
/// - `logo_url`: 最大2048バイト、http/https/dataスキームのみ許可
/// - `text`: 最大100文字
#[tauri::command]
pub async fn save_brand_settings(
    brand_settings: BrandSettings,
    state: tauri::State<'_, AppState>,
) -> Result<BrandSettings, String> {
    // 入力検証
    let validated = validate_brand_settings(brand_settings)?;

    let pool = &state.db;
    let now = chrono::Utc::now().to_rfc3339();

    let json_str = serde_json::to_string(&validated)
        .map_err(|e| format!("JSON serialize error: {}", e))?;

    sqlx::query(
        r#"
        INSERT INTO settings (key, value, updated_at)
        VALUES ('brand_settings', ?, ?)
        ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at
        "#,
    )
    .bind(&json_str)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    log::info!("Brand settings saved");
    Ok(validated)
}

/// ブランド更新をWebSocketでブロードキャスト
///
/// 同期的にブロードキャストし、完了を待機する。
/// RwLockガードをawait境界をまたいで保持しないよう設計。
#[tauri::command]
pub async fn broadcast_brand_update(
    brand_settings: BrandSettings,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let payload = BrandUpdatePayload {
        logo_url: brand_settings.logo_url,
        text: brand_settings.text,
    };

    let message = WsMessage::BrandUpdate { payload };

    // ステップ1: serverのガードを取得してpeersのArcをクローン、即座にガード解放
    let peers_arc = {
        let ws_state = state.server.read().await;
        ws_state.get_peers_arc()
    }; // ここでws_stateのガード解放

    // ステップ2: ガード解放後にpeersをawait（ガード保持中にawaitしていない）
    let peers_guard = peers_arc.read().await;
    let peers: Vec<_> = peers_guard
        .iter()
        .map(|(id, tx)| (*id, tx.clone()))
        .collect();
    drop(peers_guard);

    crate::server::websocket::WebSocketState::send_to_peers(&peers, &message);
    log::debug!("Brand update broadcasted to {} peers", peers.len());

    Ok(())
}

/// ブランド設定を保存してブロードキャスト
#[tauri::command]
pub async fn save_and_broadcast_brand(
    brand_settings: BrandSettings,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let validated = save_brand_settings(brand_settings, state.clone()).await?;
    broadcast_brand_update(validated, state).await?;
    Ok(())
}

/// ブランド設定の入力検証
fn validate_brand_settings(settings: BrandSettings) -> Result<BrandSettings, String> {
    let mut validated = settings;

    // ロゴURL検証
    if let Some(ref url) = validated.logo_url {
        // 長さチェック
        if url.len() > MAX_LOGO_URL_LENGTH {
            return Err(format!(
                "Logo URL too long: {} bytes (max {})",
                url.len(),
                MAX_LOGO_URL_LENGTH
            ));
        }

        // スキーム検証（http, https, data:image/ のみ許可）
        // NOTE: data:はロゴ画像用途のためdata:image/に限定（data:text/html等を拒否）
        if !url.is_empty() {
            let is_valid = url.starts_with("http://")
                || url.starts_with("https://")
                || url.starts_with("data:image/");

            if !is_valid {
                return Err(
                    "Invalid URL scheme. Only http, https, or data:image/ URLs are allowed."
                        .to_string(),
                );
            }
        }

        // 空文字列はNoneに正規化
        if url.is_empty() {
            validated.logo_url = None;
        }
    }

    // テキスト検証
    if let Some(ref text) = validated.text {
        if text.chars().count() > MAX_TEXT_LENGTH {
            return Err(format!(
                "Text too long: {} chars (max {})",
                text.chars().count(),
                MAX_TEXT_LENGTH
            ));
        }

        // 空文字列はNoneに正規化
        if text.is_empty() {
            validated.text = None;
        }
    }

    Ok(validated)
}
