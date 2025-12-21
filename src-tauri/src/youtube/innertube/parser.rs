//! InnerTube レスポンスパーサー

use chrono::{TimeZone, Utc};
use std::collections::HashMap;
use std::sync::RwLock;
use once_cell::sync::Lazy;
use regex::Regex;

use super::types::*;
use crate::youtube::types::{ChatMessage, EmojiImage, EmojiInfo, EmojiThumbnail, MessageRun, MessageType};

/// 絵文字キャッシュ: ショートカット -> EmojiInfo
/// InnerTubeレスポンスで取得した絵文字情報をキャッシュし、
/// テキストトークンで送られてきた絵文字ショートカットを画像に変換するために使用
static EMOJI_CACHE: Lazy<RwLock<HashMap<String, EmojiInfo>>> = Lazy::new(|| {
    RwLock::new(HashMap::new())
});

/// 絵文字ショートカットパターン（:_xxx:形式）
static EMOJI_SHORTCUT_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r":_[^:]+:").expect("Failed to compile emoji shortcut regex")
});

/// 絵文字キャッシュをクリア（テスト用・デバッグ用）
#[allow(dead_code)]
pub fn clear_emoji_cache() {
    if let Ok(mut cache) = EMOJI_CACHE.write() {
        cache.clear();
    }
}

/// 絵文字キャッシュのサイズを取得（デバッグ用）
#[allow(dead_code)]
pub fn get_emoji_cache_size() -> usize {
    EMOJI_CACHE.read().map(|c| c.len()).unwrap_or(0)
}

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

    // flat_mapを使用してparse_actionが返す複数メッセージを統合
    actions
        .into_iter()
        .flat_map(parse_action)
        .collect()
}

