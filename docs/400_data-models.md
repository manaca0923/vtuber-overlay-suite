# データモデル定義

## SQLite スキーマ

### 初期マイグレーション

```sql
-- 001_initial.sql

-- 設定テーブル
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 楽曲テーブル
CREATE TABLE IF NOT EXISTS songs (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    artist TEXT,
    category TEXT,
    tags TEXT,  -- JSON array
    duration_seconds INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- セットリストテーブル
CREATE TABLE IF NOT EXISTS setlists (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- セットリスト楽曲（中間テーブル）
CREATE TABLE IF NOT EXISTS setlist_songs (
    id TEXT PRIMARY KEY,
    setlist_id TEXT NOT NULL REFERENCES setlists(id) ON DELETE CASCADE,
    song_id TEXT NOT NULL REFERENCES songs(id) ON DELETE CASCADE,
    position INTEGER NOT NULL,
    started_at TEXT,  -- 演奏開始時刻
    ended_at TEXT,    -- 演奏終了時刻
    UNIQUE(setlist_id, position)
);

-- コメントログテーブル
CREATE TABLE IF NOT EXISTS comment_logs (
    id TEXT PRIMARY KEY,
    youtube_id TEXT NOT NULL,
    message TEXT NOT NULL,
    author_name TEXT NOT NULL,
    author_channel_id TEXT NOT NULL,
    author_image_url TEXT,
    is_owner INTEGER NOT NULL DEFAULT 0,
    is_moderator INTEGER NOT NULL DEFAULT 0,
    is_member INTEGER NOT NULL DEFAULT 0,
    message_type TEXT NOT NULL DEFAULT 'text',
    message_data TEXT,  -- JSON (スパチャ金額等)
    published_at TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- インデックス
CREATE INDEX IF NOT EXISTS idx_setlist_songs_setlist ON setlist_songs(setlist_id);
CREATE INDEX IF NOT EXISTS idx_setlist_songs_position ON setlist_songs(setlist_id, position);
CREATE INDEX IF NOT EXISTS idx_comment_logs_published ON comment_logs(published_at);
CREATE INDEX IF NOT EXISTS idx_songs_title ON songs(title);
```

---

## Rust 型定義

### 設定

```rust
// src-tauri/src/db/models.rs

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Setting {
    pub key: String,
    pub value: String,
    pub updated_at: String,
}

// 設定キー定数
pub mod settings_keys {
    pub const API_KEY: &str = "youtube_api_key";  // セキュアストレージ参照ID
    pub const CURRENT_VIDEO_ID: &str = "current_video_id";
    pub const CURRENT_SETLIST_ID: &str = "current_setlist_id";
    pub const THEME: &str = "theme";
    pub const PRIMARY_COLOR: &str = "primary_color";
    pub const SERVER_PORT_HTTP: &str = "server_port_http";
    pub const SERVER_PORT_WS: &str = "server_port_ws";
    pub const COMMENT_MAX_COUNT: &str = "comment_max_count";
    pub const COMMENT_SHOW_AVATAR: &str = "comment_show_avatar";
    pub const SETLIST_SHOW_ARTIST: &str = "setlist_show_artist";
}
```

### 楽曲

```rust
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Song {
    pub id: String,
    pub title: String,
    pub artist: Option<String>,
    pub category: Option<String>,
    pub tags: Option<String>,  // JSON array
    pub duration_seconds: Option<i32>,
    pub created_at: String,
    pub updated_at: String,
}

impl Song {
    pub fn new(title: String) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            id: Uuid::new_v4().to_string(),
            title,
            artist: None,
            category: None,
            tags: None,
            duration_seconds: None,
            created_at: now.clone(),
            updated_at: now,
        }
    }

    pub fn tags_vec(&self) -> Vec<String> {
        self.tags
            .as_ref()
            .and_then(|t| serde_json::from_str(t).ok())
            .unwrap_or_default()
    }
}
```

