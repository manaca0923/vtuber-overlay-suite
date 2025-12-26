use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    SqlitePool,
};
use std::str::FromStr;
use std::time::Duration;

pub mod models;

/// busy_timeout設定（ミリ秒）
/// SQLiteのロック競合時に待機する最大時間
/// 5秒あれば通常の競合は解消される
const SQLITE_BUSY_TIMEOUT_MS: u64 = 5000;

/// データベース接続プールを作成し、マイグレーションを実行
pub async fn create_pool(db_path: &str) -> Result<SqlitePool, sqlx::Error> {
    // SqliteConnectOptionsを使用してbusy_timeoutを明示的に設定
    // URIパラメータではなくAPIを使用することで、設定が確実に適用される
    let connect_options = SqliteConnectOptions::from_str(&format!("sqlite:{}?mode=rwc", db_path))?
        .busy_timeout(Duration::from_millis(SQLITE_BUSY_TIMEOUT_MS));

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(connect_options)
        .await?;

    // マイグレーション実行
    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    /// ユニークなテスト用DBパスを生成
    /// プロセスIDとタイムスタンプを組み合わせて衝突を回避
    fn unique_test_db_path(prefix: &str) -> std::path::PathBuf {
        let temp_dir = env::temp_dir();
        let pid = std::process::id();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        temp_dir.join(format!("{}_{}_{}_{}.db", prefix, pid, timestamp, rand_suffix()))
    }

    /// 簡易的なランダムサフィックス生成
    fn rand_suffix() -> u32 {
        use std::collections::hash_map::RandomState;
        use std::hash::{BuildHasher, Hasher};
        let state = RandomState::new();
        let mut hasher = state.build_hasher();
        hasher.write_usize(0);
        (hasher.finish() & 0xFFFF) as u32
    }

    /// create_poolのスモークテスト
    /// busy_timeout設定を含む接続オプションでプールが正常に作成されることを検証
    #[tokio::test]
    async fn test_create_pool_with_busy_timeout() {
        // ユニークなテスト用DBファイルを作成（プロセス間衝突を回避）
        let db_path = unique_test_db_path("test_busy_timeout");
        let db_path_str = db_path.to_str().unwrap();

        // プール作成が成功することを検証
        let result = create_pool(db_path_str).await;
        assert!(
            result.is_ok(),
            "Pool creation with busy_timeout should succeed: {:?}",
            result.err()
        );

        let pool = result.unwrap();

        // 簡単なクエリが実行できることを確認
        let row: (i64,) = sqlx::query_as("SELECT 1")
            .fetch_one(&pool)
            .await
            .expect("Simple query should succeed");
        assert_eq!(row.0, 1);

        // クリーンアップ
        drop(pool);
        let _ = fs::remove_file(&db_path);
    }

    /// busy_timeoutがPRAGMAレベルで設定されていることを検証
    #[tokio::test]
    async fn test_busy_timeout_pragma_is_set() {
        let db_path = unique_test_db_path("test_pragma_busy_timeout");
        let db_path_str = db_path.to_str().unwrap();

        let pool = create_pool(db_path_str)
            .await
            .expect("Pool creation should succeed");

        // PRAGMA busy_timeoutの値を確認
        // SqliteConnectOptionsで設定した値がセッションに反映されているか検証
        let row: (i64,) = sqlx::query_as("PRAGMA busy_timeout")
            .fetch_one(&pool)
            .await
            .expect("PRAGMA query should succeed");

        // busy_timeout はSQLITE_BUSY_TIMEOUT_MS（5000ms）に設定されているはず
        assert_eq!(
            row.0 as u64, SQLITE_BUSY_TIMEOUT_MS,
            "busy_timeout should be set to {} ms",
            SQLITE_BUSY_TIMEOUT_MS
        );

        // クリーンアップ
        drop(pool);
        let _ = fs::remove_file(&db_path);
    }
}
