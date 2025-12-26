use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

pub mod models;

/// busy_timeout設定（ミリ秒）
/// SQLiteのロック競合時に待機する最大時間
/// 5秒あれば通常の競合は解消される
const SQLITE_BUSY_TIMEOUT_MS: u32 = 5000;

/// データベース接続プールを作成し、マイグレーションを実行
pub async fn create_pool(db_path: &str) -> Result<SqlitePool, sqlx::Error> {
    // busy_timeout: ロック競合時に即座にSQLITE_BUSYを返さず、指定時間待機してリトライ
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&format!(
            "sqlite:{}?mode=rwc&busy_timeout={}",
            db_path, SQLITE_BUSY_TIMEOUT_MS
        ))
        .await?;

    // マイグレーション実行
    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}
