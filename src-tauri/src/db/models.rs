use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// 楽曲モデル
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Song {
    pub id: String,
    pub title: String,
    pub artist: Option<String>,
    pub category: Option<String>,
    pub tags: Option<String>, // JSON array string
    pub duration_seconds: Option<i64>,
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

    /// タグをVec<String>として取得
    pub fn tags_vec(&self) -> Vec<String> {
        self.tags
            .as_ref()
            .and_then(|t| serde_json::from_str(t).ok())
            .unwrap_or_default()
    }
}

/// セットリストモデル
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Setlist {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl Setlist {
    pub fn new(name: String, description: Option<String>) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

/// セットリスト楽曲（中間テーブル）
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SetlistSong {
    pub id: String,
    pub setlist_id: String,
    pub song_id: String,
    pub position: i64,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
}

/// 結合クエリ用（Song情報を含む）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetlistSongWithDetails {
    pub id: String,
    pub position: i64,
    pub song: Song,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
    pub status: SongStatus,
}

/// 楽曲ステータス
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum SongStatus {
    Pending,
    Current,
    Done,
}

/// セットリスト（楽曲リスト付き）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetlistWithSongs {
    pub setlist: Setlist,
    pub songs: Vec<SetlistSongWithDetails>,
    pub current_index: i64,
}

/// 設定モデル
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Setting {
    pub key: String,
    pub value: String,
    pub updated_at: String,
}

/// コメントログモデル
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
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
    pub message_data: Option<String>, // JSON
    pub published_at: String,
    pub created_at: String,
}