/// 単一のアクションをパース（複数メッセージを返す可能性あり）
///
/// リプレイアクションには複数のメッセージが含まれる場合があるため、
/// Vec<ChatMessage>を返す設計に変更。
fn parse_action(action: ChatAction) -> Vec<ChatMessage> {
    // 通常のメッセージ追加
    if let Some(add_action) = action.add_chat_item_action {
        if let Some(msg) = parse_chat_item(add_action.item) {
            return vec![msg];
        }
        return vec![];
    }

    // リプレイアクション（アーカイブ視聴時）
    // 複数のadd_chat_item_actionが含まれる場合、すべてを処理
    if let Some(replay_action) = action.replay_chat_item_action {
        if let Some(actions) = replay_action.actions {
            let messages: Vec<ChatMessage> = actions
                .into_iter()
                .filter_map(|inner_action| {
                    inner_action.add_chat_item_action
                        .and_then(|add_action| parse_chat_item(add_action.item))
                })
                .collect();
            return messages;
        }
    }

    vec![]
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
/// 
/// 絵文字キャッシュ機能:
/// 1. 絵文字オブジェクトを受信したらショートカット→EmojiInfoをキャッシュ
/// 2. テキストトークン内の:_xxx:パターンをキャッシュから画像に変換
fn parse_runs(runs: &Option<Vec<RunItem>>) -> Option<Vec<MessageRun>> {
    let runs = runs.as_ref()?;
    if runs.is_empty() {
        return None;
    }

    let mut parsed: Vec<MessageRun> = Vec::new();

    for run in runs {
        if let Some(emoji) = &run.emoji {
            // 空のemoji_idは無効なのでスキップ
            if emoji.emoji_id.is_empty() {
                log::debug!("Skipping emoji with empty emoji_id");
                continue;
            }

            let emoji_info = EmojiInfo {
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
            };

            // キャッシュに追加（ショートカットごとに登録）
            if let Ok(mut cache) = EMOJI_CACHE.write() {
                for shortcut in &emoji_info.shortcuts {
                    if !cache.contains_key(shortcut) {
                        cache.insert(shortcut.clone(), emoji_info.clone());
                        log::debug!("Cached emoji: {}", shortcut);
                    }
                }
            }

            parsed.push(MessageRun::Emoji { emoji: emoji_info });
        } else if let Some(text) = &run.text {
            // テキストトークン内の:_xxx:パターンをキャッシュから画像に変換
            let converted = convert_text_with_emoji_cache(text);
            parsed.extend(converted);
        }
    }

    if parsed.is_empty() {
        None
    } else {
        Some(parsed)
    }
}

/// テキスト内の:_xxx:パターンを絵文字キャッシュから画像に変換
/// 
/// 例: "こんにちは:_草lol:です" → [Text("こんにちは"), Emoji(...), Text("です")]
fn convert_text_with_emoji_cache(text: &str) -> Vec<MessageRun> {
    let cache = match EMOJI_CACHE.read() {
        Ok(c) => c,
        Err(_) => return vec![MessageRun::Text { text: text.to_string() }],
    };

    // キャッシュが空ならそのままテキストを返す
    if cache.is_empty() {
        return vec![MessageRun::Text { text: text.to_string() }];
    }

    let mut result: Vec<MessageRun> = Vec::new();
    let mut last_end = 0;

    for mat in EMOJI_SHORTCUT_REGEX.find_iter(text) {
        let shortcut = mat.as_str();

        // マッチ前のテキストを追加
        if mat.start() > last_end {
            let prefix = &text[last_end..mat.start()];
            if !prefix.is_empty() {
                result.push(MessageRun::Text { text: prefix.to_string() });
            }
        }

        // キャッシュに絵文字があれば画像に変換、なければテキストのまま
        if let Some(emoji_info) = cache.get(shortcut) {
            result.push(MessageRun::Emoji { emoji: emoji_info.clone() });
            log::debug!("Converted text emoji from cache: {}", shortcut);
        } else {
            result.push(MessageRun::Text { text: shortcut.to_string() });
        }

        last_end = mat.end();
    }

    // 残りのテキストを追加
    if last_end < text.len() {
        let suffix = &text[last_end..];
        if !suffix.is_empty() {
            result.push(MessageRun::Text { text: suffix.to_string() });
        }
    }

    // マッチがなかった場合は元のテキストをそのまま返す
    if result.is_empty() {
        result.push(MessageRun::Text { text: text.to_string() });
    }

    result
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
/// 頻繁に呼ばれるため、パース失敗時のログはdebugレベル
fn parse_timestamp(timestamp_usec: &Option<String>) -> chrono::DateTime<Utc> {
    match timestamp_usec {
        Some(ts) => {
            match ts.parse::<i64>() {
                Ok(usec) => {
                    Utc.timestamp_micros(usec).single().unwrap_or_else(|| {
                        log::debug!("Invalid timestamp microseconds: {}", usec);
                        Utc::now()
                    })
                }
                Err(e) => {
                    log::debug!("Failed to parse timestamp '{}': {}", ts, e);
                    Utc::now()
                }
            }
        }
        None => Utc::now(),
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
    use crate::youtube::innertube::types::*;

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
    fn test_parse_amount_euro_and_gbp() {
        let (amount, currency) = parse_amount("€10.00");
        assert_eq!(amount, "10.00");
        assert_eq!(currency, "EUR");

        let (amount, currency) = parse_amount("£20.00");
        assert_eq!(amount, "20.00");
        assert_eq!(currency, "GBP");
    }

    #[test]
    fn test_parse_amount_unknown_currency() {
        // 不明な通貨記号はUSDにフォールバック
        let (amount, currency) = parse_amount("₩1000");
        assert_eq!(amount, "1000");
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

    #[test]
    fn test_extract_plain_text_empty() {
        let runs: Option<Vec<MessageRun>> = None;
        let text = extract_plain_text(&runs);
        assert_eq!(text, "");
    }

    // ========================================
    // parse_chat_response 異常系テスト
    // ========================================

    #[test]
    fn test_parse_chat_response_empty() {
        // continuation_contentsがNoneの場合
        let response = InnerTubeChatResponse {
            continuation_contents: None,
        };
        let messages = parse_chat_response(response);
        assert!(messages.is_empty());
    }

    #[test]
    fn test_parse_chat_response_no_continuation() {
        // live_chat_continuationがNoneの場合
        let response = InnerTubeChatResponse {
            continuation_contents: Some(ContinuationContents {
                live_chat_continuation: None,
            }),
        };
        let messages = parse_chat_response(response);
        assert!(messages.is_empty());
    }

    #[test]
    fn test_parse_chat_response_no_actions() {
        // actionsがNoneの場合
        let response = InnerTubeChatResponse {
            continuation_contents: Some(ContinuationContents {
                live_chat_continuation: Some(LiveChatContinuation {
                    actions: None,
                    continuations: None,
                }),
            }),
        };
        let messages = parse_chat_response(response);
        assert!(messages.is_empty());
    }

    #[test]
    fn test_parse_chat_response_empty_actions() {
        // actionsが空配列の場合
        let response = InnerTubeChatResponse {
            continuation_contents: Some(ContinuationContents {
                live_chat_continuation: Some(LiveChatContinuation {
                    actions: Some(vec![]),
                    continuations: None,
                }),
            }),
        };
        let messages = parse_chat_response(response);
        assert!(messages.is_empty());
    }

    #[test]
    fn test_parse_chat_response_valid_text_message() {
        // 正常なテキストメッセージ
        let response = InnerTubeChatResponse {
            continuation_contents: Some(ContinuationContents {
                live_chat_continuation: Some(LiveChatContinuation {
                    actions: Some(vec![ChatAction {
                        add_chat_item_action: Some(AddChatItemAction {
                            item: ChatItem {
                                live_chat_text_message_renderer: Some(LiveChatTextMessageRenderer {
                                    id: "test-id".to_string(),
                                    message: Some(MessageContent {
                                        runs: Some(vec![RunItem {
                                            text: Some("Hello World".to_string()),
                                            emoji: None,
                                        }]),
                                    }),
                                    author_name: Some(SimpleText {
                                        simple_text: Some("Test User".to_string()),
                                        runs: None,
                                    }),
                                    author_photo: None,
                                    author_external_channel_id: Some("channel-123".to_string()),
                                    timestamp_usec: Some("1703145600000000".to_string()),
                                    author_badges: None,
                                }),
                                live_chat_paid_message_renderer: None,
                                live_chat_paid_sticker_renderer: None,
                                live_chat_membership_item_renderer: None,
                                live_chat_sponsor_gift_announcement_renderer: None,
                            },
                        }),
                        replay_chat_item_action: None,
                    }]),
                    continuations: None,
                }),
            }),
        };

        let messages = parse_chat_response(response);
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].id, "test-id");
        assert_eq!(messages[0].message, "Hello World");
        assert_eq!(messages[0].author_name, "Test User");
    }

    // ========================================
    // parse_author_badges テスト
    // ========================================

    #[test]
    fn test_parse_author_badges_none() {
        let (is_owner, is_moderator, is_member) = parse_author_badges(&None);
        assert!(!is_owner);
        assert!(!is_moderator);
        assert!(!is_member);
    }

    #[test]
    fn test_parse_author_badges_empty() {
        let badges: Vec<AuthorBadge> = vec![];
        let (is_owner, is_moderator, is_member) = parse_author_badges(&Some(badges));
        assert!(!is_owner);
        assert!(!is_moderator);
        assert!(!is_member);
    }

    #[test]
    fn test_parse_author_badges_owner() {
        let badges = vec![AuthorBadge {
            live_chat_author_badge_renderer: Some(BadgeRenderer {
                custom_thumbnail: None,
                icon: Some(BadgeIcon {
                    icon_type: "OWNER".to_string(),
                }),
                tooltip: None,
            }),
        }];
        let (is_owner, is_moderator, is_member) = parse_author_badges(&Some(badges));
        assert!(is_owner);
        assert!(!is_moderator);
        assert!(!is_member);
    }

    #[test]
    fn test_parse_author_badges_moderator() {
        let badges = vec![AuthorBadge {
            live_chat_author_badge_renderer: Some(BadgeRenderer {
                custom_thumbnail: None,
                icon: Some(BadgeIcon {
                    icon_type: "MODERATOR".to_string(),
                }),
                tooltip: None,
            }),
        }];
        let (is_owner, is_moderator, is_member) = parse_author_badges(&Some(badges));
        assert!(!is_owner);
        assert!(is_moderator);
        assert!(!is_member);
    }

    #[test]
    fn test_parse_author_badges_member() {
        let badges = vec![AuthorBadge {
            live_chat_author_badge_renderer: Some(BadgeRenderer {
                custom_thumbnail: Some(ThumbnailContainer {
                    thumbnails: vec![Thumbnail {
                        url: "https://example.com/badge.png".to_string(),
                        width: Some(16),
                        height: Some(16),
                    }],
                }),
                icon: None,
                tooltip: Some("メンバー（1か月）".to_string()),
            }),
        }];
        let (is_owner, is_moderator, is_member) = parse_author_badges(&Some(badges));
        assert!(!is_owner);
        assert!(!is_moderator);
        assert!(is_member);
    }

    #[test]
    fn test_parse_author_badges_multiple() {
        // オーナー + モデレーター + メンバーが同時に存在するケース
        let badges = vec![
            AuthorBadge {
                live_chat_author_badge_renderer: Some(BadgeRenderer {
                    custom_thumbnail: None,
                    icon: Some(BadgeIcon {
                        icon_type: "OWNER".to_string(),
                    }),
                    tooltip: None,
                }),
            },
            AuthorBadge {
                live_chat_author_badge_renderer: Some(BadgeRenderer {
                    custom_thumbnail: None,
                    icon: Some(BadgeIcon {
                        icon_type: "MODERATOR".to_string(),
                    }),
                    tooltip: None,
                }),
            },
            AuthorBadge {
                live_chat_author_badge_renderer: Some(BadgeRenderer {
                    custom_thumbnail: Some(ThumbnailContainer {
                        thumbnails: vec![Thumbnail {
                            url: "https://example.com/member.png".to_string(),
                            width: Some(16),
                            height: Some(16),
                        }],
                    }),
                    icon: None,
                    tooltip: Some("メンバー（12か月）".to_string()),
                }),
            },
        ];
        let (is_owner, is_moderator, is_member) = parse_author_badges(&Some(badges));
        assert!(is_owner);
        assert!(is_moderator);
        assert!(is_member);
    }

    #[test]
    fn test_parse_author_badges_verified_not_owner() {
        // VERIFIEDはis_ownerにならない
        let badges = vec![AuthorBadge {
            live_chat_author_badge_renderer: Some(BadgeRenderer {
                custom_thumbnail: None,
                icon: Some(BadgeIcon {
                    icon_type: "VERIFIED".to_string(),
                }),
                tooltip: None,
            }),
        }];
        let (is_owner, is_moderator, is_member) = parse_author_badges(&Some(badges));
        assert!(!is_owner);
        assert!(!is_moderator);
        assert!(!is_member);
    }

    #[test]
    fn test_parse_author_badges_unknown_type() {
        // 不明なバッジタイプは無視される
        let badges = vec![AuthorBadge {
            live_chat_author_badge_renderer: Some(BadgeRenderer {
                custom_thumbnail: None,
                icon: Some(BadgeIcon {
                    icon_type: "UNKNOWN_NEW_TYPE".to_string(),
                }),
                tooltip: None,
            }),
        }];
        let (is_owner, is_moderator, is_member) = parse_author_badges(&Some(badges));
        assert!(!is_owner);
        assert!(!is_moderator);
        assert!(!is_member);
    }

    // ========================================
    // parse_timestamp テスト
    // ========================================

    #[test]
    fn test_parse_timestamp_valid() {
        let ts = parse_timestamp(&Some("1703145600000000".to_string()));
        // 2023-12-21 08:00:00 UTC
        assert_eq!(ts.timestamp_micros(), 1703145600000000);
    }

    #[test]
    fn test_parse_timestamp_none() {
        let ts = parse_timestamp(&None);
        // Noneの場合は現在時刻（テストでは具体的な値を検証せず、エラーなく動作することを確認）
        assert!(ts.timestamp() > 0);
    }

    #[test]
    fn test_parse_timestamp_invalid() {
        let ts = parse_timestamp(&Some("invalid".to_string()));
        // 無効な場合は現在時刻
        assert!(ts.timestamp() > 0);
    }

    // ========================================
    // parse_runs テスト
    // ========================================

    #[test]
    fn test_parse_runs_none() {
        let result = parse_runs(&None);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_runs_empty() {
        let runs: Vec<RunItem> = vec![];
        let result = parse_runs(&Some(runs));
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_runs_with_emoji() {
        let runs = vec![
            RunItem {
                text: Some("Hello ".to_string()),
                emoji: None,
            },
            RunItem {
                text: None,
                emoji: Some(InnerTubeEmoji {
                    emoji_id: "UC123/custom_emoji".to_string(),
                    shortcuts: Some(vec![":custom:".to_string()]),
                    search_terms: None,
                    image: ThumbnailContainer {
                        thumbnails: vec![Thumbnail {
                            url: "https://example.com/emoji.png".to_string(),
                            width: Some(24),
                            height: Some(24),
                        }],
                    },
                    is_custom_emoji: Some(true),
                }),
            },
        ];
        let result = parse_runs(&Some(runs));
        assert!(result.is_some());
        let parsed = result.unwrap();
        assert_eq!(parsed.len(), 2);

        match &parsed[0] {
            MessageRun::Text { text } => assert_eq!(text, "Hello "),
            _ => panic!("Expected Text run"),
        }

        match &parsed[1] {
            MessageRun::Emoji { emoji } => {
                assert_eq!(emoji.emoji_id, "UC123/custom_emoji");
                assert!(emoji.is_custom_emoji);
            }
            _ => panic!("Expected Emoji run"),
        }
    }

    #[test]
    fn test_parse_runs_skip_empty_emoji_id() {
        // 空のemoji_idはスキップされる
        let runs = vec![RunItem {
            text: None,
            emoji: Some(InnerTubeEmoji {
                emoji_id: "".to_string(), // 空のID
                shortcuts: Some(vec![":empty:".to_string()]),
                search_terms: None,
                image: ThumbnailContainer { thumbnails: vec![] },
                is_custom_emoji: Some(false),
            }),
        }];
        let result = parse_runs(&Some(runs));
        assert!(result.is_none()); // 空なのでNone
    }

    // ========================================
    // replay_chat_item_action 複数アクションテスト
    // ========================================

    /// リプレイアクション内の複数メッセージが正しく処理されることを確認
    #[test]
    fn test_parse_action_replay_multiple_messages() {
        // リプレイアクション内に複数のadd_chat_item_actionを含むケース
        let replay_action = ChatAction {
            add_chat_item_action: None,
            replay_chat_item_action: Some(ReplayChatItemAction {
                actions: Some(vec![
                    // 1つ目のメッセージ
                    ChatAction {
                        add_chat_item_action: Some(AddChatItemAction {
                            item: ChatItem {
                                live_chat_text_message_renderer: Some(LiveChatTextMessageRenderer {
                                    id: "replay-msg-1".to_string(),
                                    message: Some(MessageContent {
                                        runs: Some(vec![RunItem {
                                            text: Some("First message".to_string()),
                                            emoji: None,
                                        }]),
                                    }),
                                    author_name: Some(SimpleText {
                                        simple_text: Some("User1".to_string()),
                                        runs: None,
                                    }),
                                    author_photo: None,
                                    author_external_channel_id: Some("channel-1".to_string()),
                                    timestamp_usec: Some("1703145600000000".to_string()),
                                    author_badges: None,
                                }),
                                live_chat_paid_message_renderer: None,
                                live_chat_paid_sticker_renderer: None,
                                live_chat_membership_item_renderer: None,
                                live_chat_sponsor_gift_announcement_renderer: None,
                            },
                        }),
                        replay_chat_item_action: None,
                    },
                    // 2つ目のメッセージ
                    ChatAction {
                        add_chat_item_action: Some(AddChatItemAction {
                            item: ChatItem {
                                live_chat_text_message_renderer: Some(LiveChatTextMessageRenderer {
                                    id: "replay-msg-2".to_string(),
                                    message: Some(MessageContent {
                                        runs: Some(vec![RunItem {
                                            text: Some("Second message".to_string()),
                                            emoji: None,
                                        }]),
                                    }),
                                    author_name: Some(SimpleText {
                                        simple_text: Some("User2".to_string()),
                                        runs: None,
                                    }),
                                    author_photo: None,
                                    author_external_channel_id: Some("channel-2".to_string()),
                                    timestamp_usec: Some("1703145601000000".to_string()),
                                    author_badges: None,
                                }),
                                live_chat_paid_message_renderer: None,
                                live_chat_paid_sticker_renderer: None,
                                live_chat_membership_item_renderer: None,
                                live_chat_sponsor_gift_announcement_renderer: None,
                            },
                        }),
                        replay_chat_item_action: None,
                    },
                ]),
            }),
        };

        let messages = parse_action(replay_action);

        // 2つのメッセージが返されるべき
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].id, "replay-msg-1");
        assert_eq!(messages[0].message, "First message");
        assert_eq!(messages[1].id, "replay-msg-2");
        assert_eq!(messages[1].message, "Second message");
    }

    /// 空のリプレイアクションは空のVecを返す
    #[test]
    fn test_parse_action_replay_empty() {
        let replay_action = ChatAction {
            add_chat_item_action: None,
            replay_chat_item_action: Some(ReplayChatItemAction { actions: None }),
        };

        let messages = parse_action(replay_action);
        assert!(messages.is_empty());
    }

    /// 通常のadd_chat_item_actionは1メッセージを返す
    #[test]
    fn test_parse_action_single_message() {
        let action = ChatAction {
            add_chat_item_action: Some(AddChatItemAction {
                item: ChatItem {
                    live_chat_text_message_renderer: Some(LiveChatTextMessageRenderer {
                        id: "single-msg".to_string(),
                        message: Some(MessageContent {
                            runs: Some(vec![RunItem {
                                text: Some("Single message".to_string()),
                                emoji: None,
                            }]),
                        }),
                        author_name: Some(SimpleText {
                            simple_text: Some("User".to_string()),
                            runs: None,
                        }),
                        author_photo: None,
                        author_external_channel_id: Some("channel".to_string()),
                        timestamp_usec: Some("1703145600000000".to_string()),
                        author_badges: None,
                    }),
                    live_chat_paid_message_renderer: None,
                    live_chat_paid_sticker_renderer: None,
                    live_chat_membership_item_renderer: None,
                    live_chat_sponsor_gift_announcement_renderer: None,
                },
            }),
            replay_chat_item_action: None,
        };

        let messages = parse_action(action);
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].id, "single-msg");
    }
}

