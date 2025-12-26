//! YouTube関連のDB操作を共通化したモジュール

use std::time::Duration;

use sqlx::SqlitePool;
use tokio::time::sleep;

use super::types::{ChatMessage, MessageType};

/// バッチ処理のチャンクサイズ
/// ロック保持時間を短縮するため、大きなバッチを分割して処理
const BATCH_CHUNK_SIZE: usize = 50;

/// SQLITE_BUSYエラー時の最大試行回数（初回 + リトライ）
/// 3回 = 初回試行 + 2回リトライ（100ms, 200ms後）
const MAX_ATTEMPTS: u32 = 3;

/// 初回リトライ時のバックオフ時間（ミリ秒）
const INITIAL_BACKOFF_MS: u64 = 100;

/// 最大バックオフ時間（ミリ秒）
const MAX_BACKOFF_MS: u64 = 1000;

/// トランザクション処理の結果
#[derive(Debug, Clone, Copy, PartialEq)]
enum TransactionResult {
    /// 成功
    Success,
    /// SQLITE_BUSYエラー（リトライ可能）
    Busy,
    /// その他のエラー（リトライ不可）
    OtherError,
}

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
/// - SQLITE_BUSYエラー: exponential backoffでリトライ（最大3回）
/// - トランザクション開始/コミット失敗時: 1件ずつ個別INSERTにフォールバック
/// - 個別INSERT失敗時: ログ出力して次のメッセージへ進む
pub async fn save_comments_to_db(pool: &SqlitePool, messages: &[ChatMessage]) {
    if messages.is_empty() {
        return;
    }

    // チャンクに分割して処理（ロック保持時間を短縮）
    for chunk in messages.chunks(BATCH_CHUNK_SIZE) {
        if !save_chunk_with_retry(pool, chunk).await {
            // トランザクション失敗時は1件ずつ個別INSERTにフォールバック
            log::debug!("Transaction failed after retries, falling back to individual inserts");
            save_chunk_individually(pool, chunk).await;
        }
    }
}

/// チャンクをexponential backoffでリトライしながら保存
///
/// SQLITE_BUSYエラーが発生した場合、最大MAX_ATTEMPTS回試行する。
/// 試行間隔はexponential backoffで増加（100ms → 200ms...最大1000ms）。
///
/// 例: MAX_ATTEMPTS=3の場合
///   1回目: 即時試行
///   2回目: 100ms後にリトライ
///   3回目: 200ms後にリトライ
///   → 失敗時はフォールバック（個別INSERT）
async fn save_chunk_with_retry(pool: &SqlitePool, messages: &[ChatMessage]) -> bool {
    let mut attempt = 0;
    let mut backoff_ms = INITIAL_BACKOFF_MS;

    loop {
        match save_chunk_with_transaction(pool, messages).await {
            TransactionResult::Success => return true,
            TransactionResult::OtherError => {
                // 非SQLITE_BUSYエラー（テーブル不存在など）はリトライしない
                return false;
            }
            TransactionResult::Busy => {
                attempt += 1;
                if attempt >= MAX_ATTEMPTS {
                    log::warn!(
                        "SQLITE_BUSY: Max attempts ({}) exceeded, giving up",
                        MAX_ATTEMPTS
                    );
                    return false;
                }
                log::debug!(
                    "SQLITE_BUSY: Attempt {}/{} failed, retrying after {}ms",
                    attempt,
                    MAX_ATTEMPTS,
                    backoff_ms
                );
                sleep(Duration::from_millis(backoff_ms)).await;
                backoff_ms = (backoff_ms * 2).min(MAX_BACKOFF_MS);
            }
        }
    }
}

