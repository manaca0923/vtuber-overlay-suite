//! 告知（プロモ）管理コマンド
//!
//! 告知アイテムの追加・削除・更新、サイクル設定、WebSocketブロードキャストを提供する。
//! データはDBのsettingsテーブルに保存される。

use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::server::types::{PromoItem, PromoUpdatePayload, WsMessage};
use crate::AppState;

/// デフォルトの各アイテム表示時間（秒）
/// オーバーレイ側のPromoPanel.jsと同期（デフォルト: 6秒）
const DEFAULT_SHOW_SEC: u32 = 6;

/// デフォルトのサイクル間隔（秒）
/// オーバーレイ側のPromoPanel.jsと同期（デフォルト: 30秒）
const DEFAULT_CYCLE_SEC: u32 = 30;

/// 告知状態（保存用）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PromoState {
    /// 告知アイテム一覧
    pub items: Vec<PromoItem>,
    /// サイクル間隔（秒）- 将来的にサイクル間の休憩時間として使用予定
    pub cycle_sec: Option<u32>,
    /// 各アイテム表示時間（秒）
    pub show_sec: Option<u32>,
}

/// 告知状態を取得
///
/// ## JSON破損時のフォールバック
/// 保存されているJSONが破損している場合:
/// 1. 破損データを`promo_state_backup`キーに退避保存
/// 2. 破損した`promo_state`キーを削除（次回以降のフォールバックを防止）
/// 3. デフォルト状態にフォールバックして返却
/// 4. 警告ログを出力
///
/// これにより、UIが復旧不能な状態になることを防止し、
/// 破損データが永続化されて毎回フォールバックし続ける問題を回避する。
#[tauri::command]
pub async fn get_promo_state(state: tauri::State<'_, AppState>) -> Result<PromoState, String> {
    let pool = &state.db;

    let result: Option<(String,)> =
        sqlx::query_as("SELECT value FROM settings WHERE key = 'promo_state'")
            .fetch_optional(pool)
            .await
            .map_err(|e| format!("DB error: {}", e))?;

    if let Some((json_str,)) = result {
        match serde_json::from_str::<PromoState>(&json_str) {
            Ok(promo_state) => Ok(promo_state),
            Err(e) => {
                // JSON破損時: 破損データを退避してデフォルト値を返す
                log::warn!(
                    "Promo state JSON corrupted, falling back to default. Error: {}",
                    e
                );

                // 破損データをバックアップキーに退避（復旧調査用）
                // バックアップ成功時のみ元キーを削除（データ損失防止）
                // ナノ秒精度で衝突を回避
                let now = chrono::Utc::now()
                    .to_rfc3339_opts(chrono::SecondsFormat::Nanos, true);
                let backup_result = sqlx::query(
                    r#"
                    INSERT INTO settings (key, value, updated_at)
                    VALUES (?, ?, ?)
                    ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at
                    "#,
                )
                .bind(format!("promo_state_backup_{}", now))
                .bind(&json_str)
                .bind(&now)
                .execute(pool)
                .await;

                match backup_result {
                    Ok(_) => {
                        log::info!("Corrupted promo state backed up successfully");
                        // バックアップ成功時のみ破損キーを削除
                        if let Err(delete_err) =
                            sqlx::query("DELETE FROM settings WHERE key = 'promo_state'")
                                .execute(pool)
                                .await
                        {
                            log::error!("Failed to delete corrupted promo state: {}", delete_err);
                        } else {
                            log::info!(
                                "Deleted corrupted promo_state key to prevent repeated fallback"
                            );
                        }
                    }
                    Err(backup_err) => {
                        // バックアップ失敗時は元キーを保持（データ損失防止）
                        log::error!(
                            "Failed to backup corrupted promo state, keeping original key: {}",
                            backup_err
                        );
                    }
                }

                Ok(PromoState::default())
            }
        }
    } else {
        Ok(PromoState::default())
    }
}

/// 告知状態を保存
///
/// ## 範囲検証
/// - `cycle_sec`: 10〜120秒にクランプ
/// - `show_sec`: 3〜15秒にクランプ
///
/// Tauriコマンドとして外部公開されているため、任意の値が渡される可能性があります。
/// `set_promo_settings`と同じクランプルールを適用することで、
/// オーバーレイ表示の異常挙動や極端値による性能劣化を防止します。
///
/// ## 戻り値
/// クランプ適用後の`PromoState`を返します。これにより、保存値とブロードキャスト値の
/// 一致が保証されます。
#[tauri::command]
pub async fn save_promo_state(
    mut promo_state: PromoState,
    state: tauri::State<'_, AppState>,
) -> Result<PromoState, String> {
    let pool = &state.db;
    let now = chrono::Utc::now().to_rfc3339();

    // 保存時にも範囲検証を行う（set_promo_settingsと同じルール）
    if let Some(sec) = promo_state.cycle_sec {
        promo_state.cycle_sec = Some(sec.clamp(10, 120));
    }
    if let Some(sec) = promo_state.show_sec {
        promo_state.show_sec = Some(sec.clamp(3, 15));
    }

    let json_str =
        serde_json::to_string(&promo_state).map_err(|e| format!("JSON serialize error: {}", e))?;

    sqlx::query(
        r#"
        INSERT INTO settings (key, value, updated_at)
        VALUES ('promo_state', ?, ?)
        ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at
        "#,
    )
    .bind(&json_str)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    log::info!("Promo state saved");
    Ok(promo_state)
}

