use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

pub mod models;

/// データベース接続プールを作成し、マイグレーションを実行
pub async fn create_pool(db_path: &str) -> Result<SqlitePool, sqlx::Error> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&format!("sqlite:{}?mode=rwc", db_path))
        .await?;

    // マイグレーション実行
    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}
