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
    pub setlist_id: String,
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
    pub layout: LayoutPreset,
    pub primary_color: String,
    pub font_family: String,
    pub border_radius: u32,
    // コメントオーバーレイ設定
    pub comment: CommentSettingsPayload,
    // セットリストオーバーレイ設定
    pub setlist: SetlistSettingsPayload,
}

/// コメントオーバーレイの表示位置
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CommentPosition {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

/// セットリストオーバーレイの表示位置
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SetlistPosition {
    Top,
    Bottom,
    Left,
    Right,
}

/// レイアウトプリセット
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LayoutPreset {
    Streaming,
    Talk,
    Music,
    Gaming,
    Custom,
}

/// NOTE: maxCountは画面高さベースの自動調整に統一したため削除
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentSettingsPayload {
    pub enabled: bool,
    pub position: CommentPosition,
    pub show_avatar: bool,
    pub font_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetlistSettingsPayload {
    pub enabled: bool,
    pub position: SetlistPosition,
    pub show_artist: bool,
    pub font_size: u32,
}
