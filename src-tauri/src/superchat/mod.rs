//! スパチャ専用ウィジェット管理モジュール
//!
//! コメント欄とは別に、スパチャを専用ウィジェット(left.lowerスロット)で
//! 目立たせて表示するための管理機能を提供する。
//!
//! ## 機能
//! - 金額からTier(1-7)を判定
//! - Tierに基づく表示時間の計算
//! - スパチャキューの管理
//! - 表示完了時のremoveメッセージ送信

use crate::server::types::{SuperchatPayload, SuperchatRemovePayload, WsMessage};
use crate::server::websocket::WebSocketState;
use crate::youtube::types::{ChatMessage, MessageType};
use std::sync::Arc;
use tokio::sync::RwLock;

/// 通貨別の日本円換算レート
/// TODO: 将来的に為替レートAPIから取得することを検討
const EXCHANGE_RATES: &[(&str, f64)] = &[
    ("JPY", 1.0),
    ("USD", 150.0),
    ("CAD", 110.0),
    ("AUD", 100.0),
    ("EUR", 160.0),
    ("GBP", 190.0),
    ("KRW", 0.11),
    ("TWD", 4.7),
];

/// Tier判定の閾値（日本円換算）
/// YouTube公式の金額帯に準拠
const TIER_THRESHOLDS: &[(u64, u8)] = &[
    (10_000, 7), // ¥10,000+ → Tier 7 (Red)
    (5_000, 6),  // ¥5,000-9,999 → Tier 6 (Pink)
    (2_000, 5),  // ¥2,000-4,999 → Tier 5 (Orange)
    (1_000, 4),  // ¥1,000-1,999 → Tier 4 (Yellow)
    (500, 3),    // ¥500-999 → Tier 3 (Teal)
    (200, 2),    // ¥200-499 → Tier 2 (Cyan)
    (0, 1),      // ¥100-199 → Tier 1 (Blue)
];

/// Tier別の表示時間（ミリ秒）
/// 高額スパチャほど長く表示
const TIER_DISPLAY_DURATIONS: &[(u8, u64)] = &[
    (7, 300_000), // Tier 7: 5分
    (6, 180_000), // Tier 6: 3分
    (5, 120_000), // Tier 5: 2分
    (4, 60_000),  // Tier 4: 1分
    (3, 30_000),  // Tier 3: 30秒
    (2, 20_000),  // Tier 2: 20秒
    (1, 10_000),  // Tier 1: 10秒
];

/// 金額をマイクロ単位から通常単位に変換
fn micros_to_amount(micros: u64) -> f64 {
    micros as f64 / 1_000_000.0
}

/// 通貨コードから為替レートを取得
fn get_exchange_rate(currency: &str) -> f64 {
    EXCHANGE_RATES
        .iter()
        .find(|(c, _)| *c == currency)
        .map(|(_, rate)| *rate)
        .unwrap_or(1.0) // 不明な通貨はそのまま
}

/// 金額（マイクロ単位）と通貨から日本円換算額を計算
pub fn convert_to_jpy(amount_micros: u64, currency: &str) -> u64 {
    let amount = micros_to_amount(amount_micros);
    let rate = get_exchange_rate(currency);
    (amount * rate) as u64
}

/// 日本円換算額からTierを判定
pub fn calculate_tier(jpy_amount: u64) -> u8 {
    for &(threshold, tier) in TIER_THRESHOLDS {
        if jpy_amount >= threshold {
            return tier;
        }
    }
    1 // 最低Tier
}

/// TierからWebSocketメッセージの表示時間を取得
pub fn get_display_duration(tier: u8) -> u64 {
    TIER_DISPLAY_DURATIONS
        .iter()
        .find(|(t, _)| *t == tier)
        .map(|(_, duration)| *duration)
        .unwrap_or(10_000) // デフォルト10秒
}

/// ChatMessageからSuperchatPayloadを生成
/// スパチャでない場合はNoneを返す
pub fn create_superchat_payload(message: &ChatMessage) -> Option<SuperchatPayload> {
    match &message.message_type {
        MessageType::SuperChat { amount, currency } => {
            // 金額文字列からマイクロ単位を推定
            // NOTE: YouTube APIからはamount_microsが取得できるが、
            // ChatMessage型には含まれていないため、表示文字列からパース
            let amount_micros = parse_amount_micros(amount);
            let jpy_amount = convert_to_jpy(amount_micros, currency);
            let tier = calculate_tier(jpy_amount);
            let display_duration_ms = get_display_duration(tier);

            Some(SuperchatPayload {
                id: message.id.clone(),
                author_name: message.author_name.clone(),
                author_image_url: message.author_image_url.clone(),
                amount: amount.clone(),
                amount_micros,
                currency: currency.clone(),
                message: message.message.clone(),
                tier,
                display_duration_ms,
            })
        }
        _ => None,
    }
}