/// 告知アイテムを追加
#[tauri::command]
pub async fn add_promo_item(
    text: String,
    icon: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<PromoState, String> {
    let mut promo_state = get_promo_state(state.clone()).await?;

    let item = PromoItem { text, icon };
    promo_state.items.push(item);

    // save_promo_stateはクランプ適用後の値を返すため、その値を使用
    let saved = save_promo_state(promo_state, state).await?;
    Ok(saved)
}

/// 告知アイテムを削除（インデックス指定）
///
/// ## エラー
/// - `index`が範囲外の場合はエラーを返す
#[tauri::command]
pub async fn remove_promo_item(
    index: usize,
    state: tauri::State<'_, AppState>,
) -> Result<PromoState, String> {
    let mut promo_state = get_promo_state(state.clone()).await?;

    if index >= promo_state.items.len() {
        return Err(format!(
            "Index out of range: {} (items count: {})",
            index,
            promo_state.items.len()
        ));
    }

    promo_state.items.remove(index);

    // save_promo_stateはクランプ適用後の値を返すため、その値を使用
    let saved = save_promo_state(promo_state, state).await?;
    Ok(saved)
}

/// 告知アイテムを更新（インデックス指定）
///
/// ## エラー
/// - `index`が範囲外の場合はエラーを返す
#[tauri::command]
pub async fn update_promo_item(
    index: usize,
    text: String,
    icon: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<PromoState, String> {
    let mut promo_state = get_promo_state(state.clone()).await?;

    if index >= promo_state.items.len() {
        return Err(format!(
            "Index out of range: {} (items count: {})",
            index,
            promo_state.items.len()
        ));
    }

    promo_state.items[index] = PromoItem { text, icon };

    // save_promo_stateはクランプ適用後の値を返すため、その値を使用
    let saved = save_promo_state(promo_state, state).await?;
    Ok(saved)
}

/// 告知をクリア
#[tauri::command]
pub async fn clear_promo(state: tauri::State<'_, AppState>) -> Result<PromoState, String> {
    let mut promo_state = get_promo_state(state.clone()).await?;

    promo_state.items.clear();

    // save_promo_stateはクランプ適用後の値を返すため、その値を使用
    let saved = save_promo_state(promo_state, state).await?;
    Ok(saved)
}

/// 告知設定を変更（cycle_sec, show_sec）
#[tauri::command]
pub async fn set_promo_settings(
    cycle_sec: Option<u32>,
    show_sec: Option<u32>,
    state: tauri::State<'_, AppState>,
) -> Result<PromoState, String> {
    let mut promo_state = get_promo_state(state.clone()).await?;

    // cycle_sec: 10-120秒でクランプ
    if let Some(sec) = cycle_sec {
        promo_state.cycle_sec = Some(sec.clamp(10, 120));
    }

    // show_sec: 3-15秒でクランプ
    if let Some(sec) = show_sec {
        promo_state.show_sec = Some(sec.clamp(3, 15));
    }

    // save_promo_stateはクランプ適用後の値を返すため、その値を使用
    let saved = save_promo_state(promo_state, state).await?;
    Ok(saved)
}

/// 告知更新をWebSocketでブロードキャスト
///
/// ## 設計ノート
/// - Fire-and-forgetパターン: ブロードキャストは`tokio::spawn`でバックグラウンド実行
/// - 呼び出し元はブロードキャスト完了を待たずに即座に`Ok(())`を返す
/// - ブロードキャスト失敗はログ出力のみで、呼び出し元のコマンド成功には影響しない
/// - RwLockガードをawait境界をまたいで保持しないようにtokio::spawnで分離
///
/// ## デフォルト値の適用
/// `show_sec`/`cycle_sec`がNoneの場合、オーバーレイ側で`null`がクランプされて
/// 最小値になってしまう問題を回避するため、ブロードキャスト時にデフォルト値を適用する。
#[tauri::command]
pub async fn broadcast_promo_update(
    promo_state: PromoState,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    // Noneの場合はデフォルト値を適用（オーバーレイ側でnullが最小値にクランプされる問題を回避）
    let payload = PromoUpdatePayload {
        items: promo_state.items,
        cycle_sec: Some(promo_state.cycle_sec.unwrap_or(DEFAULT_CYCLE_SEC)),
        show_sec: Some(promo_state.show_sec.unwrap_or(DEFAULT_SHOW_SEC)),
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
    let message = WsMessage::PromoUpdate { payload };
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
        log::debug!("Promo update broadcasted");
    });

    Ok(())
}

/// 告知状態を保存してブロードキャスト
///
/// `save_promo_state`の戻り値（クランプ適用後）でブロードキャストすることで、
/// 保存値と配信値の一致を保証します。
///
/// ## エラーハンドリング
/// - 保存成功後のブロードキャスト失敗は警告ログのみ、Okを返す
/// - 保存失敗時はエラーを返す
/// - これにより「保存完了しているのにエラー」という混乱を回避
#[tauri::command]
pub async fn save_and_broadcast_promo(
    promo_state: PromoState,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    // 保存（失敗時はエラーを返す）
    // save_promo_stateはクランプ適用後の値を返すため、その値でブロードキャスト
    let validated_state = save_promo_state(promo_state, state.clone()).await?;

    // ブロードキャスト（失敗しても保存は完了しているのでOkを返す）
    if let Err(e) = broadcast_promo_update(validated_state, state).await {
        log::warn!("Broadcast failed after save succeeded: {}", e);
    }
    Ok(())
}
