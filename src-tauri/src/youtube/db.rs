//! YouTube関連のDB操作を共通化したモジュール

use sqlx::SqlitePool;

use super::types::{ChatMessage, MessageType};

/// コメントをDBに保存（バッチ処理最適化版）
///
/// トランザクション内で複数のINSERTを実行し、I/O効率を向上させる。
/// INSERT OR IGNOREで重複を無視し、既存レコードはスキップする。
/// youtube_idのUNIQUE制約により重複コメントは自動的にスキップされる。
///
/// ## 保存形式
/// - `message_type`: 短い文字列 ("text", "superChat", "superSticker", "membership", "membershipGift")
/// - `message_data`: MessageTypeの詳細データをJSON（TextはNULL）
/// - `published_at`: RFC3339形式の文字列
pub async fn save_comments_to_db(pool: &SqlitePool, messages: &[ChatMessage]) {
    if messages.is_empty() {
        return;
    }

    // トランザクションを開始してバッチ処理
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            log::warn!("Failed to start transaction for comment batch: {:?}", e);
            return;
        }
    };

    let mut success_count = 0;
    let mut error_count = 0;

    for msg in messages {
        // MessageTypeを短い文字列に変換
        let message_type = match &msg.message_type {
            MessageType::Text => "text",
            MessageType::SuperChat { .. } => "superChat",
            MessageType::SuperSticker { .. } => "superSticker",
            MessageType::Membership { .. } => "membership",
            MessageType::MembershipGift { .. } => "membershipGift",
        };

        // MessageTypeの詳細データをJSONに変換（Textの場合はNULL）
        let message_data = match &msg.message_type {
            MessageType::Text => None,
            other => serde_json::to_string(other).ok(),
        };

        // published_atをRFC3339形式に変換
        let published_at_str = msg.published_at.to_rfc3339();

        let result = sqlx::query(
            r#"INSERT OR IGNORE INTO comment_logs
            (id, youtube_id, message, author_name, author_channel_id, author_image_url,
             is_owner, is_moderator, is_member, message_type, message_data, published_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
        )
        .bind(&msg.id)           // id (youtube_idと同じ値を使用)
        .bind(&msg.id)           // youtube_id
        .bind(&msg.message)
        .bind(&msg.author_name)
        .bind(&msg.author_channel_id)
        .bind(&msg.author_image_url)
        .bind(msg.is_owner)
        .bind(msg.is_moderator)
        .bind(msg.is_member)
        .bind(message_type)
        .bind(&message_data)
        .bind(&published_at_str)
        .execute(&mut *tx)
        .await;

        match result {
            Ok(_) => success_count += 1,
            Err(e) => {
                error_count += 1;
                log::debug!("Failed to insert comment in batch: {:?}", e);
            }
        }
    }

    // トランザクションをコミット
    if let Err(e) = tx.commit().await {
        log::warn!(
            "Failed to commit comment batch (tried {} inserts): {:?}",
            messages.len(),
            e
        );
        return;
    }

    if error_count > 0 {
        log::debug!(
            "Comment batch completed: {} succeeded, {} failed",
            success_count,
            error_count
        );
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_exists() {
        // モジュールが正しく読み込まれることを確認
        assert!(true);
    }
}
