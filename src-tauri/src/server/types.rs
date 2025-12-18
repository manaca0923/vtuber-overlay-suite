use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::websocket::WebSocketState;

/// サーバー共有状態
pub type ServerState = Arc<RwLock<WebSocketState>>;

/// WebSocketメッセージ種別
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum WsMessage {
    /// コメント追加
    #[serde(rename = "comment:add")]
    CommentAdd {
        payload: crate::youtube::types::ChatMessage,
    },

    /// コメント削除（モデレーション）
    #[serde(rename = "comment:remove")]
    CommentRemove { payload: CommentRemovePayload },

    /// セットリスト更新
    #[serde(rename = "setlist:update")]
    SetlistUpdate { payload: SetlistUpdatePayload },

    /// 設定更新
    #[serde(rename = "settings:update")]
    SettingsUpdate { payload: SettingsUpdatePayload },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentRemovePayload {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetlistUpdatePayload {
    pub current_index: i32,
    pub songs: Vec<SongItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SongItem {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub status: SongStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SongStatus {
    Pending,
    Current,
    Done,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsUpdatePayload {
    pub theme: String,
    pub primary_color: String,
    pub position: String,
    pub visible: bool,
}
