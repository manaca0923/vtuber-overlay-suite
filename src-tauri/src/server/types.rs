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

    /// KPI更新
    #[serde(rename = "kpi:update")]
    KpiUpdate { payload: KpiUpdatePayload },

    /// キュー更新
    #[serde(rename = "queue:update")]
    QueueUpdate { payload: QueueUpdatePayload },

    /// 告知更新
    #[serde(rename = "promo:update")]
    PromoUpdate { payload: PromoUpdatePayload },
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
#[serde(rename_all = "kebab-case")]
pub enum LayoutPreset {
    Streaming,
    Talk,
    Music,
    Gaming,
    Custom,
    #[serde(rename = "three-column")]
    ThreeColumn,
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

/// slot ID（3カラムレイアウト v2）
/// 11個のslot配置システム
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SlotId {
    #[serde(rename = "left.top")]
    LeftTop,
    #[serde(rename = "left.topBelow")]
    LeftTopBelow,
    #[serde(rename = "left.middle")]
    LeftMiddle,
    #[serde(rename = "left.lower")]
    LeftLower,
    #[serde(rename = "left.bottom")]
    LeftBottom,
    #[serde(rename = "center.full")]
    CenterFull,
    #[serde(rename = "right.top")]
    RightTop,
    #[serde(rename = "right.upper")]
    RightUpper,
    #[serde(rename = "right.lowerLeft")]
    RightLowerLeft,
    #[serde(rename = "right.lowerRight")]
    RightLowerRight,
    #[serde(rename = "right.bottom")]
    RightBottom,
}

/// KPI更新ペイロード
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KpiUpdatePayload {
    /// 主数値（視聴者数など）
    pub main: Option<i64>,
    /// 主数値のラベル
    pub label: Option<String>,
    /// 副数値（高評価数など）
    pub sub: Option<i64>,
    /// 副数値のラベル
    pub sub_label: Option<String>,
}

/// キュー更新ペイロード
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueUpdatePayload {
    /// キュータイトル
    pub title: Option<String>,
    /// キューアイテム一覧
    pub items: Vec<QueueItem>,
}

/// キューアイテム
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueItem {
    /// アイテムID
    pub id: Option<String>,
    /// 表示テキスト
    pub text: String,
}

/// 告知更新ペイロード
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromoUpdatePayload {
    /// 告知アイテム一覧
    pub items: Vec<PromoItem>,
    /// サイクル間隔（秒）
    pub cycle_sec: Option<u32>,
    /// 各アイテム表示時間（秒）
    pub show_sec: Option<u32>,
}

/// 告知アイテム
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromoItem {
    /// 表示テキスト
    pub text: String,
    /// アイコン（絵文字など）
    pub icon: Option<String>,
}
