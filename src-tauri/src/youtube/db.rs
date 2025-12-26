//! YouTube関連のDB操作を共通化したモジュール

use sqlx::SqlitePool;

use super::types::{ChatMessage, MessageType};

/// バッチ処理のチャンクサイズ
/// ロック保持時間を短縮するため、大きなバッチを分割して処理
const BATCH_CHUNK_SIZE: usize = 50;

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
///
/// ## エラー処理
/// - トランザクション開始/コミット失敗時: 1件ずつ個別INSERTにフォールバック
/// - 個別INSERT失敗時: ログ出力して次のメッセージへ進む
pub async fn save_comments_to_db(pool: &SqlitePool, messages: &[ChatMessage]) {
    if messages.is_empty() {
        return;
    }

    // チャンクに分割して処理（ロック保持時間を短縮）
    for chunk in messages.chunks(BATCH_CHUNK_SIZE) {
        if !save_chunk_with_transaction(pool, chunk).await {
            // トランザクション失敗時は1件ずつ個別INSERTにフォールバック
            log::debug!("Transaction failed, falling back to individual inserts");
            save_chunk_individually(pool, chunk).await;
        }
    }
}

/// チャンクをトランザクション内で保存（成功時true、失敗時false）
async fn save_chunk_with_transaction(pool: &SqlitePool, messages: &[ChatMessage]) -> bool {
    // トランザクションを開始
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            log::debug!("Failed to start transaction: {:?}", e);
            return false;
        }
    };

    for msg in messages {
        if let Err(e) = insert_comment(&mut *tx, msg).await {
            log::debug!("Failed to insert comment in transaction: {:?}", e);
            // INSERT OR IGNOREなので重複エラーは発生しないはずだが、
            // 他のエラー（disk full等）の場合は続行
        }
    }

    // コミット
    if let Err(e) = tx.commit().await {
        log::debug!("Failed to commit transaction: {:?}", e);
        return false;
    }

    true
}

/// チャンクを1件ずつ個別に保存（フォールバック用）
async fn save_chunk_individually(pool: &SqlitePool, messages: &[ChatMessage]) {
    let mut success_count = 0;
    let mut error_count = 0;

    for msg in messages {
        match insert_comment(pool, msg).await {
            Ok(_) => success_count += 1,
            Err(e) => {
                error_count += 1;
                log::debug!("Failed to insert comment individually: {:?}", e);
            }
        }
    }

    if error_count > 0 {
        log::warn!(
            "Individual insert fallback: {} succeeded, {} failed",
            success_count,
            error_count
        );
    }
}

/// 単一コメントをINSERT
async fn insert_comment<'e, E>(executor: E, msg: &ChatMessage) -> Result<(), sqlx::Error>
where
    E: sqlx::Executor<'e, Database = sqlx::Sqlite>,
{
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

    sqlx::query(
        r#"INSERT OR IGNORE INTO comment_logs
        (id, youtube_id, message, author_name, author_channel_id, author_image_url,
         is_owner, is_moderator, is_member, message_type, message_data, published_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
    )
    .bind(&msg.id)
    .bind(&msg.id)
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
    .execute(executor)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_message(id: &str, message: &str) -> ChatMessage {
        ChatMessage {
            id: id.to_string(),
            message: message.to_string(),
            message_runs: None,
            author_name: "TestUser".to_string(),
            author_channel_id: "UC123".to_string(),
            author_image_url: "https://example.com/icon.png".to_string(),
            message_type: MessageType::Text,
            is_owner: false,
            is_moderator: false,
            is_member: false,
            is_verified: false,
            published_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_save_empty_batch_is_noop() {
        // in-memoryデータベースを作成
        let pool = sqlx::SqlitePool::connect(":memory:").await.unwrap();

        // 空のバッチを保存（エラーなく完了すること）
        save_comments_to_db(&pool, &[]).await;
    }

    #[tokio::test]
    async fn test_save_comments_batch() {
        // in-memoryデータベースを作成
        let pool = sqlx::SqlitePool::connect(":memory:").await.unwrap();

        // comment_logsテーブルを作成
        sqlx::query(
            r#"CREATE TABLE comment_logs (
                id TEXT PRIMARY KEY,
                youtube_id TEXT UNIQUE NOT NULL,
                message TEXT NOT NULL,
                author_name TEXT NOT NULL,
                author_channel_id TEXT NOT NULL,
                author_image_url TEXT,
                is_owner BOOLEAN NOT NULL DEFAULT 0,
                is_moderator BOOLEAN NOT NULL DEFAULT 0,
                is_member BOOLEAN NOT NULL DEFAULT 0,
                message_type TEXT NOT NULL,
                message_data TEXT,
                published_at TEXT NOT NULL
            )"#,
        )
        .execute(&pool)
        .await
        .unwrap();

        // テストメッセージを作成
        let messages = vec![
            create_test_message("msg1", "Hello"),
            create_test_message("msg2", "World"),
            create_test_message("msg3", "Test"),
        ];

        // バッチ保存
        save_comments_to_db(&pool, &messages).await;

        // 保存されたレコード数を確認
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM comment_logs")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count.0, 3);
    }

    #[tokio::test]
    async fn test_save_comments_ignores_duplicates() {
        // in-memoryデータベースを作成
        let pool = sqlx::SqlitePool::connect(":memory:").await.unwrap();

        // comment_logsテーブルを作成
        sqlx::query(
            r#"CREATE TABLE comment_logs (
                id TEXT PRIMARY KEY,
                youtube_id TEXT UNIQUE NOT NULL,
                message TEXT NOT NULL,
                author_name TEXT NOT NULL,
                author_channel_id TEXT NOT NULL,
                author_image_url TEXT,
                is_owner BOOLEAN NOT NULL DEFAULT 0,
                is_moderator BOOLEAN NOT NULL DEFAULT 0,
                is_member BOOLEAN NOT NULL DEFAULT 0,
                message_type TEXT NOT NULL,
                message_data TEXT,
                published_at TEXT NOT NULL
            )"#,
        )
        .execute(&pool)
        .await
        .unwrap();

        // 同じIDのメッセージを含むバッチを作成
        let messages = vec![
            create_test_message("msg1", "First"),
            create_test_message("msg1", "Duplicate"), // 重複ID
            create_test_message("msg2", "Second"),
        ];

        // バッチ保存（重複は無視される）
        save_comments_to_db(&pool, &messages).await;

        // 重複が無視されて2件のみ保存されること
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM comment_logs")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count.0, 2);

        // 最初のメッセージが保存されていること
        let msg: (String,) = sqlx::query_as("SELECT message FROM comment_logs WHERE id = 'msg1'")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(msg.0, "First");
    }
}