/// エラーがSQLITE_BUSY/SQLITE_LOCKEDかどうかを判定
///
/// リトライ可能なエラー:
/// - SQLITE_BUSY (5): 他の接続がロックを保持している
/// - SQLITE_LOCKED (6): 同一接続内でのデッドロック検出
/// - SQLITE_BUSY_RECOVERY, SQLITE_BUSY_SNAPSHOT などの拡張コード
///
/// 参考: https://www.sqlite.org/rescode.html
fn is_sqlite_busy_error(e: &sqlx::Error) -> bool {
    if let sqlx::Error::Database(db_err) = e {
        // エラーコードで判定（優先）
        if let Some(code) = db_err.code() {
            let code = code.as_ref();
            // SQLite拡張エラーコードは (extended_code % 256) で基本コードを取得
            // 5 = SQLITE_BUSY, 6 = SQLITE_LOCKED
            if code == "5" || code == "6" {
                return true;
            }
            // 文字列として"SQLITE_BUSY"/"SQLITE_LOCKED"が返される場合も対応
            let code_upper = code.to_uppercase();
            if code_upper.starts_with("SQLITE_BUSY") || code_upper.starts_with("SQLITE_LOCKED") {
                return true;
            }
        }
        // エラーメッセージでも判定（フォールバック）
        // コードが取得できない場合やコードが未知の場合に対応
        let msg = db_err.message().to_lowercase();
        if msg.contains("database is locked") || msg.contains("busy") || msg.contains("locked") {
            return true;
        }
    }
    false
}

