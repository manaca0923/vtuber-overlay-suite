//! キュー（短冊）管理コマンド
//!
//! キューアイテムの追加・削除・クリア、WebSocketブロードキャストを提供する。
//! データはDBのsettingsテーブルに保存される。

use serde::{Deserialize, Serialize};
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
#[tauri::command]
pub async fn get_queue_state(state: tauri::State<'_, AppState>) -> Result<QueueState, String> {
    let pool = &state.db;

    let result: Option<(String,)> =
        sqlx::query_as("SELECT value FROM settings WHERE key = 'queue_state'")
            .fetch_optional(pool)
            .await
            .map_err(|e| format!("DB error: {}", e))?;

    if let Some((json_str,)) = result {
        let queue_state: QueueState =
            serde_json::from_str(&json_str).map_err(|e| format!("JSON parse error: {}", e))?;
        Ok(queue_state)
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
#[tauri::command]
pub async fn broadcast_queue_update(
    queue_state: QueueState,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let payload = QueueUpdatePayload {
        title: queue_state.title,
        items: queue_state.items,
    };

    let server_state = state.server.read().await;
    server_state
        .broadcast(WsMessage::QueueUpdate { payload })
        .await;

    log::info!("Queue update broadcasted");
    Ok(())
}

/// キュー状態を保存してブロードキャスト
#[tauri::command]
pub async fn save_and_broadcast_queue(
    queue_state: QueueState,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    save_queue_state(queue_state.clone(), state.clone()).await?;
    broadcast_queue_update(queue_state, state).await?;
    Ok(())
}
