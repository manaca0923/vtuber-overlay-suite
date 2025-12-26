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

    /// create_poolのスモークテスト
    /// busy_timeout設定を含む接続オプションでプールが正常に作成されることを検証
    #[tokio::test]
    async fn test_create_pool_with_busy_timeout() {
        // 一時ディレクトリにテスト用DBファイルを作成
        let temp_dir = env::temp_dir();
        let db_path = temp_dir.join("test_busy_timeout_pool.db");
        let db_path_str = db_path.to_str().unwrap();

        // 既存のテストDBがあれば削除
        let _ = fs::remove_file(&db_path);

        // プール作成が成功することを検証
        let result = create_pool(db_path_str).await;
        assert!(result.is_ok(), "Pool creation with busy_timeout should succeed: {:?}", result.err());

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
}