/// 金額表示文字列からマイクロ単位の金額を推定
/// 例: "¥1,000" → 1_000_000_000
///
/// ## エッジケース
/// - 空文字列や通貨記号のみの場合は 0 を返す（Tier 1扱い）
/// - 複数の通貨記号（例: "A$100.00"）も正しく処理される
/// - パース失敗時はwarnログを出力して 0 を返す
fn parse_amount_micros(amount_str: &str) -> u64 {
    // 数字とピリオド、カンマのみを抽出
    let digits: String = amount_str
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '.' || *c == ',')
        .collect();

    // 欧州形式の判定: カンマの後に1-2桁の数字で終わる（例: "5,00", "1.000,50"）
    let has_comma_decimal = if let Some(comma_pos) = digits.rfind(',') {
        let after_comma = &digits[comma_pos + 1..];
        // カンマの後に1-2桁の数字のみ
        !after_comma.is_empty()
            && after_comma.len() <= 2
            && after_comma.chars().all(|c| c.is_ascii_digit())
    } else {
        false
    };

    let cleaned = if has_comma_decimal {
        // 欧州形式: ピリオドを除去（千の区切り）、カンマをピリオドに（小数点）
        digits.replace('.', "").replace(',', ".")
    } else {
        // 英語/日本形式: カンマを除去（千の区切り）、ピリオドはそのまま（小数点）
        digits.replace(',', "")
    };

    let amount: f64 = cleaned.parse().unwrap_or_else(|_| {
        log::warn!("Failed to parse superchat amount: '{}' (cleaned: '{}')", amount_str, cleaned);
        0.0
    });
    (amount * 1_000_000.0) as u64
}

/// スパチャをWebSocketでブロードキャスト
pub async fn broadcast_superchat(
    ws_state: &Arc<RwLock<WebSocketState>>,
    payload: SuperchatPayload,
) {
    let message = WsMessage::SuperchatAdd {
        payload: payload.clone(),
    };

    let state = ws_state.read().await;
    state.broadcast(message).await;
    log::info!(
        "スパチャをブロードキャスト: {} (Tier {}, {})",
        payload.author_name,
        payload.tier,
        payload.amount
    );
}

/// スパチャ削除をWebSocketでブロードキャスト
pub async fn broadcast_superchat_remove(ws_state: &Arc<RwLock<WebSocketState>>, id: String) {
    let message = WsMessage::SuperchatRemove {
        payload: SuperchatRemovePayload { id: id.clone() },
    };

    let state = ws_state.read().await;
    state.broadcast(message).await;
    log::debug!("スパチャ削除をブロードキャスト: {}", id);
}

/// スパチャの表示タイマーを開始
/// 指定時間後にsuperchat:removeメッセージを送信
pub fn schedule_superchat_removal(
    ws_state: Arc<RwLock<WebSocketState>>,
    id: String,
    duration_ms: u64,
) {
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(duration_ms)).await;
        broadcast_superchat_remove(&ws_state, id).await;
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_amount_micros() {
        // 日本円
        assert_eq!(parse_amount_micros("¥1,000"), 1_000_000_000);
        assert_eq!(parse_amount_micros("¥100"), 100_000_000);
        assert_eq!(parse_amount_micros("¥10,000"), 10_000_000_000);

        // ドル
        assert_eq!(parse_amount_micros("$5.00"), 5_000_000);
        assert_eq!(parse_amount_micros("$100.00"), 100_000_000);

        // ユーロ（欧州形式）
        assert_eq!(parse_amount_micros("€5,00"), 5_000_000);
    }

    #[test]
    fn test_convert_to_jpy() {
        // 日本円はそのまま
        assert_eq!(convert_to_jpy(1_000_000_000, "JPY"), 1000);

        // USD: $10 = ¥1,500
        assert_eq!(convert_to_jpy(10_000_000, "USD"), 1500);

        // EUR: €10 = ¥1,600
        assert_eq!(convert_to_jpy(10_000_000, "EUR"), 1600);
    }

    #[test]
    fn test_calculate_tier() {
        assert_eq!(calculate_tier(100), 1);
        assert_eq!(calculate_tier(199), 1);
        assert_eq!(calculate_tier(200), 2);
        assert_eq!(calculate_tier(499), 2);
        assert_eq!(calculate_tier(500), 3);
        assert_eq!(calculate_tier(999), 3);
        assert_eq!(calculate_tier(1000), 4);
        assert_eq!(calculate_tier(1999), 4);
        assert_eq!(calculate_tier(2000), 5);
        assert_eq!(calculate_tier(4999), 5);
        assert_eq!(calculate_tier(5000), 6);
        assert_eq!(calculate_tier(9999), 6);
        assert_eq!(calculate_tier(10000), 7);
        assert_eq!(calculate_tier(50000), 7);
    }

    #[test]
    fn test_get_display_duration() {
        assert_eq!(get_display_duration(1), 10_000);
        assert_eq!(get_display_duration(4), 60_000);
        assert_eq!(get_display_duration(7), 300_000);
    }
}
