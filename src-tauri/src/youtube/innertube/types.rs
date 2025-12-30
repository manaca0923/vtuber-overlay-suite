//! InnerTube API 固有の型定義

use serde::{Deserialize, Serialize};

/// Continuation種別（ポーリング間隔の制御に使用）
///
/// InnerTube APIは3種類のContinuationデータを返す:
/// - `invalidationContinuationData`: timeout_msはOptional、推奨間隔（短縮可能）
/// - `timedContinuationData`: timeout_msは必須、明示的な待機時間（厳守）
/// - `reloadContinuationData/replay`: 初期化・リプレイ用
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContinuationType {
    /// invalidationContinuationData - 推奨間隔（短縮可能）
    Invalidation,
    /// timedContinuationData - 明示的な待機時間（厳守）
    Timed,
    /// reloadContinuationData/replay - 初期化・リプレイ用
    Reload,
}

/// ポーリング間隔の最大値（30秒）
/// 極端に大きな値が返された場合のガード
const MAX_POLLING_INTERVAL_MS: u64 = 30000;

/// ポーリング間隔の最小値（500ms）
/// 極端に短い値によるサーバー過負荷を防止
const MIN_POLLING_INTERVAL_MS: u64 = 500;

/// InnerTubeモードのバッファ間隔（1秒）
/// オーバーレイ側で1秒間に取得したコメントを等間隔で表示するための設定
/// ポーリング間隔（1秒）と合わせることで、リアルタイムに近い表示を実現
pub const INNERTUBE_BUFFER_INTERVAL_MS: u32 = 1000;

impl ContinuationType {
    /// 実効的なポーリング間隔を計算
    ///
    /// Continuation種別に応じて適切なポーリング間隔を返す:
    /// - `Invalidation`: 1秒固定（リアルタイム表示のため高頻度ポーリング）
    /// - `Timed`: APIの値を使用（500ms〜30秒でガード）
    /// - `Reload`: 1秒固定（初期化後は即座にポーリング）
    pub fn effective_timeout_ms(&self, api_timeout: u64) -> u64 {
        match self {
            ContinuationType::Invalidation => 1000, // 1秒固定（リアルタイム表示のため）
            ContinuationType::Timed => api_timeout.clamp(MIN_POLLING_INTERVAL_MS, MAX_POLLING_INTERVAL_MS),
            ContinuationType::Reload => 1000,
        }
    }
}

