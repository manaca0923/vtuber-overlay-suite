use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::websocket::WebSocketState;
use crate::weather::WeatherData;

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

    /// 天気更新（単一都市）
    #[serde(rename = "weather:update")]
    WeatherUpdate { payload: WeatherUpdatePayload },

    /// 天気更新（マルチシティ）
    #[serde(rename = "weather:multi-update")]
    WeatherMultiUpdate { payload: WeatherMultiUpdatePayload },

    /// スパチャ追加（専用ウィジェット表示用）
    #[serde(rename = "superchat:add")]
    SuperchatAdd { payload: SuperchatPayload },

    /// スパチャ削除（表示完了時）
    #[serde(rename = "superchat:remove")]
    SuperchatRemove { payload: SuperchatRemovePayload },

    /// ブランド（ロゴ）更新
    #[serde(rename = "brand:update")]
    BrandUpdate { payload: BrandUpdatePayload },
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
    // テーマ設定（カラー・フォント統合）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme_settings: Option<ThemeSettings>,
}

/// マルチシティ用都市エントリ
///
/// ## 部分的デシリアライズ
/// 全フィールドに`#[serde(default)]`を付与し、フィールド欠損時もデシリアライズ可能
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CityEntry {
    /// ユニークID
    #[serde(default)]
    pub id: String,
    /// API用都市名（英語）
    #[serde(default)]
    pub name: String,
    /// 表示用都市名（日本語）
    #[serde(default)]
    pub display_name: String,
    /// 有効/無効
    #[serde(default = "CityEntry::default_enabled")]
    pub enabled: bool,
    /// 並び順
    #[serde(default)]
    pub order: u32,
}

impl CityEntry {
    fn default_enabled() -> bool {
        true
    }
}

impl Default for CityEntry {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            display_name: String::new(),
            enabled: true,
            order: 0,
        }
    }
}

/// デフォルトのローテーション間隔（秒）
const DEFAULT_ROTATION_INTERVAL_SEC: u32 = 10;

/// マルチシティ設定
///
/// ## 部分的デシリアライズ
/// 全フィールドに`#[serde(default)]`を付与し、フィールド欠損時もデシリアライズ可能
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MultiCitySettings {
    /// マルチシティモード有効
    #[serde(default)]
    pub enabled: bool,
    /// ローテーション間隔（秒）
    #[serde(default = "MultiCitySettings::default_rotation_interval_sec")]
    pub rotation_interval_sec: u32,
    /// 都市リスト
    #[serde(default)]
    pub cities: Vec<CityEntry>,
}

impl MultiCitySettings {
    fn default_rotation_interval_sec() -> u32 {
        DEFAULT_ROTATION_INTERVAL_SEC
    }
}

impl Default for MultiCitySettings {
    fn default() -> Self {
        Self {
            enabled: false,
            rotation_interval_sec: DEFAULT_ROTATION_INTERVAL_SEC,
            cities: Vec::new(),
        }
    }
}

/// 天気ウィジェット設定（共通型）
/// - DB保存用（overlay.rs）
/// - WebSocket配信用（SettingsUpdatePayload）
/// - HTTP API用（http.rs）
///
/// ## 部分的デシリアライズ
/// 全フィールドに`#[serde(default)]`を付与し、フィールド欠損時もデシリアライズ可能
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct WeatherSettings {
    pub enabled: bool,
    pub position: WeatherPosition,
    /// マルチシティモード設定（オプション）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub multi_city: Option<MultiCitySettings>,
}

impl Default for WeatherSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            position: WeatherPosition::default(),
            multi_city: None,
        }
    }
}

/// ウィジェット表示設定（共通型）
/// - DB保存用（overlay.rs）
/// - WebSocket配信用（SettingsUpdatePayload）
/// - HTTP API用（http.rs）
///
/// ## 部分的デシリアライズ
/// 全フィールドに`#[serde(default)]`を付与し、フィールド欠損時もデシリアライズ可能
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
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

impl Default for WidgetVisibilitySettings {
    fn default() -> Self {
        Self {
            clock: true,
            weather: true,
            comment: true,
            superchat: true,
            logo: true,
            setlist: true,
            kpi: true,
            tanzaku: true,
            announcement: true,
        }
    }
}

/// スパチャウィジェット設定（共通型）
/// - DB保存用（overlay.rs）
/// - WebSocket配信用（SettingsUpdatePayload）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuperchatSettings {
    /// 同時表示数（1-3、デフォルト: 1）
    #[serde(default = "SuperchatSettings::default_max_display")]
    pub max_display: u32,
    /// 表示時間（秒、10-120、デフォルト: 60）
    #[serde(default = "SuperchatSettings::default_display_duration_sec")]
    pub display_duration_sec: u32,
    /// キュー表示ON/OFF（待機中のスパチャを順次表示）
    #[serde(default = "SuperchatSettings::default_queue_enabled")]
    pub queue_enabled: bool,
}

