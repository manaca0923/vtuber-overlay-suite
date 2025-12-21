//! InnerTube レスポンスパーサー

use chrono::{TimeZone, Utc};

use super::types::*;
use crate::youtube::types::{ChatMessage, EmojiImage, EmojiInfo, EmojiThumbnail, MessageRun, MessageType};

/// InnerTubeレスポンスをChatMessageリストに変換
pub fn parse_chat_response(response: InnerTubeChatResponse) -> Vec<ChatMessage> {
    let Some(contents) = response.continuation_contents else {
        return vec![];
    };
    let Some(continuation) = contents.live_chat_continuation else {
        return vec![];
    };
    let Some(actions) = continuation.actions else {
        return vec![];
    };

    actions
        .into_iter()
        .filter_map(parse_action)
        .collect()
}

/// 単一のアクションをパース
fn parse_action(action: ChatAction) -> Option<ChatMessage> {
    // 通常のメッセージ追加
    if let Some(add_action) = action.add_chat_item_action {
        return parse_chat_item(add_action.item);
    }

    // リプレイアクション（アーカイブ視聴時）
    if let Some(replay_action) = action.replay_chat_item_action {
        if let Some(actions) = replay_action.actions {
            for inner_action in actions {
                if let Some(add_action) = inner_action.add_chat_item_action {
                    return parse_chat_item(add_action.item);
                }
            }
        }
    }

    None
}

/// チャットアイテムをパース
fn parse_chat_item(item: ChatItem) -> Option<ChatMessage> {
    // テキストメッセージ
    if let Some(text_msg) = item.live_chat_text_message_renderer {
        return Some(parse_text_message(text_msg));
    }

    // スーパーチャット
    if let Some(paid_msg) = item.live_chat_paid_message_renderer {
        return Some(parse_paid_message(paid_msg));
    }

    // スーパーステッカー
    if let Some(sticker_msg) = item.live_chat_paid_sticker_renderer {
        return Some(parse_sticker_message(sticker_msg));
    }

    // メンバーシップ
    if let Some(member_msg) = item.live_chat_membership_item_renderer {
        return Some(parse_membership_message(member_msg));
    }

    // メンバーシップギフト
    if let Some(gift_msg) = item.live_chat_sponsor_gift_announcement_renderer {
        return Some(parse_gift_message(gift_msg));
    }

    None
}

/// テキストメッセージをパース
fn parse_text_message(msg: LiveChatTextMessageRenderer) -> ChatMessage {
    let message_runs = msg.message.as_ref().and_then(|m| parse_runs(&m.runs));
    let message_text = extract_plain_text(&message_runs);
    let (is_owner, is_moderator, is_member) = parse_author_badges(&msg.author_badges);
    let published_at = parse_timestamp(&msg.timestamp_usec);

    ChatMessage {
        id: msg.id,
        message: message_text,
        author_name: msg
            .author_name
            .map(|n| n.get_text())
            .unwrap_or_default(),
        author_channel_id: msg.author_external_channel_id.unwrap_or_default(),
        author_image_url: msg
            .author_photo
            .and_then(|p| p.thumbnails.first().map(|t| t.url.clone()))
            .unwrap_or_default(),
        published_at,
        is_owner,
        is_moderator,
        is_member,
        is_verified: false,
        message_type: MessageType::Text,
        message_runs,
    }
}

/// スーパーチャットをパース
fn parse_paid_message(msg: LiveChatPaidMessageRenderer) -> ChatMessage {
    let message_runs = msg.message.as_ref().and_then(|m| parse_runs(&m.runs));
    let message_text = extract_plain_text(&message_runs);
    let (is_owner, is_moderator, is_member) = parse_author_badges(&msg.author_badges);
    let published_at = parse_timestamp(&msg.timestamp_usec);

    // 金額テキストをパース（例: "¥1,000" -> amount="1,000", currency="JPY"）
    let amount_text = msg
        .purchase_amount_text
        .map(|t| t.get_text())
        .unwrap_or_default();
    let (amount, currency) = parse_amount(&amount_text);

    ChatMessage {
        id: msg.id,
        message: message_text,
        author_name: msg
            .author_name
            .map(|n| n.get_text())
            .unwrap_or_default(),
        author_channel_id: msg.author_external_channel_id.unwrap_or_default(),
        author_image_url: msg
            .author_photo
            .and_then(|p| p.thumbnails.first().map(|t| t.url.clone()))
            .unwrap_or_default(),
        published_at,
        is_owner,
        is_moderator,
        is_member,
        is_verified: false,
        message_type: MessageType::SuperChat { amount, currency },
        message_runs,
    }
}