/// チャンクをトランザクション内で保存
///
/// INSERT OR IGNOREを使用しているため、UNIQUE/CHECK等の制約エラーは発生しない。
/// エラーが発生した場合は以下の致命的な問題のみ:
/// - テーブル不存在（DDLエラー）
/// - ディスクフル/I/Oエラー
/// - RAISE(ABORT, ...)トリガー
/// - SQLITE_BUSY（データベースロック）
///
/// これらの致命的エラー発生時は最初のエラーで即座にロールバックする。
/// SQLITE_BUSYの場合はリトライ可能として`TransactionResult::Busy`を返す。
async fn save_chunk_with_transaction(
    pool: &SqlitePool,
    messages: &[ChatMessage],
) -> TransactionResult {
    // トランザクションを開始
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            if is_sqlite_busy_error(&e) {
                log::debug!("SQLITE_BUSY on transaction begin: {:?}", e);
                return TransactionResult::Busy;
            }
            log::warn!("Failed to start transaction: {:?}", e);
            return TransactionResult::OtherError;
        }
    };

    for msg in messages {
        if let Err(e) = insert_comment(&mut *tx, msg).await {
            // INSERT OR IGNOREなので重複エラーは発生しないはず
            // エラーが発生した場合は致命的な問題（テーブル不存在等）
            // 最初のエラーで即座にロールバック（warn spam回避）
            if is_sqlite_busy_error(&e) {
                log::debug!("SQLITE_BUSY during insert: {:?}", e);
                // ロールバック（dropで自動的に行われるが明示的に）
                let _ = tx.rollback().await;
                return TransactionResult::Busy;
            }
            log::warn!("Insert failed in transaction, rolling back: {:?}", e);
            // ロールバック（dropで自動的に行われるが明示的に）
            if let Err(rb_err) = tx.rollback().await {
                log::debug!("Rollback failed (may already be rolled back): {:?}", rb_err);
            }
            return TransactionResult::OtherError;
        }
    }

    // コミット
    // 注: commit()はselfを消費するため、失敗後にrollback()を呼び出すことはできない
    // sqlxのTransactionはcommit失敗時に自動的にロールバックされる
    if let Err(e) = tx.commit().await {
        if is_sqlite_busy_error(&e) {
            log::debug!("SQLITE_BUSY on commit: {:?}", e);
            return TransactionResult::Busy;
        }
        log::warn!("Failed to commit transaction: {:?}", e);
        return TransactionResult::OtherError;
    }

    TransactionResult::Success
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

    #[test]
    fn test_transaction_result_enum() {
        // TransactionResult enumの基本的な動作を確認
        assert_eq!(TransactionResult::Success, TransactionResult::Success);
        assert_eq!(TransactionResult::Busy, TransactionResult::Busy);
        assert_eq!(TransactionResult::OtherError, TransactionResult::OtherError);
        assert_ne!(TransactionResult::Success, TransactionResult::Busy);
    }

    /// MockDatabaseErrorでエラー判定のテスト用構造体
    #[derive(Debug)]
    struct MockDatabaseError {
        code: Option<String>,
        message: String,
    }

    impl std::fmt::Display for MockDatabaseError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.message)
        }
    }

    impl std::error::Error for MockDatabaseError {}

    impl sqlx::error::DatabaseError for MockDatabaseError {
        fn message(&self) -> &str {
            &self.message
        }

        fn code(&self) -> Option<std::borrow::Cow<'_, str>> {
            self.code.as_ref().map(|c| std::borrow::Cow::Borrowed(c.as_str()))
        }

        fn as_error(&self) -> &(dyn std::error::Error + Send + Sync + 'static) {
            self
        }

        fn as_error_mut(&mut self) -> &mut (dyn std::error::Error + Send + Sync + 'static) {
            self
        }

        fn into_error(self: Box<Self>) -> Box<dyn std::error::Error + Send + Sync + 'static> {
            self
        }

        fn kind(&self) -> sqlx::error::ErrorKind {
            sqlx::error::ErrorKind::Other
        }
    }

    fn create_mock_db_error(code: Option<&str>, message: &str) -> sqlx::Error {
        sqlx::Error::Database(Box::new(MockDatabaseError {
            code: code.map(|c| c.to_string()),
            message: message.to_string(),
        }))
    }

    #[test]
    fn test_is_sqlite_busy_error_code_5() {
        // エラーコード5（SQLITE_BUSY）を検出
        let err = create_mock_db_error(Some("5"), "database is busy");
        assert!(is_sqlite_busy_error(&err), "Should detect code 5 as busy");
    }

    #[test]
    fn test_is_sqlite_busy_error_code_6() {
        // エラーコード6（SQLITE_LOCKED）を検出
        let err = create_mock_db_error(Some("6"), "database table is locked");
        assert!(is_sqlite_busy_error(&err), "Should detect code 6 as busy");
    }

    #[test]
    fn test_is_sqlite_busy_error_string_code() {
        // 文字列コード "SQLITE_BUSY" を検出
        let err = create_mock_db_error(Some("SQLITE_BUSY"), "database is busy");
        assert!(is_sqlite_busy_error(&err), "Should detect SQLITE_BUSY string as busy");

        // 文字列コード "SQLITE_BUSY_TIMEOUT" を検出
        let err = create_mock_db_error(Some("SQLITE_BUSY_TIMEOUT"), "timeout");
        assert!(is_sqlite_busy_error(&err), "Should detect SQLITE_BUSY_TIMEOUT as busy");

        // 文字列コード "SQLITE_LOCKED" を検出
        let err = create_mock_db_error(Some("SQLITE_LOCKED"), "locked");
        assert!(is_sqlite_busy_error(&err), "Should detect SQLITE_LOCKED as busy");
    }

    #[test]
    fn test_is_sqlite_busy_error_message_fallback() {
        // コードがない場合、メッセージで判定
        let err = create_mock_db_error(None, "database is locked");
        assert!(is_sqlite_busy_error(&err), "Should detect 'database is locked' message as busy");

        let err = create_mock_db_error(None, "The database is busy");
        assert!(is_sqlite_busy_error(&err), "Should detect 'busy' message as busy");

        let err = create_mock_db_error(None, "table is locked by another connection");
        assert!(is_sqlite_busy_error(&err), "Should detect 'locked' message as busy");
    }

    #[test]
    fn test_is_sqlite_busy_error_non_busy_code() {
        // 非busy/lockedエラーコードはfalse
        let err = create_mock_db_error(Some("1"), "SQL logic error");
        assert!(!is_sqlite_busy_error(&err), "Should not detect code 1 as busy");

        let err = create_mock_db_error(Some("19"), "constraint failed");
        assert!(!is_sqlite_busy_error(&err), "Should not detect code 19 as busy");
    }

    #[test]
    fn test_is_sqlite_busy_error_non_busy_message() {
        // 非busyエラーメッセージはfalse
        let err = create_mock_db_error(None, "syntax error");
        assert!(!is_sqlite_busy_error(&err), "Should not detect syntax error as busy");

        let err = create_mock_db_error(None, "table does not exist");
        assert!(!is_sqlite_busy_error(&err), "Should not detect table error as busy");
    }

    #[test]
    fn test_is_sqlite_busy_error_non_database_error() {
        // 非Databaseエラーはfalse
        let err = sqlx::Error::RowNotFound;
        assert!(!is_sqlite_busy_error(&err), "Should not detect RowNotFound as busy");
    }

    #[tokio::test]
    async fn test_retry_logic_with_success() {
        // リトライロジックが成功時に正常動作することを確認
        let pool = create_test_pool().await;
        create_test_table(&pool).await;

        let messages = vec![
            create_test_message("retry1", "Message 1"),
            create_test_message("retry2", "Message 2"),
        ];

        // save_chunk_with_retryが成功すること
        let result = save_chunk_with_retry(&pool, &messages).await;
        assert!(result, "Retry should succeed on first attempt");

        // データが保存されていること
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM comment_logs")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count.0, 2);
    }

    #[tokio::test]
    async fn test_retry_gives_up_on_non_busy_error() {
        // 非SQLITE_BUSYエラー時はリトライせずにすぐ失敗すること
        let pool = create_test_pool().await;
        // テーブルを作成しない → テーブル不存在エラー

        let messages = vec![create_test_message("msg1", "Hello")];

        // save_chunk_with_retryが失敗すること（リトライせずに）
        let result = save_chunk_with_retry(&pool, &messages).await;
        assert!(!result, "Should fail immediately on non-BUSY error");
    }

    /// 並行書き込みテスト用のファイルベースプールを作成
    async fn create_file_based_pool(path: &str) -> SqlitePool {
        use sqlx::sqlite::SqliteConnectOptions;
        use std::str::FromStr;

        // busy_timeoutを短く設定（テスト用）
        let connect_options = SqliteConnectOptions::from_str(&format!("sqlite:{}?mode=rwc", path))
            .unwrap()
            .busy_timeout(std::time::Duration::from_millis(50)); // 短いタイムアウト

        SqlitePoolOptions::new()
            .max_connections(2)
            .connect_with(connect_options)
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn test_concurrent_writes_with_retry() {
        use std::sync::Arc;
        use tokio::sync::Barrier;

        // テンポラリファイルを使用
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join(format!("concurrent_test_{}.db", std::process::id()));
        let db_path_str = db_path.to_str().unwrap();

        // ファイルベースのプールを作成
        let pool = create_file_based_pool(db_path_str).await;

        // テーブル作成
        sqlx::query(
            r#"CREATE TABLE IF NOT EXISTS comment_logs (
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

        let pool = Arc::new(pool);
        let barrier = Arc::new(Barrier::new(2));

        // 2つのタスクから同時に書き込み
        let pool1 = Arc::clone(&pool);
        let barrier1 = Arc::clone(&barrier);
        let task1 = tokio::spawn(async move {
            let messages: Vec<ChatMessage> = (0..30)
                .map(|i| create_test_message(&format!("task1_msg{}", i), &format!("Task1 Message {}", i)))
                .collect();
            barrier1.wait().await;
            save_comments_to_db(&pool1, &messages).await;
        });

        let pool2 = Arc::clone(&pool);
        let barrier2 = Arc::clone(&barrier);
        let task2 = tokio::spawn(async move {
            let messages: Vec<ChatMessage> = (0..30)
                .map(|i| create_test_message(&format!("task2_msg{}", i), &format!("Task2 Message {}", i)))
                .collect();
            barrier2.wait().await;
            save_comments_to_db(&pool2, &messages).await;
        });

        // 両タスクの完了を待つ
        let _ = tokio::join!(task1, task2);

        // 全てのメッセージが保存されていること（リトライ or フォールバックで）
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM comment_logs")
            .fetch_one(pool.as_ref())
            .await
            .unwrap();
        assert_eq!(count.0, 60, "All 60 messages should be saved despite concurrency");

        // クリーンアップ
        drop(pool);
        let _ = std::fs::remove_file(&db_path);
    }
}