impl SuperchatSettings {
    fn default_max_display() -> u32 {
        1
    }

    fn default_display_duration_sec() -> u32 {
        60
    }

    fn default_queue_enabled() -> bool {
        true
    }
}

impl Default for SuperchatSettings {
    fn default() -> Self {
        Self {
            max_display: Self::default_max_display(),
            display_duration_sec: Self::default_display_duration_sec(),
            queue_enabled: Self::default_queue_enabled(),
        }
    }
}

/// カスタムカラーエントリ（最大3件保存）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomColorEntry {
    /// ユニークID (UUID)
    pub id: String,
    /// ユーザー設定の名前
    pub name: String,
    /// HEXカラーコード (#RRGGBB)
    pub color: String,
}

/// ウィジェット個別カラーオーバーライド
/// 各ウィジェットのカラーを個別に設定可能
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WidgetColorOverrides {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clock: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weather: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub superchat: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logo: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub setlist: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kpi: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tanzaku: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub announcement: Option<String>,
}

/// グローバルテーマ名
/// TypeScript側 `ThemeName` と対応
///
/// ## 後方互換性
/// - `#[serde(other)]` により未知の値は `Unknown` にフォールバック
/// - `Default` は `White` を返す（最も汎用的なテーマ）
/// - `normalize()` で `Unknown` を `White` に正規化（API応答前に呼び出す）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum GlobalTheme {
    #[default]
    White,
    Purple,
    Sakura,
    Ocean,
    Custom,
    /// 未知の値（旧バージョンとの互換性用）
    /// API応答前に `normalize()` でデフォルト値に変換される
    #[serde(other)]
    Unknown,
}

impl GlobalTheme {
    /// Unknown値をデフォルト値に正規化
    /// API応答でフロントエンドに渡す前に呼び出す
    pub fn normalize(self) -> Self {
        match self {
            Self::Unknown => Self::default(),
            other => other,
        }
    }
}

/// フォントプリセット
/// TypeScript側 `FontPresetName` と対応
///
/// ## 後方互換性
/// - `#[serde(other)]` により未知の値は `Unknown` にフォールバック
/// - `Default` は `NotoSansJp` を返す（最も互換性の高いフォント）
/// - `normalize()` で `Unknown` を `NotoSansJp` に正規化（API応答前に呼び出す）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum FontPreset {
    #[default]
    NotoSansJp,
    #[serde(rename = "m-plus-1")]
    MPlusOne,
    YuGothic,
    Meiryo,
    System,
    /// 未知の値（旧バージョンとの互換性用）
    /// API応答前に `normalize()` でデフォルト値に変換される
    #[serde(other)]
    Unknown,
}

impl FontPreset {
    /// Unknown値をデフォルト値に正規化
    /// API応答でフロントエンドに渡す前に呼び出す
    pub fn normalize(self) -> Self {
        match self {
            Self::Unknown => Self::default(),
            other => other,
        }
    }
}

/// デフォルトのプライマリカラー
/// ThemeSettings::default() と serde の両方で使用
const DEFAULT_PRIMARY_COLOR: &str = "#6366f1";

/// テーマ設定（カラー・フォント統合）
/// - グローバルテーマ（white/purple/sakura/ocean/custom）
/// - ウィジェット個別カラー
/// - フォントプリセット・システムフォント
///
/// ## 後方互換性
/// - 全フィールドに`#[serde(default)]`を付与し、部分欠損を許容
/// - `Default`実装で安全なデフォルト値を提供（`global_primary_color`含む）
/// - `normalize()`でUnknown値をデフォルト値に正規化
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ThemeSettings {
    /// グローバルテーマ
    pub global_theme: GlobalTheme,
    /// グローバルプライマリカラー (#RRGGBB)
    #[serde(default = "default_primary_color")]
    pub global_primary_color: String,
    /// カスタムカラー（最大3件）
    #[serde(default)]
    pub custom_colors: Vec<CustomColorEntry>,
    /// ウィジェット個別カラーオーバーライド
    #[serde(default)]
    pub widget_color_overrides: WidgetColorOverrides,
    /// フォントプリセット
    pub font_preset: FontPreset,
    /// システムフォント選択時のフォントファミリー
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_font_family: Option<String>,
}

/// ThemeSettingsのDefaultトレイト実装
///
/// ## 注意
/// `#[derive(Default)]`を使用すると`global_primary_color`が空文字になるため、
/// 手動で実装して適切なデフォルト値を設定する
impl Default for ThemeSettings {
    fn default() -> Self {
        Self {
            global_theme: GlobalTheme::default(),
            global_primary_color: DEFAULT_PRIMARY_COLOR.to_string(),
            custom_colors: Vec::new(),
            widget_color_overrides: WidgetColorOverrides::default(),
            font_preset: FontPreset::default(),
            custom_font_family: None,
        }
    }
}

