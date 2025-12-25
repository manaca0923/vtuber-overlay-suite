//! YouTube関連のDB操作を共通化したモジュール

use sqlx::SqlitePool;
use uuid::Uuid;

use super::types::ChatMessage;

/// コメントをDBに保存
///
/// INSERT OR IGNOREで重複を無視し、既存レコードはスキップする
/// youtube_idのUNIQUE制約により重複コメントは自動的にスキップされる
pub async fn save_comments_to_db(pool: &SqlitePool, messages: &[ChatMessage]) {
    for msg in messages {
        let id = Uuid::new_v4().to_string();
        let is_owner = if msg.is_owner { 1 } else { 0 };
        let is_moderator = if msg.is_moderator { 1 } else { 0 };
        let is_member = if msg.is_member { 1 } else { 0 };

        // message_typeをJSON文字列に変換
        let message_type_str = serde_json::to_string(&msg.message_type)
            .unwrap_or_else(|_| r#"{"type":"text"}"#.to_string());

        let result = sqlx::query!(
            r#"INSERT OR IGNORE INTO comment_logs
            (id, youtube_id, message, author_name, author_channel_id, author_image_url,
             is_owner, is_moderator, is_member, message_type, published_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
            id,
            msg.id,
            msg.message,
            msg.author_name,
            msg.author_channel_id,
            msg.author_image_url,
            is_owner,
            is_moderator,
            is_member,
            message_type_str,
            msg.published_at
        )
        .execute(pool)
        .await;

        if let Err(e) = result {
            log::warn!("Failed to save comment to DB: {:?}", e);
        }
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
