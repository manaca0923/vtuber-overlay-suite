use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub message: String,
    pub author_name: String,
    pub author_channel_id: String,
    pub author_image_url: String,
    pub published_at: DateTime<Utc>,
    pub is_owner: bool,
    pub is_moderator: bool,
    pub is_member: bool, // isChatSponsor
    pub is_verified: bool,
    pub message_type: MessageType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum MessageType {
    Text,
    SuperChat { amount: String, currency: String },
    SuperSticker { sticker_id: String },
    Membership { level: String },
    MembershipGift { count: u32 },
}

// YouTube API レスポンス型
#[derive(Debug, Deserialize)]
pub struct LiveChatMessagesResponse {
    #[serde(rename = "pollingIntervalMillis")]
    pub polling_interval_millis: u64,
    #[serde(rename = "nextPageToken")]
    pub next_page_token: Option<String>,
    pub items: Vec<LiveChatMessageItem>,
}

#[derive(Debug, Deserialize)]
pub struct LiveChatMessageItem {
    pub id: String,
    pub snippet: MessageSnippet,
    #[serde(rename = "authorDetails")]
    pub author_details: AuthorDetails,
}

#[derive(Debug, Deserialize)]
pub struct MessageSnippet {
    #[serde(rename = "type")]
    pub message_type: String,
    #[serde(rename = "publishedAt")]
    pub published_at: String,
    #[serde(rename = "displayMessage")]
    pub display_message: String,
}

#[derive(Debug, Deserialize)]
pub struct AuthorDetails {
    #[serde(rename = "channelId")]
    pub channel_id: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "profileImageUrl")]
    pub profile_image_url: String,
    #[serde(rename = "isVerified")]
    pub is_verified: bool,
    #[serde(rename = "isChatOwner")]
    pub is_chat_owner: bool,
    #[serde(rename = "isChatSponsor")]
    pub is_chat_sponsor: bool,
    #[serde(rename = "isChatModerator")]
    pub is_chat_moderator: bool,
}

#[derive(Debug, Deserialize)]
pub struct VideoResponse {
    pub items: Vec<VideoItem>,
}

#[derive(Debug, Deserialize)]
pub struct VideoItem {
    #[serde(rename = "liveStreamingDetails")]
    pub live_streaming_details: Option<LiveStreamingDetails>,
}

#[derive(Debug, Deserialize)]
pub struct LiveStreamingDetails {
    #[serde(rename = "activeLiveChatId")]
    pub active_live_chat_id: Option<String>,
}
