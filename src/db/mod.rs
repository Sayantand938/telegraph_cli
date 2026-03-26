mod macros;
mod categories;
mod places;
mod tags;
mod persons;
mod transactions;
mod activities;
mod todos;
mod journal;

use sqlx::SqlitePool;
use std::path::PathBuf;
use crate::error::AppResult;

pub use categories::*;
pub use places::*;
pub use tags::*;
pub use persons::*;
pub use transactions::*;
pub use activities::*;
pub use todos::*;
pub use journal::*;

pub async fn connect_db(db_path: Option<PathBuf>) -> AppResult<SqlitePool> {
    let path = if let Some(p) = db_path {
        p
    } else {
        let data_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("logbook");
        std::fs::create_dir_all(&data_dir)?;
        data_dir.join("logbook.db")
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

    // Todos table with category_id and place_id
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS todos (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            description TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'pending',
            priority TEXT,
            due_date TEXT,
            created_at TEXT NOT NULL,
            completed_at TEXT,
            category_id INTEGER,
            place_id INTEGER,
            FOREIGN KEY (category_id) REFERENCES categories(id),
            FOREIGN KEY (place_id) REFERENCES places(id)
        );
        "#,
    )
    .execute(pool)
    .await?;

    // Todo-Tags junction table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS todo_tags (
            todo_id INTEGER NOT NULL,
            tag_id INTEGER NOT NULL,
            PRIMARY KEY (todo_id, tag_id),
            FOREIGN KEY (todo_id) REFERENCES todos(id) ON DELETE CASCADE,
            FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
        );
        "#,
    )
    .execute(pool)
    .await?;

    // Todo-Persons junction table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS todo_persons (
            todo_id INTEGER NOT NULL,
            person_id INTEGER NOT NULL,
            PRIMARY KEY (todo_id, person_id),
            FOREIGN KEY (todo_id) REFERENCES todos(id) ON DELETE CASCADE,
            FOREIGN KEY (person_id) REFERENCES persons(id) ON DELETE CASCADE
        );
        "#,
    )
    .execute(pool)
    .await?;

    // Journal entries table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS journal_entries (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            content TEXT NOT NULL,
            date TEXT,
            category_id INTEGER,
            place_id INTEGER,
            created_at TEXT NOT NULL,
            FOREIGN KEY (category_id) REFERENCES categories(id),
            FOREIGN KEY (place_id) REFERENCES places(id)
        );
        "#,
    )
    .execute(pool)
    .await?;

    // FTS5 virtual table for journal search
    sqlx::query(
        r#"
        CREATE VIRTUAL TABLE IF NOT EXISTS journal_fts USING fts5(
            content,
            tags,
            persons,
            content='journal_entries',
            content_rowid='id'
        );
        "#,
    )
    .execute(pool)
    .await?;

    // Journal-Tags junction table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS journal_tags (
            journal_id INTEGER NOT NULL,
            tag_id INTEGER NOT NULL,
            PRIMARY KEY (journal_id, tag_id),
            FOREIGN KEY (journal_id) REFERENCES journal_entries(id) ON DELETE CASCADE,
            FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
        );
        "#,
    )
    .execute(pool)
    .await?;

    // Journal-Persons junction table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS journal_persons (
            journal_id INTEGER NOT NULL,
            person_id INTEGER NOT NULL,
            PRIMARY KEY (journal_id, person_id),
            FOREIGN KEY (journal_id) REFERENCES journal_entries(id) ON DELETE CASCADE,
            FOREIGN KEY (person_id) REFERENCES persons(id) ON DELETE CASCADE
        );
        "#,
    )
    .execute(pool)
    .await?;

    // Triggers for FTS5 sync
    sqlx::query(
        r#"
        CREATE TRIGGER IF NOT EXISTS journal_ai AFTER INSERT ON journal_entries BEGIN
            INSERT INTO journal_fts(rowid, content, tags, persons)
            VALUES (new.id, new.content, '', '');
        END;
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TRIGGER IF NOT EXISTS journal_ad AFTER DELETE ON journal_entries BEGIN
            INSERT INTO journal_fts(journal_fts, rowid, content, tags, persons)
            VALUES('delete', old.id, old.content, '', '');
        END;
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Create an in-memory database for testing
    async fn create_test_pool() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .unwrap();
        init_tables(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn test_connect_db_custom_path() {
        let temp_path = std::env::temp_dir().join(format!("tx_tracker_custom_test_{}.db", std::process::id()));
        let result = connect_db(Some(temp_path.clone())).await;
        assert!(result.is_ok());
        assert!(temp_path.exists());
        // Cleanup
        let _ = std::fs::remove_file(temp_path);
    }

    #[tokio::test]
    async fn test_init_tables_idempotent() {
        let pool = create_test_pool().await;
        
        // Call init_tables again - should not fail
        let result = init_tables(&pool).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_init_tables_creates_all_tables() {
        let pool = create_test_pool().await;
        
        // Check all tables exist
        let tables = [
            "categories", "places", "tags", "persons",
            "transaction_tags", "activity_tags",
            "transaction_persons", "activity_persons",
            "transactions", "activities",
        ];
        
        for table in &tables {
            let result: (i64,) = sqlx::query_as(&format!(
                "SELECT COUNT(*) FROM {}",
                table
            ))
            .fetch_one(&pool)
            .await
            .unwrap();
            assert_eq!(result.0, 0); // Should be empty but exist
        }
    }
}