### セットリスト

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Setlist {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SetlistSong {
    pub id: String,
    pub setlist_id: String,
    pub song_id: String,
    pub position: i32,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
}

// 結合クエリ用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetlistSongWithDetails {
    pub id: String,
    pub position: i32,
    pub song: Song,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
    pub status: SongStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SongStatus {
    Pending,
    Current,
    Done,
}
```

### コメント

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CommentLog {
    pub id: String,
    pub youtube_id: String,
    pub message: String,
    pub author_name: String,
    pub author_channel_id: String,
    pub author_image_url: Option<String>,
    pub is_owner: bool,
    pub is_moderator: bool,
    pub is_member: bool,
    pub message_type: String,
    pub message_data: Option<String>,  // JSON
    pub published_at: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    Text,
    SuperChat { amount: String, currency: String },
    SuperSticker { sticker_id: String },
    Membership { level: String },
    MembershipGift { count: u32 },
}

impl MessageType {
    pub fn to_string(&self) -> String {
        match self {
            Self::Text => "text".to_string(),
            Self::SuperChat { .. } => "superchat".to_string(),
            Self::SuperSticker { .. } => "supersticker".to_string(),
            Self::Membership { .. } => "membership".to_string(),
            Self::MembershipGift { .. } => "membership_gift".to_string(),
        }
    }

    pub fn to_json(&self) -> Option<String> {
        match self {
            Self::Text => None,
            _ => serde_json::to_string(self).ok(),
        }
    }
}
```

---

## TypeScript 型定義

### 設定

```typescript
// src/types/settings.ts

export interface AppSettings {
  youtubeApiKey: string | null;  // セキュアストレージから取得
  currentVideoId: string | null;
  currentSetlistId: string | null;
  theme: ThemeName;
  primaryColor: string;
  serverPortHttp: number;
  serverPortWs: number;
  comment: CommentSettings;
  setlist: SetlistSettings;
}

export interface CommentSettings {
  enabled: boolean;
  showAvatar: boolean;
  fontSize: number;
  position: Position;
  // NOTE: maxCountは画面高さベースの自動調整に統一したため削除
}

export interface SetlistSettings {
  showArtist: boolean;
  showPrevNext: boolean;
  fontSize: number;
  position: 'top' | 'bottom' | 'left' | 'right';
}

export type ThemeName = 'default' | 'sakura' | 'ocean' | 'custom';
export type Position = 'top-left' | 'top-right' | 'bottom-left' | 'bottom-right';
```

### 楽曲

```typescript
// src/types/song.ts

export interface Song {
  id: string;
  title: string;
  artist: string | null;
  category: string | null;
  tags: string | null;  // JSON array from Rust (要パース)
  durationSeconds: number | null;
  createdAt: string;
  updatedAt: string;
}

// Song.tags の型変換について:
// - DB: TEXT (JSON array string)
// - Rust → Frontend (Song): string | null (JSONシリアライズ済み)
// - Frontend → Rust (CreateSongInput): string[] | null (配列)
// - Rustがserde_json::to_string()でJSON化してDB保存
// - Frontendは parseTags() 関数でパース

export interface CreateSongInput {
  title: string;
  artist?: string | null;
  category?: string | null;
  tags?: string[] | null;  // 配列で渡す。Rust側でJSON文字列に変換
  duration_seconds?: number | null;  // snake_case (Tauriコマンド引数名)
  [key: string]: unknown;
}

export interface UpdateSongInput extends CreateSongInput {
  id: string;
}
```

### セットリスト

```typescript
// src/types/setlist.ts

export interface Setlist {
  id: string;
  name: string;
  description: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface SetlistSong {
  id: string;
  position: number;
  song: Song;
  startedAt: string | null;
  endedAt: string | null;
  status: SongStatus;
}

export type SongStatus = 'pending' | 'current' | 'done';

export interface SetlistWithSongs extends Setlist {
  songs: SetlistSong[];
  currentIndex: number;
}

export interface CreateSetlistInput {
  name: string;
  description?: string;
}

export interface ReorderSongsInput {
  setlistId: string;
  songIds: string[];  // 新しい順序
}
```

### コメント

```typescript
// src/types/comment.ts

export interface ChatMessage {
  id: string;
  youtubeId: string;
  message: string;
  authorName: string;
  authorChannelId: string;
  authorImageUrl: string | null;
  isOwner: boolean;
  isModerator: boolean;
  isMember: boolean;
  messageType: MessageType;
  publishedAt: string;
}

export type MessageType =
  | { type: 'text' }
  | { type: 'superChat'; amount: string; currency: string }
  | { type: 'superSticker'; stickerId: string }
  | { type: 'membership'; level: string }
  | { type: 'membershipGift'; count: number };

// バッジ表示用ヘルパー
export function getBadges(message: ChatMessage): Badge[] {
  const badges: Badge[] = [];
  if (message.isOwner) badges.push({ type: 'owner', label: '配信者' });
  if (message.isModerator) badges.push({ type: 'moderator', label: 'モデ' });
  if (message.isMember) badges.push({ type: 'member', label: 'メンバー' });
  return badges;
}

export interface Badge {
  type: 'owner' | 'moderator' | 'member' | 'verified';
  label: string;
}
```

---

## Tauri Commands

### 型定義

```typescript
// src/types/commands.ts

import { invoke } from '@tauri-apps/api/core';

// 設定
export const getSettings = () => invoke<AppSettings>('get_settings');
export const updateSettings = (settings: Partial<AppSettings>) =>
  invoke<void>('update_settings', { settings });

// APIキー（セキュアストレージ）
export const saveApiKey = (key: string) => invoke<void>('save_api_key', { key });
export const getApiKey = () => invoke<string | null>('get_api_key');
export const deleteApiKey = () => invoke<void>('delete_api_key');

// YouTube
export const getLiveChatId = (videoId: string) =>
  invoke<string>('get_live_chat_id', { videoId });
export const startCommentFetch = (liveChatId: string) =>
  invoke<void>('start_comment_fetch', { liveChatId });
export const stopCommentFetch = () => invoke<void>('stop_comment_fetch');

// 楽曲
export const getSongs = () => invoke<Song[]>('get_songs');
export const createSong = (input: CreateSongInput) =>
  invoke<Song>('create_song', { input });
export const updateSong = (input: UpdateSongInput) =>
  invoke<Song>('update_song', { input });
export const deleteSong = (id: string) => invoke<void>('delete_song', { id });

// セットリスト
export const getSetlists = () => invoke<Setlist[]>('get_setlists');
export const getSetlistWithSongs = (id: string) =>
  invoke<SetlistWithSongs>('get_setlist_with_songs', { id });
export const createSetlist = (input: CreateSetlistInput) =>
  invoke<Setlist>('create_setlist', { input });
export const addSongToSetlist = (setlistId: string, songId: string) =>
  invoke<void>('add_song_to_setlist', { setlistId, songId });
export const removeSongFromSetlist = (setlistId: string, songId: string) =>
  invoke<void>('remove_song_from_setlist', { setlistId, songId });
export const reorderSetlistSongs = (input: ReorderSongsInput) =>
  invoke<void>('reorder_setlist_songs', { input });
export const setCurrentSong = (setlistId: string, position: number) =>
  invoke<void>('set_current_song', { setlistId, position });
export const nextSong = (setlistId: string) =>
  invoke<void>('next_song', { setlistId });
export const previousSong = (setlistId: string) =>
  invoke<void>('previous_song', { setlistId });
```

---

## タイムスタンプ出力フォーマット

### YouTube概要欄用

```typescript
export function formatTimestamps(songs: SetlistSong[]): string {
  return songs
    .filter(s => s.startedAt)
    .map((s, i) => {
      const time = formatTime(s.startedAt!);
      const artist = s.song.artist ? ` / ${s.song.artist}` : '';
      return `${time} ${s.song.title}${artist}`;
    })
    .join('\n');
}

function formatTime(isoString: string): string {
  // 配信開始からの経過時間を計算
  const date = new Date(isoString);
  const hours = date.getUTCHours();
  const minutes = date.getUTCMinutes().toString().padStart(2, '0');
  const seconds = date.getUTCSeconds().toString().padStart(2, '0');

  if (hours > 0) {
    return `${hours}:${minutes}:${seconds}`;
  }
  return `${minutes}:${seconds}`;
}

// 出力例:
// 0:00 オープニング
// 3:45 曲名1 / アーティスト1
// 8:20 曲名2 / アーティスト2
```

---

## 3カラムレイアウト型定義（将来実装予定）

> **ステータス**: 設計完了、実装予定

### TypeScript 型定義

```typescript
// src/types/template.ts

// slot識別子
export type SlotId =
  | 'left.top' | 'left.topBelow' | 'left.middle' | 'left.lower' | 'left.bottom'
  | 'center.full'
  | 'right.top' | 'right.upper' | 'right.lowerLeft' | 'right.lowerRight' | 'right.bottom';

// コンポーネント種別
export type ComponentType =
  | 'ClockWidget' | 'WeatherWidget' | 'ChatLog' | 'SuperChatCard' | 'BrandBlock'
  | 'MainAvatarStage' | 'ChannelBadge' | 'SetList' | 'KPIBlock' | 'PromoPanel' | 'QueueList';

// レイアウト設定
export interface LayoutConfig {
  type: 'threeColumn';
  leftPct: number;    // 0.18-0.28
  centerPct: number;  // 0.44-0.64
  rightPct: number;   // 0.18-0.28
  gutterPx: number;   // 0-64
}

// セーフエリア
export interface SafeAreaPct {
  top: number;    // 0.0-0.10
  right: number;
  bottom: number;
  left: number;
}

// テーマ設定
export interface ThemeConfig {
  fontFamily: string;
  textColor: string;
  panel: {
    bg: string;
    blurPx: number;   // 0-24
    radiusPx: number; // 0-32
  };
  shadow: {
    enabled: boolean;
    blur: number;     // 0-24
    opacity: number;  // 0.0-1.0
    offsetX: number;  // -20 to 20
    offsetY: number;
  };
  outline: {
    enabled: boolean;
    width: number;    // 0-6
    color: string;
  };
}

// チューニング（微調整）
export interface TuningConfig {
  offsetX: number;  // -40 to 40
  offsetY: number;
}

// コンポーネント設定
export interface ComponentConfig {
  id: string;
  type: ComponentType;
  slot: SlotId;
  enabled: boolean;
  style?: Record<string, unknown>;
  rules?: ComponentRules;
  tuning?: TuningConfig;
}

// コンポーネントルール
export interface ComponentRules {
  maxLines?: number;    // 4-14
  maxItems?: number;    // 6-20
  cycleSec?: number;    // 10-120
  showSec?: number;     // 3-15
  overflow?: 'ellipsis' | 'fade' | 'scroll';
}

// テンプレート全体
export interface TemplateConfig {
  layout: LayoutConfig;
  safeAreaPct: SafeAreaPct;
  theme?: ThemeConfig;
  components: ComponentConfig[];
}
```

### Rust 型定義

```rust
// src-tauri/src/server/template_types.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComponentType {
    ClockWidget,
    WeatherWidget,
    ChatLog,
    SuperChatCard,
    BrandBlock,
    MainAvatarStage,
    ChannelBadge,
    SetList,
    KPIBlock,
    PromoPanel,
    QueueList,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LayoutConfig {
    #[serde(rename = "type")]
    pub layout_type: String,  // "threeColumn"
    pub left_pct: f32,
    pub center_pct: f32,
    pub right_pct: f32,
    pub gutter_px: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SafeAreaPct {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentRules {
    pub max_lines: Option<u32>,
    pub max_items: Option<u32>,
    pub cycle_sec: Option<u32>,
    pub show_sec: Option<u32>,
    pub overflow: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TuningConfig {
    pub offset_x: i32,
    pub offset_y: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentConfig {
    pub id: String,
    #[serde(rename = "type")]
    pub component_type: ComponentType,
    pub slot: SlotId,
    pub enabled: bool,
    pub style: Option<serde_json::Value>,
    pub rules: Option<ComponentRules>,
    pub tuning: Option<TuningConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateConfig {
    pub layout: LayoutConfig,
    pub safe_area_pct: SafeAreaPct,
    pub theme: Option<serde_json::Value>,
    pub components: Vec<ComponentConfig>,
}
```

### クランプ規約

テンプレート設定の安全範囲。実装時はJSON Schema検証 + Rust側でのクランプを行う。

| パラメータ | 最小値 | 最大値 | 備考 |
|-----------|--------|--------|------|
| `offsetX` | -40 | 40 | px相当 |
| `offsetY` | -40 | 40 | px相当 |
| `maxLines` | 4 | 14 | ChatLog用 |
| `maxItems` | 6 | 20 | SetList/QueueList用 |
| `cycleSec` | 10 | 120 | PromoPanel用 |
| `showSec` | 3 | 15 | PromoPanel用 |
| `leftPct` | 0.18 | 0.28 | レイアウト比率 |
| `centerPct` | 0.44 | 0.64 | レイアウト比率 |
| `rightPct` | 0.18 | 0.28 | レイアウト比率 |
| `gutterPx` | 0 | 64 | カラム間隔 |
| `blurPx` | 0 | 24 | パネルブラー |
| `radiusPx` | 0 | 32 | 角丸 |

```rust
// クランプ関数の実装例
impl TuningConfig {
    pub fn clamp(&mut self) {
        self.offset_x = self.offset_x.clamp(-40, 40);
        self.offset_y = self.offset_y.clamp(-40, 40);
    }
}

impl ComponentRules {
    pub fn clamp(&mut self) {
        if let Some(ref mut v) = self.max_lines {
            *v = (*v).clamp(4, 14);
        }
        if let Some(ref mut v) = self.max_items {
            *v = (*v).clamp(6, 20);
        }
        if let Some(ref mut v) = self.cycle_sec {
            *v = (*v).clamp(10, 120);
        }
        if let Some(ref mut v) = self.show_sec {
            *v = (*v).clamp(3, 15);
        }
    }
}
```