impl ThemeSettings {
    /// Unknown値をデフォルト値に正規化
    /// API応答やWebSocket配信前に呼び出すことで、フロントエンドに未知値が渡るのを防ぐ
    pub fn normalize(mut self) -> Self {
        self.global_theme = self.global_theme.normalize();
        self.font_preset = self.font_preset.normalize();
        self
    }
}

fn default_primary_color() -> String {
    DEFAULT_PRIMARY_COLOR.to_string()
}

/// 天気ウィジェットの表示位置
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum WeatherPosition {
    #[default]
    LeftTop,
    LeftBottom,
    RightTop,
    RightBottom,
}

/// コメントオーバーレイの表示位置
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum CommentPosition {
    TopLeft,
    TopRight,
    BottomLeft,
    #[default]
    BottomRight,
}

/// セットリストオーバーレイの表示位置
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SetlistPosition {
    Top,
    #[default]
    Bottom,
    Left,
    Right,
}

/// レイアウトプリセット
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum LayoutPreset {
    Streaming,
    Talk,
    Music,
    Gaming,
    Custom,
    #[default]
    #[serde(rename = "three-column")]
    ThreeColumn,
}

/// コメントオーバーレイ設定のデフォルトフォントサイズ
const COMMENT_DEFAULT_FONT_SIZE: u32 = 16;

/// コメントオーバーレイ設定（共通型）
/// - DB保存用（overlay.rs）
/// - WebSocket配信用（SettingsUpdatePayload）
/// - HTTP API用（http.rs）
/// NOTE: maxCountは画面高さベースの自動調整に統一したため削除
///
/// ## 部分的デシリアライズ
/// 全フィールドに`#[serde(default)]`を付与し、フィールド欠損時もデシリアライズ可能
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CommentSettings {
    pub enabled: bool,
    pub position: CommentPosition,
    pub show_avatar: bool,
    #[serde(default = "CommentSettings::default_font_size")]
    pub font_size: u32,
}

impl CommentSettings {
    fn default_font_size() -> u32 {
        COMMENT_DEFAULT_FONT_SIZE
    }
}

impl Default for CommentSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            position: CommentPosition::default(),
            show_avatar: true,
            font_size: COMMENT_DEFAULT_FONT_SIZE,
        }
    }
}

/// セットリストオーバーレイ設定のデフォルトフォントサイズ
const SETLIST_DEFAULT_FONT_SIZE: u32 = 24;

/// セットリストオーバーレイ設定（共通型）
/// - DB保存用（overlay.rs）
/// - WebSocket配信用（SettingsUpdatePayload）
/// - HTTP API用（http.rs）
///
/// ## 部分的デシリアライズ
/// 全フィールドに`#[serde(default)]`を付与し、フィールド欠損時もデシリアライズ可能
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SetlistSettings {
    pub enabled: bool,
    pub position: SetlistPosition,
    pub show_artist: bool,
    #[serde(default = "SetlistSettings::default_font_size")]
    pub font_size: u32,
}

impl SetlistSettings {
    fn default_font_size() -> u32 {
        SETLIST_DEFAULT_FONT_SIZE
    }
}

impl Default for SetlistSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            position: SetlistPosition::default(),
            show_artist: true,
            font_size: SETLIST_DEFAULT_FONT_SIZE,
        }
    }
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

impl From<&WeatherData> for WeatherUpdatePayload {
    fn from(data: &WeatherData) -> Self {
        Self {
            icon: data.icon.clone(),
            temp: data.temp,
            description: data.description.clone(),
            location: data.location.clone(),
            humidity: Some(data.humidity),
        }
    }
}

/// マルチシティ天気更新ペイロード
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeatherMultiUpdatePayload {
    /// 各都市の天気データ
    pub cities: Vec<CityWeatherData>,
    /// ローテーション間隔（秒）
    pub rotation_interval_sec: u32,
}

/// 都市ごとの天気データ
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CityWeatherData {
    /// 都市ID
    pub city_id: String,
    /// 都市名（表示用）
    pub city_name: String,
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

/// ブランド（ロゴ）更新ペイロード
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrandUpdatePayload {
    /// ロゴ画像URL（http/https/data スキームのみ許可）
    pub logo_url: Option<String>,
    /// 代替テキスト（ロゴがない場合やエラー時に表示）
    pub text: Option<String>,
}

/// ブランド（ロゴ）設定（保存用）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrandSettings {
    /// ロゴ画像URL
    pub logo_url: Option<String>,
    /// 代替テキスト
    pub text: Option<String>,
}