/// InnerTube APIレスポンス（ライブチャット取得）
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InnerTubeChatResponse {
    pub continuation_contents: Option<ContinuationContents>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContinuationContents {
    pub live_chat_continuation: Option<LiveChatContinuation>,
}

#[derive(Debug, Deserialize)]
pub struct LiveChatContinuation {
    pub actions: Option<Vec<ChatAction>>,
    pub continuations: Option<Vec<Continuation>>,
}

/// チャットアクション（メッセージ追加など）
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatAction {
    pub add_chat_item_action: Option<AddChatItemAction>,
    pub replay_chat_item_action: Option<ReplayChatItemAction>,
}

#[derive(Debug, Deserialize)]
pub struct AddChatItemAction {
    pub item: ChatItem,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayChatItemAction {
    pub actions: Option<Vec<ChatAction>>,
}

/// チャットアイテム（各種メッセージレンダラーを含む）
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatItem {
    pub live_chat_text_message_renderer: Option<LiveChatTextMessageRenderer>,
    pub live_chat_paid_message_renderer: Option<LiveChatPaidMessageRenderer>,
    pub live_chat_paid_sticker_renderer: Option<LiveChatPaidStickerRenderer>,
    pub live_chat_membership_item_renderer: Option<LiveChatMembershipItemRenderer>,
    pub live_chat_sponsor_gift_announcement_renderer: Option<LiveChatSponsorGiftRenderer>,
}

/// テキストメッセージレンダラー
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveChatTextMessageRenderer {
    pub id: String,
    pub message: Option<MessageContent>,
    pub author_name: Option<SimpleText>,
    pub author_photo: Option<ThumbnailContainer>,
    pub author_external_channel_id: Option<String>,
    pub timestamp_usec: Option<String>,
    pub author_badges: Option<Vec<AuthorBadge>>,
}

/// スーパーチャットレンダラー
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveChatPaidMessageRenderer {
    pub id: String,
    pub message: Option<MessageContent>,
    pub author_name: Option<SimpleText>,
    pub author_photo: Option<ThumbnailContainer>,
    pub author_external_channel_id: Option<String>,
    pub timestamp_usec: Option<String>,
    pub author_badges: Option<Vec<AuthorBadge>>,
    pub purchase_amount_text: Option<SimpleText>,
    pub header_background_color: Option<i64>,
    pub header_text_color: Option<i64>,
    pub body_background_color: Option<i64>,
    pub body_text_color: Option<i64>,
}

/// スーパーステッカーレンダラー
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveChatPaidStickerRenderer {
    pub id: String,
    pub author_name: Option<SimpleText>,
    pub author_photo: Option<ThumbnailContainer>,
    pub author_external_channel_id: Option<String>,
    pub timestamp_usec: Option<String>,
    pub author_badges: Option<Vec<AuthorBadge>>,
    pub purchase_amount_text: Option<SimpleText>,
    pub sticker: Option<ThumbnailContainer>,
    pub sticker_display_width: Option<i32>,
    pub sticker_display_height: Option<i32>,
}

/// メンバーシップレンダラー
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveChatMembershipItemRenderer {
    pub id: String,
    pub message: Option<MessageContent>,
    pub author_name: Option<SimpleText>,
    pub author_photo: Option<ThumbnailContainer>,
    pub author_external_channel_id: Option<String>,
    pub timestamp_usec: Option<String>,
    pub author_badges: Option<Vec<AuthorBadge>>,
    pub header_sub_text: Option<MessageContent>,
}

/// メンバーシップギフトレンダラー
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveChatSponsorGiftRenderer {
    pub id: String,
    pub author_name: Option<SimpleText>,
    pub author_photo: Option<ThumbnailContainer>,
    pub author_external_channel_id: Option<String>,
    pub timestamp_usec: Option<String>,
    pub primary_text: Option<MessageContent>,
}

/// メッセージ内容（runs配列を含む）
#[derive(Debug, Deserialize)]
pub struct MessageContent {
    pub runs: Option<Vec<RunItem>>,
}

/// runs配列の要素（テキストまたは絵文字）
#[derive(Debug, Deserialize)]
pub struct RunItem {
    pub text: Option<String>,
    pub emoji: Option<InnerTubeEmoji>,
}

/// InnerTube絵文字情報
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InnerTubeEmoji {
    pub emoji_id: String,
    pub shortcuts: Option<Vec<String>>,
    pub search_terms: Option<Vec<String>>,
    pub image: ThumbnailContainer,
    pub is_custom_emoji: Option<bool>,
}

/// サムネイルコンテナ
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ThumbnailContainer {
    pub thumbnails: Vec<Thumbnail>,
}

/// サムネイル
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Thumbnail {
    pub url: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

/// シンプルテキスト
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimpleText {
    pub simple_text: Option<String>,
    pub runs: Option<Vec<RunItem>>,
}

impl SimpleText {
    /// テキスト内容を取得
    pub fn get_text(&self) -> String {
        if let Some(text) = &self.simple_text {
            return text.clone();
        }
        if let Some(runs) = &self.runs {
            return runs
                .iter()
                .filter_map(|r| r.text.as_ref())
                .cloned()
                .collect::<Vec<_>>()
                .join("");
        }
        String::new()
    }
}

/// Continuation（次回取得用トークン）
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Continuation {
    pub invalidation_continuation_data: Option<InvalidationContinuationData>,
    pub timed_continuation_data: Option<TimedContinuationData>,
    pub live_chat_replay_continuation_data: Option<LiveChatReplayContinuationData>,
}

#[derive(Debug, Deserialize)]
pub struct InvalidationContinuationData {
    pub continuation: String,
    #[serde(rename = "timeoutMs")]
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimedContinuationData {
    pub continuation: String,
    pub timeout_ms: u64,
}

#[derive(Debug, Deserialize)]
pub struct LiveChatReplayContinuationData {
    pub continuation: String,
    #[serde(rename = "timeUntilLastMessageMsec")]
    pub time_until_last_message_msec: Option<u64>,
}

/// 投稿者バッジ
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorBadge {
    pub live_chat_author_badge_renderer: Option<BadgeRenderer>,
}

/// バッジレンダラー
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BadgeRenderer {
    pub custom_thumbnail: Option<ThumbnailContainer>,
    pub icon: Option<BadgeIcon>,
    pub tooltip: Option<String>,
}

/// バッジアイコン
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BadgeIcon {
    pub icon_type: String,
}

impl InnerTubeChatResponse {
    /// 次回取得用のcontinuationトークンを抽出
    ///
    /// # Returns
    /// - `(continuation_token, timeout_ms, continuation_type)` のタプル
    /// - `continuation_type` はポーリング間隔の制御に使用
    pub fn get_next_continuation(&self) -> Option<(String, u64, ContinuationType)> {
        let continuation = self
            .continuation_contents
            .as_ref()?
            .live_chat_continuation
            .as_ref()?
            .continuations
            .as_ref()?
            .first()?;

        // 優先順位: invalidation > timed > replay
        if let Some(data) = &continuation.invalidation_continuation_data {
            return Some((
                data.continuation.clone(),
                data.timeout_ms.unwrap_or(5000),
                ContinuationType::Invalidation,
            ));
        }
        if let Some(data) = &continuation.timed_continuation_data {
            return Some((
                data.continuation.clone(),
                data.timeout_ms,
                ContinuationType::Timed,
            ));
        }
        if let Some(data) = &continuation.live_chat_replay_continuation_data {
            return Some((
                data.continuation.clone(),
                data.time_until_last_message_msec.unwrap_or(5000),
                ContinuationType::Reload,
            ));
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effective_timeout_ms_invalidation() {
        let ct = ContinuationType::Invalidation;
        // リアルタイム表示のため、常に1秒固定
        assert_eq!(ct.effective_timeout_ms(0), 1000);
        assert_eq!(ct.effective_timeout_ms(500), 1000);
        assert_eq!(ct.effective_timeout_ms(1000), 1000);
        assert_eq!(ct.effective_timeout_ms(3000), 1000);
        assert_eq!(ct.effective_timeout_ms(5000), 1000);
        assert_eq!(ct.effective_timeout_ms(10000), 1000);
        assert_eq!(ct.effective_timeout_ms(u64::MAX), 1000);
    }

    #[test]
    fn test_effective_timeout_ms_timed() {
        let ct = ContinuationType::Timed;
        // 下限ガード（500ms未満は500msに）
        assert_eq!(ct.effective_timeout_ms(0), 500);
        assert_eq!(ct.effective_timeout_ms(100), 500);
        assert_eq!(ct.effective_timeout_ms(499), 500);
        // 範囲内はそのまま
        assert_eq!(ct.effective_timeout_ms(500), 500);
        assert_eq!(ct.effective_timeout_ms(1000), 1000);
        assert_eq!(ct.effective_timeout_ms(5000), 5000);
        assert_eq!(ct.effective_timeout_ms(29999), 29999);
        // 最大値ガード（30000ms）
        assert_eq!(ct.effective_timeout_ms(30000), 30000);
        assert_eq!(ct.effective_timeout_ms(30001), 30000);
        assert_eq!(ct.effective_timeout_ms(60000), 30000);
        assert_eq!(ct.effective_timeout_ms(u64::MAX), 30000);
    }

    #[test]
    fn test_effective_timeout_ms_reload() {
        let ct = ContinuationType::Reload;
        // 常に1000ms固定
        assert_eq!(ct.effective_timeout_ms(0), 1000);
        assert_eq!(ct.effective_timeout_ms(1000), 1000);
        assert_eq!(ct.effective_timeout_ms(5000), 1000);
        assert_eq!(ct.effective_timeout_ms(99999), 1000);
        assert_eq!(ct.effective_timeout_ms(u64::MAX), 1000);
    }
}
