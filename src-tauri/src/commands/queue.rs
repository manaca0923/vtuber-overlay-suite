//! キュー（短冊）管理コマンド
//!
//! キューアイテムの追加・削除・クリア、WebSocketブロードキャストを提供する。
//! データはDBのsettingsテーブルに保存される。

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::server::types::{QueueItem, QueueUpdatePayload, WsMessage};
use crate::AppState;

/// キュー状態（保存用）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct QueueState {
    /// キュータイトル（例: "リクエスト曲", "待機リスト"）
    pub title: Option<String>,
    /// キューアイテム一覧
    pub items: Vec<QueueItem>,
}

/// キュー状態を取得
///
/// ## 旧データ互換性
/// `id: None`のアイテムが含まれる場合、新しいUUIDを付与して正規化する。
/// これにより、旧データでも削除操作が可能になる。
///
/// ## JSON破損時のフォールバック
/// 保存されているJSONが破損している場合:
/// 1. 破損データを`queue_state_backup_{timestamp}`キーに退避保存
/// 2. 破損した`queue_state`キーを削除（次回以降のフォールバックを防止）
/// 3. デフォルト状態にフォールバックして返却
/// 4. 警告ログを出力
///
/// これにより、UIが復旧不能な状態になることを防止する。
#[tauri::command]
pub async fn get_queue_state(state: tauri::State<'_, AppState>) -> Result<QueueState, String> {
    let pool = &state.db;

    let result: Option<(String,)> =
        sqlx::query_as("SELECT value FROM settings WHERE key = 'queue_state'")
            .fetch_optional(pool)
            .await
            .map_err(|e| format!("DB error: {}", e))?;

    if let Some((json_str,)) = result {
        match serde_json::from_str::<QueueState>(&json_str) {
            Ok(mut queue_state) => {
                // 旧データ互換性: id: None のアイテムにUUIDを付与
                let mut needs_save = false;
                for item in &mut queue_state.items {
                    if item.id.is_none() {
                        item.id = Some(Uuid::new_v4().to_string());
                        needs_save = true;
                    }
                }

                // 正規化したデータを保存（マイグレーション）
                if needs_save {
                    log::info!("Migrating queue items: assigning UUIDs to items without id");
                    let now = chrono::Utc::now().to_rfc3339();
                    let json_str = serde_json::to_string(&queue_state)
                        .map_err(|e| format!("JSON serialize error: {}", e))?;

                    sqlx::query(
                        r#"
                        INSERT INTO settings (key, value, updated_at)
                        VALUES ('queue_state', ?, ?)
                        ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at
                        "#,
                    )
                    .bind(&json_str)
                    .bind(&now)
                    .execute(pool)
                    .await
                    .map_err(|e| format!("DB error during migration: {}", e))?;
                }

                Ok(queue_state)
            }
            Err(e) => {
                // JSON破損時: 破損データを退避してデフォルト値を返す
                log::warn!(
                    "Queue state JSON corrupted, falling back to default. Error: {}",
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
                .bind(format!("queue_state_backup_{}", now))
                .bind(&json_str)
                .bind(&now)
                .execute(pool)
                .await;

                match backup_result {
                    Ok(_) => {
                        log::info!("Corrupted queue state backed up successfully");
                        // バックアップ成功時のみ破損キーを削除
                        if let Err(delete_err) =
                            sqlx::query("DELETE FROM settings WHERE key = 'queue_state'")
                                .execute(pool)
                                .await
                        {
                            log::error!("Failed to delete corrupted queue state: {}", delete_err);
                        } else {
                            log::info!(
                                "Deleted corrupted queue_state key to prevent repeated fallback"
                            );
                        }
                    }
                    Err(backup_err) => {
                        // バックアップ失敗時は元キーを保持（データ損失防止）
                        log::error!(
                            "Failed to backup corrupted queue state, keeping original key: {}",
                            backup_err
                        );
                    }
                }

                Ok(QueueState::default())
            }
        }
    } else {
        Ok(QueueState::default())
    }
}

/// キュー状態を保存
#[tauri::command]
pub async fn save_queue_state(
    queue_state: QueueState,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let pool = &state.db;
    let now = chrono::Utc::now().to_rfc3339();

    let json_str =
        serde_json::to_string(&queue_state).map_err(|e| format!("JSON serialize error: {}", e))?;

    sqlx::query(
        r#"
        INSERT INTO settings (key, value, updated_at)
        VALUES ('queue_state', ?, ?)
        ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at
        "#,
    )
    .bind(&json_str)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    log::info!("Queue state saved");
    Ok(())
}

/// キューにアイテムを追加
#[tauri::command]
pub async fn add_queue_item(
    text: String,
    state: tauri::State<'_, AppState>,
) -> Result<QueueState, String> {
    let mut queue_state = get_queue_state(state.clone()).await?;

    let item = QueueItem {
        id: Some(Uuid::new_v4().to_string()),
        text,
    };
    queue_state.items.push(item);

    save_queue_state(queue_state.clone(), state).await?;
    Ok(queue_state)
}

/// キューからアイテムを削除
#[tauri::command]
pub async fn remove_queue_item(
    id: String,
    state: tauri::State<'_, AppState>,
) -> Result<QueueState, String> {
    let mut queue_state = get_queue_state(state.clone()).await?;

    queue_state.items.retain(|item| {
        item.id.as_ref().map(|item_id| item_id != &id).unwrap_or(true)
    });

    save_queue_state(queue_state.clone(), state).await?;
    Ok(queue_state)
}

/// キューをクリア
#[tauri::command]
pub async fn clear_queue(state: tauri::State<'_, AppState>) -> Result<QueueState, String> {
    let mut queue_state = get_queue_state(state.clone()).await?;

    queue_state.items.clear();

    save_queue_state(queue_state.clone(), state).await?;
    Ok(queue_state)
}

/// キュータイトルを設定
#[tauri::command]
pub async fn set_queue_title(
    title: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<QueueState, String> {
    let mut queue_state = get_queue_state(state.clone()).await?;

    queue_state.title = title;

    save_queue_state(queue_state.clone(), state).await?;
    Ok(queue_state)
}

/// キュー更新をWebSocketでブロードキャスト
///
/// ## 設計ノート
/// - Fire-and-forgetパターン: ブロードキャストは`tokio::spawn`でバックグラウンド実行
/// - 呼び出し元はブロードキャスト完了を待たずに即座に`Ok(())`を返す
/// - ブロードキャスト失敗はログ出力のみで、呼び出し元のコマンド成功には影響しない
/// - RwLockガードをawait境界をまたいで保持しないようにtokio::spawnで分離
#[tauri::command]
pub async fn broadcast_queue_update(
    queue_state: QueueState,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let payload = QueueUpdatePayload {
        title: queue_state.title,
        items: queue_state.items,
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
    let message = WsMessage::QueueUpdate { payload };
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
        log::debug!("Queue update broadcasted");
    });

    Ok(())
}

/// キュー状態を保存してブロードキャスト
///
/// ## エラーハンドリング
/// - 保存成功後のブロードキャスト失敗は警告ログのみ、Okを返す
/// - 保存失敗時はエラーを返す
/// - これにより「保存完了しているのにエラー」という混乱を回避
#[tauri::command]
pub async fn save_and_broadcast_queue(
    queue_state: QueueState,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    // 保存（失敗時はエラーを返す）
    save_queue_state(queue_state.clone(), state.clone()).await?;

    // ブロードキャスト（失敗しても保存は完了しているのでOkを返す）
    if let Err(e) = broadcast_queue_update(queue_state, state).await {
        log::warn!("Broadcast failed after save succeeded: {}", e);
    }
    Ok(())
}
