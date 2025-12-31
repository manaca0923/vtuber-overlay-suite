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
        /// 即座に表示するかどうか（gRPC/InnerTubeの場合はtrue、ポーリングの場合はfalse）
        #[serde(default)]
        instant: bool,
        /// バッファ間隔（ミリ秒）。InnerTubeは1000、公式APIはNone（デフォルト5000）
        #[serde(skip_serializing_if = "Option::is_none")]
        buffer_interval_ms: Option<u32>,
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

    /// 天気更新
    #[serde(rename = "weather:update")]
    WeatherUpdate { payload: WeatherUpdatePayload },

    /// スパチャ追加（専用ウィジェット表示用）
    #[serde(rename = "superchat:add")]
    SuperchatAdd { payload: SuperchatPayload },

    /// スパチャ削除（表示完了時）
    #[serde(rename = "superchat:remove")]
    SuperchatRemove { payload: SuperchatRemovePayload },
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
    pub comment: CommentSettings,
    // セットリストオーバーレイ設定
    pub setlist: SetlistSettings,
    // 天気ウィジェット設定
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weather: Option<WeatherSettings>,
    // ウィジェット表示設定
    #[serde(skip_serializing_if = "Option::is_none")]
    pub widget: Option<WidgetVisibilitySettings>,
    // スパチャウィジェット設定
    #[serde(skip_serializing_if = "Option::is_none")]
    pub superchat: Option<SuperchatSettings>,
}

/// 天気ウィジェット設定（共通型）
/// - DB保存用（overlay.rs）
/// - WebSocket配信用（SettingsUpdatePayload）
/// - HTTP API用（http.rs）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeatherSettings {
    pub enabled: bool,
    pub position: WeatherPosition,
}

/// ウィジェット表示設定（共通型）
/// - DB保存用（overlay.rs）
/// - WebSocket配信用（SettingsUpdatePayload）
/// - HTTP API用（http.rs）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WidgetVisibilitySettings {
    pub clock: bool,
    pub weather: bool,
    pub comment: bool,
    pub superchat: bool,
    pub logo: bool,
    pub setlist: bool,
    pub kpi: bool,
    pub tanzaku: bool,
    pub announcement: bool,
}

/// スパチャウィジェット設定（共通型）
/// - DB保存用（overlay.rs）
/// - WebSocket配信用（SettingsUpdatePayload）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuperchatSettings {
    /// 同時表示数（1-3、デフォルト: 1）
    pub max_display: u32,
    /// 表示時間（秒、10-120、デフォルト: 60）
    pub display_duration_sec: u32,
    /// キュー表示ON/OFF（待機中のスパチャを順次表示）
    pub queue_enabled: bool,
}

/// 天気ウィジェットの表示位置
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WeatherPosition {
    LeftTop,
    LeftBottom,
    RightTop,
    RightBottom,
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

/// コメントオーバーレイ設定（共通型）
/// - DB保存用（overlay.rs）
/// - WebSocket配信用（SettingsUpdatePayload）
/// - HTTP API用（http.rs）
/// NOTE: maxCountは画面高さベースの自動調整に統一したため削除
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentSettings {
    pub enabled: bool,
    pub position: CommentPosition,
    pub show_avatar: bool,
    pub font_size: u32,
}

/// セットリストオーバーレイ設定（共通型）
/// - DB保存用（overlay.rs）
/// - WebSocket配信用（SettingsUpdatePayload）
/// - HTTP API用（http.rs）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetlistSettings {
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

/// 天気更新ペイロード
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeatherUpdatePayload {
    /// 天気アイコン（絵文字）
    pub icon: String,
    /// 気温（摂氏）
    pub temp: f64,
    /// 天気の説明
    pub description: String,
    /// 地域名
    pub location: String,
    /// 湿度（%）
    pub humidity: Option<i32>,
}

/// スパチャペイロード（専用ウィジェット表示用）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuperchatPayload {
    /// メッセージID（コメントIDと同一）
    pub id: String,
    /// 送信者名
    pub author_name: String,
    /// 送信者アイコンURL
    pub author_image_url: String,
    /// 金額表示文字列（"¥1,000" 等）
    pub amount: String,
    /// 金額（マイクロ単位）
    /// 例: ¥1,000 = 1_000_000_000 micros
    pub amount_micros: u64,
    /// 通貨コード（"JPY", "USD" 等）
    pub currency: String,
    /// メッセージ本文
    pub message: String,
    /// 金額帯（1-7, YouTube公式準拠）
    /// 1: ¥100-199, 2: ¥200-499, 3: ¥500-999,
    /// 4: ¥1,000-1,999, 5: ¥2,000-4,999, 6: ¥5,000-9,999, 7: ¥10,000+
    pub tier: u8,
    /// 表示時間（ミリ秒）
    pub display_duration_ms: u64,
}

/// スパチャ削除ペイロード
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuperchatRemovePayload {
    /// 削除するスパチャのID
    pub id: String,
}
