use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};

/// YouTube APIは数値を文字列で返すことがあるため、両方に対応
fn deserialize_string_or_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrU64 {
        String(String),
        U64(u64),
    }
    
    match StringOrU64::deserialize(deserializer)? {
        StringOrU64::String(s) => s.parse::<u64>().map_err(|e| {
            D::Error::custom(format!("Failed to parse '{}' as u64: {}", s, e))
        }),
        StringOrU64::U64(n) => Ok(n),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub id: String,
    pub message: String,
    pub author_name: String,       // → authorName (serde rename)
    pub author_channel_id: String, // → authorChannelId
    pub author_image_url: String,  // → authorImageUrl
    pub published_at: DateTime<Utc>, // → publishedAt
    pub is_owner: bool,            // → isOwner
    pub is_moderator: bool,        // → isModerator
    pub is_member: bool,           // → isMember (isChatSponsor)
    pub is_verified: bool,         // → isVerified
    pub message_type: MessageType, // → messageType
    /// InnerTube API使用時のみ設定される構造化メッセージ（絵文字情報を含む）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_runs: Option<Vec<MessageRun>>,
}

/// メッセージのruns配列要素（テキストまたは絵文字）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageRun {
    Text { text: String },
    Emoji { emoji: EmojiInfo },
}

/// 絵文字情報
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmojiInfo {
    pub emoji_id: String,
    pub shortcuts: Vec<String>,
    pub image: EmojiImage,
    pub is_custom_emoji: bool,
}

/// 絵文字画像情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmojiImage {
    pub thumbnails: Vec<EmojiThumbnail>,
}

/// 絵文字サムネイル
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmojiThumbnail {
    pub url: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum MessageType {
    #[serde(rename = "text")]
    Text,
    #[serde(rename = "superChat")]
    SuperChat { amount: String, currency: String },
    #[serde(rename = "superSticker")]
    SuperSticker { sticker_id: String },
    #[serde(rename = "membership")]
    Membership { level: String },
    #[serde(rename = "membershipGift")]
    MembershipGift { count: u32 },
}

// YouTube API レスポンス型
#[derive(Debug, Deserialize)]
pub struct LiveChatMessagesResponse {
    #[serde(rename = "pollingIntervalMillis", deserialize_with = "deserialize_string_or_u64")]
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
    #[serde(rename = "superChatDetails")]
    pub super_chat_details: Option<SuperChatDetails>,
    #[serde(rename = "superStickerDetails")]
    pub super_sticker_details: Option<SuperStickerDetails>,
    #[serde(rename = "membershipGiftingDetails")]
    pub membership_gifting_details: Option<MembershipGiftingDetails>,
}

// YouTube APIレスポンスの全フィールドをパースするため、一部未使用フィールドあり
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct SuperChatDetails {
    #[serde(rename = "amountDisplayString")]
    pub amount_display_string: String,
    pub currency: String,
    #[serde(rename = "amountMicros", deserialize_with = "deserialize_string_or_u64")]
    pub amount_micros: u64,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct SuperStickerDetails {
    #[serde(rename = "superStickerMetadata")]
    pub super_sticker_metadata: Option<SuperStickerMetadata>,
    #[serde(rename = "amountDisplayString")]
    pub amount_display_string: String,
    pub currency: String,
    #[serde(rename = "amountMicros", deserialize_with = "deserialize_string_or_u64")]
    pub amount_micros: u64,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct SuperStickerMetadata {
    #[serde(rename = "stickerId")]
    pub sticker_id: String,
    #[serde(rename = "altText")]
    pub alt_text: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct MembershipGiftingDetails {
    #[serde(rename = "giftMembershipsCount")]
    pub gift_memberships_count: Option<u32>,
    #[serde(rename = "giftMembershipsLevelName")]
    pub gift_memberships_level_name: Option<String>,
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
    /// 動画統計情報
    pub statistics: Option<VideoStatistics>,
}

#[derive(Debug, Deserialize)]
pub struct LiveStreamingDetails {
    #[serde(rename = "activeLiveChatId")]
    pub active_live_chat_id: Option<String>,
    /// 同時視聴者数（配信中のみ）
    #[serde(rename = "concurrentViewers")]
    pub concurrent_viewers: Option<String>,
}

/// 動画統計情報
#[derive(Debug, Deserialize)]
pub struct VideoStatistics {
    /// 再生回数
    #[serde(rename = "viewCount")]
    pub view_count: Option<String>,
    /// 高評価数
    #[serde(rename = "likeCount")]
    pub like_count: Option<String>,
    /// コメント数
    /// NOTE: YouTube APIレスポンスに含まれるフィールド。現在未使用だが、将来のKPI表示で使用予定
    #[serde(rename = "commentCount")]
    #[allow(dead_code)]
    pub comment_count: Option<String>,
}

/// ライブ配信ステータス
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveStreamStats {
    /// 同時視聴者数
    pub concurrent_viewers: Option<i64>,
    /// 高評価数
    pub like_count: Option<i64>,
    /// 総再生回数
    pub view_count: Option<i64>,
}

/// YouTube APIのメッセージタイプを解析してMessageTypeに変換
pub fn parse_message_type(snippet: &MessageSnippet) -> MessageType {
    match snippet.message_type.as_str() {
        "textMessageEvent" => MessageType::Text,
        "superChatEvent" => {
            if let Some(details) = &snippet.super_chat_details {
                MessageType::SuperChat {
                    amount: details.amount_display_string.clone(),
                    currency: details.currency.clone(),
                }
            } else {
                log::warn!(
                    "superChatEvent without superChatDetails, falling back to Text"
                );
                MessageType::Text
            }
        }
        "superStickerEvent" => {
            if let Some(details) = &snippet.super_sticker_details {
                let sticker_id = details
                    .super_sticker_metadata
                    .as_ref()
                    .map(|m| m.sticker_id.clone())
                    .unwrap_or_default();
                MessageType::SuperSticker { sticker_id }
            } else {
                log::warn!(
                    "superStickerEvent without superStickerDetails, using empty sticker_id"
                );
                MessageType::SuperSticker {
                    sticker_id: String::new(),
                }
            }
        }
        "newSponsorEvent" => {
            // YouTube APIではnewSponsorEventにメンバーシップレベル情報は含まれない
            // レベル情報は別途memberships APIで取得する必要があるが、
            // 現時点では空文字列で対応
            MessageType::Membership {
                level: String::new(),
            }
        }
        "memberMilestoneChatEvent" => {
            // メンバー継続のマイルストーンイベント
            MessageType::Membership {
                level: "milestone".to_string(),
            }
        }
        "membershipGiftingEvent" => {
            if let Some(details) = &snippet.membership_gifting_details {
                let count = details.gift_memberships_count.unwrap_or(1);
                MessageType::MembershipGift { count }
            } else {
                log::warn!(
                    "membershipGiftingEvent without membershipGiftingDetails, using count=1"
                );
                MessageType::MembershipGift { count: 1 }
            }
        }
        "giftMembershipReceivedEvent" => {
            // ギフトを受け取った通知（受け取り側）
            MessageType::MembershipGift { count: 1 }
        }
        _ => {
            log::debug!("Unknown message type: {}", snippet.message_type);
            MessageType::Text
        }
    }
}

