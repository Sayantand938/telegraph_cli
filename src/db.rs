use sqlx::SqlitePool;
use std::fs::{self, File};
use crate::error::AppResult;

pub async fn connect_db() -> AppResult<SqlitePool> {
    let data_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("telegraph_cli");
    
    fs::create_dir_all(&data_dir)?;
    
    let db_path = data_dir.join("tracker.db");
    
    // Create empty db file if it doesn't exist
    if !db_path.exists() {
        File::create(&db_path)?;
    }
    
    let db_url = db_path.to_string_lossy().to_string();
    let pool = SqlitePool::connect(&db_url).await?;
    Ok(pool)
}

pub async fn init_tables(pool: &SqlitePool) -> AppResult<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS transactions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            amount REAL NOT NULL,
            kind TEXT NOT NULL,
            description TEXT,
            timestamp TEXT NOT NULL
        );
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS activities (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            start_time TEXT NOT NULL,
            stop_time TEXT NOT NULL,
            description TEXT
        );
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}