/// スーパーステッカーをパース
fn parse_sticker_message(msg: LiveChatPaidStickerRenderer) -> ChatMessage {
    let (is_owner, is_moderator, is_member) = parse_author_badges(&msg.author_badges);
    let published_at = parse_timestamp(&msg.timestamp_usec);

    // ステッカーIDを抽出
    let sticker_id = msg
        .sticker
        .and_then(|s| s.thumbnails.first().map(|t| t.url.clone()))
        .unwrap_or_default();

    ChatMessage {
        id: msg.id,
        message: String::new(),
        author_name: msg
            .author_name
            .map(|n| n.get_text())
            .unwrap_or_default(),
        author_channel_id: msg.author_external_channel_id.unwrap_or_default(),
        author_image_url: msg
            .author_photo
            .and_then(|p| p.thumbnails.first().map(|t| t.url.clone()))
            .unwrap_or_default(),
        published_at,
        is_owner,
        is_moderator,
        is_member,
        is_verified: false,
        message_type: MessageType::SuperSticker { sticker_id },
        message_runs: None,
    }
}

/// メンバーシップメッセージをパース
fn parse_membership_message(msg: LiveChatMembershipItemRenderer) -> ChatMessage {
    let message_runs = msg.message.as_ref().and_then(|m| parse_runs(&m.runs));
    let message_text = extract_plain_text(&message_runs);
    let (is_owner, is_moderator, _) = parse_author_badges(&msg.author_badges);
    let published_at = parse_timestamp(&msg.timestamp_usec);

    // メンバーシップレベルを抽出
    let level = msg
        .header_sub_text
        .and_then(|t| t.runs.and_then(|r| r.first().and_then(|i| i.text.clone())))
        .unwrap_or_else(|| "新規メンバー".to_string());

    ChatMessage {
        id: msg.id,
        message: message_text,
        author_name: msg
            .author_name
            .map(|n| n.get_text())
            .unwrap_or_default(),
        author_channel_id: msg.author_external_channel_id.unwrap_or_default(),
        author_image_url: msg
            .author_photo
            .and_then(|p| p.thumbnails.first().map(|t| t.url.clone()))
            .unwrap_or_default(),
        published_at,
        is_owner,
        is_moderator,
        is_member: true,
        is_verified: false,
        message_type: MessageType::Membership { level },
        message_runs,
    }
}

/// メンバーシップギフトをパース
fn parse_gift_message(msg: LiveChatSponsorGiftRenderer) -> ChatMessage {
    let published_at = parse_timestamp(&msg.timestamp_usec);

    // ギフト数を抽出（例: "5件のメンバーシップをギフトしました"）
    let gift_text = msg
        .primary_text
        .and_then(|t| t.runs.and_then(|r| r.first().and_then(|i| i.text.clone())))
        .unwrap_or_default();

    // 数字を抽出
    let count: u32 = gift_text
        .chars()
        .filter(|c| c.is_ascii_digit())
        .collect::<String>()
        .parse()
        .unwrap_or(1);

    ChatMessage {
        id: msg.id,
        message: gift_text,
        author_name: msg
            .author_name
            .map(|n| n.get_text())
            .unwrap_or_default(),
        author_channel_id: msg.author_external_channel_id.unwrap_or_default(),
        author_image_url: msg
            .author_photo
            .and_then(|p| p.thumbnails.first().map(|t| t.url.clone()))
            .unwrap_or_default(),
        published_at,
        is_owner: false,
        is_moderator: false,
        is_member: true,
        is_verified: false,
        message_type: MessageType::MembershipGift { count },
        message_runs: None,
    }
}

