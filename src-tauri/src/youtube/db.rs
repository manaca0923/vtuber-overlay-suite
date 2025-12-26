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
///
/// INSERT OR IGNOREを使用しているため、UNIQUE/CHECK等の制約エラーは発生しない。
/// エラーが発生した場合は以下の致命的な問題のみ:
/// - テーブル不存在（DDLエラー）
/// - ディスクフル/I/Oエラー
/// - RAISE(ABORT, ...)トリガー
///
/// これらの致命的エラー発生時は最初のエラーで即座にロールバックし、
/// フォールバック処理（save_chunk_individually）に移行する。
async fn save_chunk_with_transaction(pool: &SqlitePool, messages: &[ChatMessage]) -> bool {
    // トランザクションを開始
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            log::warn!("Failed to start transaction: {:?}", e);
            return false;
        }
    };

    for msg in messages {
        if let Err(e) = insert_comment(&mut *tx, msg).await {
            // INSERT OR IGNOREなので重複エラーは発生しないはず
            // エラーが発生した場合は致命的な問題（テーブル不存在等）
            // 最初のエラーで即座にロールバック（warn spam回避）
            log::warn!("Insert failed in transaction, rolling back: {:?}", e);
            // ロールバック（dropで自動的に行われるが明示的に）
            if let Err(rb_err) = tx.rollback().await {
                log::debug!("Rollback failed (may already be rolled back): {:?}", rb_err);
            }
            return false;
        }
    }

    // コミット
    // 注: commit()はselfを消費するため、失敗後にrollback()を呼び出すことはできない
    // sqlxのTransactionはcommit失敗時に自動的にロールバックされる
    if let Err(e) = tx.commit().await {
        log::warn!("Failed to commit transaction: {:?}", e);
        return false;
    }

    true
}

