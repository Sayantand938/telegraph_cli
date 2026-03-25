mod categories;
mod places;
mod tags;
mod persons;
mod transactions;
mod activities;

use sqlx::SqlitePool;
use std::path::PathBuf;
use crate::error::AppResult;

pub use categories::*;
pub use places::*;
pub use tags::*;
pub use persons::*;
pub use transactions::*;
pub use activities::*;

pub async fn connect_db(db_path: Option<PathBuf>) -> AppResult<SqlitePool> {
    let path = if let Some(p) = db_path {
        p
    } else {
        let data_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("telegraph_cli");
        std::fs::create_dir_all(&data_dir)?;
        data_dir.join("tracker.db")
    };

    // Create empty db file if it doesn't exist
    if !path.exists() {
        std::fs::File::create(&path)?;
    }

    let db_url = path.to_string_lossy().to_string();
    let pool = SqlitePool::connect(&db_url).await?;
    Ok(pool)
}

pub async fn init_tables(pool: &SqlitePool) -> AppResult<()> {
    // Categories table (shared between transactions and activities)
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS categories (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE
        );
        "#,
    )
    .execute(pool)
    .await?;

    // Places table (shared between transactions and activities)
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS places (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE
        );
        "#,
    )
    .execute(pool)
    .await?;

    // Tags table (many-to-many with transactions and activities)
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS tags (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE
        );
        "#,
    )
    .execute(pool)
    .await?;

    // Transaction-Tags junction table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS transaction_tags (
            transaction_id INTEGER NOT NULL,
            tag_id INTEGER NOT NULL,
            PRIMARY KEY (transaction_id, tag_id),
            FOREIGN KEY (transaction_id) REFERENCES transactions(id) ON DELETE CASCADE,
            FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
        );
        "#,
    )
    .execute(pool)
    .await?;

    // Activity-Tags junction table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS activity_tags (
            activity_id INTEGER NOT NULL,
            tag_id INTEGER NOT NULL,
            PRIMARY KEY (activity_id, tag_id),
            FOREIGN KEY (activity_id) REFERENCES activities(id) ON DELETE CASCADE,
            FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
        );
        "#,
    )
    .execute(pool)
    .await?;

    // Persons table (many-to-many with transactions and activities)
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS persons (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE
        );
        "#,
    )
    .execute(pool)
    .await?;

    // Transaction-Persons junction table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS transaction_persons (
            transaction_id INTEGER NOT NULL,
            person_id INTEGER NOT NULL,
            PRIMARY KEY (transaction_id, person_id),
            FOREIGN KEY (transaction_id) REFERENCES transactions(id) ON DELETE CASCADE,
            FOREIGN KEY (person_id) REFERENCES persons(id) ON DELETE CASCADE
        );
        "#,
    )
    .execute(pool)
    .await?;

    // Activity-Persons junction table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS activity_persons (
            activity_id INTEGER NOT NULL,
            person_id INTEGER NOT NULL,
            PRIMARY KEY (activity_id, person_id),
            FOREIGN KEY (activity_id) REFERENCES activities(id) ON DELETE CASCADE,
            FOREIGN KEY (person_id) REFERENCES persons(id) ON DELETE CASCADE
        );
        "#,
    )
    .execute(pool)
    .await?;

    // Transactions table with category_id and place_id
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS transactions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            amount REAL NOT NULL,
            kind TEXT NOT NULL,
            description TEXT,
            timestamp TEXT NOT NULL,
            category_id INTEGER,
            place_id INTEGER,
            FOREIGN KEY (category_id) REFERENCES categories(id),
            FOREIGN KEY (place_id) REFERENCES places(id)
        );
        "#,
    )
    .execute(pool)
    .await?;

    // Activities table with category_id and place_id
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS activities (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            start_time TEXT NOT NULL,
            stop_time TEXT NOT NULL,
            description TEXT,
            category_id INTEGER,
            place_id INTEGER,
            FOREIGN KEY (category_id) REFERENCES categories(id),
            FOREIGN KEY (place_id) REFERENCES places(id)
        );
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}