/// runs配列をMessageRunリストに変換
fn parse_runs(runs: &Option<Vec<RunItem>>) -> Option<Vec<MessageRun>> {
    let runs = runs.as_ref()?;
    if runs.is_empty() {
        return None;
    }

    let parsed: Vec<MessageRun> = runs
        .iter()
        .filter_map(|run| {
            if let Some(text) = &run.text {
                Some(MessageRun::Text { text: text.clone() })
            } else if let Some(emoji) = &run.emoji {
                Some(MessageRun::Emoji {
                    emoji: EmojiInfo {
                        emoji_id: emoji.emoji_id.clone(),
                        shortcuts: emoji.shortcuts.clone().unwrap_or_default(),
                        image: EmojiImage {
                            thumbnails: emoji
                                .image
                                .thumbnails
                                .iter()
                                .map(|t| EmojiThumbnail {
                                    url: t.url.clone(),
                                    width: t.width.unwrap_or(24),
                                    height: t.height.unwrap_or(24),
                                })
                                .collect(),
                        },
                        is_custom_emoji: emoji.is_custom_emoji.unwrap_or(false),
                    },
                })
            } else {
                None
            }
        })
        .collect();

    if parsed.is_empty() {
        None
    } else {
        Some(parsed)
    }
}

/// MessageRunリストからプレーンテキストを抽出
fn extract_plain_text(runs: &Option<Vec<MessageRun>>) -> String {
    runs.as_ref()
        .map(|runs| {
            runs.iter()
                .map(|run| match run {
                    MessageRun::Text { text } => text.clone(),
                    MessageRun::Emoji { emoji } => emoji
                        .shortcuts
                        .first()
                        .cloned()
                        .unwrap_or_else(|| format!(":{}:", emoji.emoji_id)),
                })
                .collect()
        })
        .unwrap_or_default()
}

/// 投稿者バッジからフラグを判定
fn parse_author_badges(badges: &Option<Vec<AuthorBadge>>) -> (bool, bool, bool) {
    let Some(badges) = badges else {
        return (false, false, false);
    };

    let mut is_owner = false;
    let mut is_moderator = false;
    let mut is_member = false;

    for badge in badges {
        if let Some(renderer) = &badge.live_chat_author_badge_renderer {
            if let Some(icon) = &renderer.icon {
                match icon.icon_type.as_str() {
                    "OWNER" => is_owner = true,
                    "MODERATOR" => is_moderator = true,
                    "VERIFIED" => {} // is_verifiedは別途設定
                    _ => {}
                }
            }
            // カスタムサムネイルがある場合はメンバー
            if renderer.custom_thumbnail.is_some() {
                is_member = true;
            }
        }
    }

    (is_owner, is_moderator, is_member)
}

/// タイムスタンプをパース（マイクロ秒 -> DateTime<Utc>）
fn parse_timestamp(timestamp_usec: &Option<String>) -> chrono::DateTime<Utc> {
    match timestamp_usec {
        Some(ts) => {
            match ts.parse::<i64>() {
                Ok(usec) => {
                    Utc.timestamp_micros(usec).single().unwrap_or_else(|| {
                        log::warn!("Invalid timestamp microseconds: {}", usec);
                        Utc::now()
                    })
                }
                Err(e) => {
                    log::warn!("Failed to parse timestamp '{}': {}", ts, e);
                    Utc::now()
                }
            }
        }
        None => {
            log::debug!("No timestamp provided, using current time");
            Utc::now()
        }
    }
}

/// 金額テキストをパース（例: "¥1,000" -> ("1,000", "JPY")）
fn parse_amount(text: &str) -> (String, String) {
    // 通貨記号を判定
    let currency = if text.starts_with('¥') || text.starts_with("￥") {
        "JPY"
    } else if text.starts_with('$') {
        "USD"
    } else if text.starts_with('€') {
        "EUR"
    } else if text.starts_with('£') {
        "GBP"
    } else {
        "USD"
    };

    // 数字とカンマ、ピリオドのみ抽出
    let amount: String = text
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == ',' || *c == '.')
        .collect();

    (amount, currency.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_amount() {
        let (amount, currency) = parse_amount("¥1,000");
        assert_eq!(amount, "1,000");
        assert_eq!(currency, "JPY");

        let (amount, currency) = parse_amount("$5.00");
        assert_eq!(amount, "5.00");
        assert_eq!(currency, "USD");
    }

    #[test]
    fn test_extract_plain_text() {
        let runs = Some(vec![
            MessageRun::Text {
                text: "Hello ".to_string(),
            },
            MessageRun::Emoji {
                emoji: EmojiInfo {
                    emoji_id: "test".to_string(),
                    shortcuts: vec![":wave:".to_string()],
                    image: EmojiImage { thumbnails: vec![] },
                    is_custom_emoji: false,
                },
            },
            MessageRun::Text {
                text: " World".to_string(),
            },
        ]);

        let text = extract_plain_text(&runs);
        assert_eq!(text, "Hello :wave: World");
    }
}