/// チャンクを1件ずつ個別に保存（フォールバック用）
///
/// 単一接続を取得して再利用し、行ごとのpool checkout回避でパフォーマンス向上
async fn save_chunk_individually(pool: &SqlitePool, messages: &[ChatMessage]) {
    let mut success_count = 0;
    let mut error_count = 0;

    // 単一接続を取得して再利用（行ごとのpool checkout回避）
    let mut conn = match pool.acquire().await {
        Ok(conn) => conn,
        Err(e) => {
            log::warn!("Failed to acquire connection for fallback: {:?}", e);
            return;
        }
    };

    for msg in messages {
        match insert_comment(&mut *conn, msg).await {
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
    use sqlx::sqlite::SqlitePoolOptions;

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

    /// 単一接続のin-memoryプールを作成（DDL/DMLが同一DBで実行されることを保証）
    async fn create_test_pool() -> SqlitePool {
        SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap()
    }

    /// テスト用のcomment_logsテーブルを作成
    async fn create_test_table(pool: &SqlitePool) {
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
        .execute(pool)
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_save_empty_batch_is_noop() {
        let pool = create_test_pool().await;
        // テーブルがなくても空バッチは即座にリターン
        save_comments_to_db(&pool, &[]).await;
    }

    #[tokio::test]
    async fn test_save_comments_batch() {
        let pool = create_test_pool().await;
        create_test_table(&pool).await;

        let messages = vec![
            create_test_message("msg1", "Hello"),
            create_test_message("msg2", "World"),
            create_test_message("msg3", "Test"),
        ];

        save_comments_to_db(&pool, &messages).await;

        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM comment_logs")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count.0, 3);
    }

    #[tokio::test]
    async fn test_save_comments_ignores_duplicates() {
        let pool = create_test_pool().await;
        create_test_table(&pool).await;

        let messages = vec![
            create_test_message("msg1", "First"),
            create_test_message("msg1", "Duplicate"), // 重複ID
            create_test_message("msg2", "Second"),
        ];

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

    #[tokio::test]
    async fn test_save_comments_chunk_boundary() {
        // BATCH_CHUNK_SIZE + 1 件のメッセージでチャンク境界をテスト
        let pool = create_test_pool().await;
        create_test_table(&pool).await;

        let messages: Vec<ChatMessage> = (0..=BATCH_CHUNK_SIZE)
            .map(|i| create_test_message(&format!("msg{}", i), &format!("Message {}", i)))
            .collect();

        save_comments_to_db(&pool, &messages).await;

        // 全件保存されていること
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM comment_logs")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count.0, (BATCH_CHUNK_SIZE + 1) as i64);
    }

    #[tokio::test]
    async fn test_save_comments_fallback_on_missing_table() {
        // テーブルがない場合、トランザクション失敗→フォールバック→個別も失敗
        // エラーログは出るが、パニックせず完了すること
        let pool = create_test_pool().await;
        // テーブルを作成しない

        let messages = vec![
            create_test_message("msg1", "Hello"),
            create_test_message("msg2", "World"),
        ];

        // パニックしないことを確認（エラーはログに出力される）
        save_comments_to_db(&pool, &messages).await;
    }

    /// CHECK制約付きテーブルを作成（テスト用）
    async fn create_test_table_with_check(pool: &SqlitePool) {
        sqlx::query(
            r#"CREATE TABLE comment_logs (
                id TEXT PRIMARY KEY,
                youtube_id TEXT UNIQUE NOT NULL,
                message TEXT NOT NULL CHECK(length(message) <= 10),
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
        .execute(pool)
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_save_comments_with_check_constraint_violation() {
        // CHECK制約違反があっても、INSERT OR IGNOREにより無視される
        // 有効な行のみ保存されることを確認
        let pool = create_test_pool().await;
        create_test_table_with_check(&pool).await;

        let messages = vec![
            create_test_message("msg1", "Short"),                   // OK: 5文字
            create_test_message("msg2", "This is way too long"),    // NG: 21文字 > 10
            create_test_message("msg3", "Valid"),                   // OK: 5文字
        ];

        save_comments_to_db(&pool, &messages).await;

        // INSERT OR IGNORE により、CHECK制約違反の行はスキップされる
        // 有効な2件のみ保存されること
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM comment_logs")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count.0, 2);

        // 有効なメッセージが保存されていること
        let msg1: (String,) = sqlx::query_as("SELECT message FROM comment_logs WHERE id = 'msg1'")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(msg1.0, "Short");

        let msg3: (String,) = sqlx::query_as("SELECT message FROM comment_logs WHERE id = 'msg3'")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(msg3.0, "Valid");

        // CHECK制約違反のメッセージは保存されていないこと
        let msg2_count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM comment_logs WHERE id = 'msg2'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(msg2_count.0, 0);
    }

    /// RAISE(ABORT)トリガー付きテーブルを作成（テスト用）
    /// 特定のID（"trigger_fail"）でハードエラーを発生させる
    async fn create_test_table_with_abort_trigger(pool: &SqlitePool) {
        // テーブル作成
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
        .execute(pool)
        .await
        .unwrap();

        // 特定のIDでRAISE(ABORT)を発生させるトリガー
        sqlx::query(
            r#"CREATE TRIGGER abort_on_trigger_fail
               BEFORE INSERT ON comment_logs
               WHEN NEW.id = 'trigger_fail'
               BEGIN
                   SELECT RAISE(ABORT, 'Forced abort for testing');
               END"#,
        )
        .execute(pool)
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_save_comments_fallback_with_abort_trigger() {
        // RAISE(ABORT)トリガーによるハード障害をテスト
        // トランザクション失敗 → フォールバック → 残りの行は保存される
        let pool = create_test_pool().await;
        create_test_table_with_abort_trigger(&pool).await;

        let messages = vec![
            create_test_message("msg1", "First"),
            create_test_message("trigger_fail", "This triggers abort"), // ABORTトリガー
            create_test_message("msg3", "Third"),
        ];

        save_comments_to_db(&pool, &messages).await;

        // フォールバック処理により、トリガー対象以外の行は保存される
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM comment_logs")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count.0, 2, "Valid rows should be saved via fallback");

        // 正常なメッセージが保存されていること
        let msg1: (String,) = sqlx::query_as("SELECT message FROM comment_logs WHERE id = 'msg1'")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(msg1.0, "First");

        let msg3: (String,) = sqlx::query_as("SELECT message FROM comment_logs WHERE id = 'msg3'")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(msg3.0, "Third");

        // トリガー対象のメッセージは保存されていないこと
        let trigger_count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM comment_logs WHERE id = 'trigger_fail'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(trigger_count.0, 0, "Trigger target should not be saved");
    }
}
