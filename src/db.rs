use sqlx::{SqlitePool, Row};
use std::path::PathBuf;
use crate::error::AppResult;
use crate::{Transaction, Activity};

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

// ----- Transaction DB Operations -----

pub async fn add_transaction(
    pool: &SqlitePool,
    amount: f64,
    kind: &str,
    description: &str,
) -> AppResult<i64> {
    let result = sqlx::query("INSERT INTO transactions (amount, kind, description, timestamp) VALUES (?, ?, ?, ?)")
        .bind(amount)
        .bind(kind)
        .bind(description)
        .bind(chrono::Utc::now().to_rfc3339())
        .execute(pool)
        .await?;
    Ok(result.last_insert_rowid())
}

pub async fn get_transaction(pool: &SqlitePool, id: i64) -> AppResult<Option<Transaction>> {
    let row = sqlx::query("SELECT id, amount, kind, description, timestamp FROM transactions WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?;

    match row {
        Some(row) => {
            let id: i64 = row.try_get("id")?;
            let amount: f64 = row.try_get("amount")?;
            let kind: String = row.try_get("kind")?;
            let desc: String = row.try_get("description")?;
            Ok(Some(Transaction {
                id: Some(id),
                amount,
                kind,
                description: desc,
            }))
        }
        None => Ok(None),
    }
}

pub async fn list_transactions(
    pool: &SqlitePool,
    kind_filter: Option<&str>,
) -> AppResult<Vec<Transaction>> {
    let rows = if let Some(kind) = kind_filter {
        sqlx::query("SELECT id, amount, kind, description, timestamp FROM transactions WHERE kind = ?")
            .bind(kind)
            .fetch_all(pool)
            .await?
    } else {
        sqlx::query("SELECT id, amount, kind, description, timestamp FROM transactions")
            .fetch_all(pool)
            .await?
    };

    let mut transactions = Vec::new();
    for row in rows {
        let id: i64 = row.try_get("id")?;
        let amount: f64 = row.try_get("amount")?;
        let kind: String = row.try_get("kind")?;
        let desc: String = row.try_get("description")?;
        transactions.push(Transaction {
            id: Some(id),
            amount,
            kind,
            description: desc,
        });
    }
    Ok(transactions)
}

pub async fn update_transaction(
    pool: &SqlitePool,
    id: i64,
    amount: Option<f64>,
    kind: Option<&str>,
    desc: Option<&str>,
) -> AppResult<()> {
    if amount.is_none() && kind.is_none() && desc.is_none() {
        return Err(crate::error::AppError::ValidationError(
            "Nothing to update. Provide amount, kind, or description.".to_string(),
        ));
    }

    if let Some(amount) = amount {
        sqlx::query("UPDATE transactions SET amount = ? WHERE id = ?")
            .bind(amount)
            .bind(id)
            .execute(pool)
            .await?;
    }

    if let Some(kind) = kind {
        sqlx::query("UPDATE transactions SET kind = ? WHERE id = ?")
            .bind(kind)
            .bind(id)
            .execute(pool)
            .await?;
    }

    if let Some(desc) = desc {
        sqlx::query("UPDATE transactions SET description = ? WHERE id = ?")
            .bind(desc)
            .bind(id)
            .execute(pool)
            .await?;
    }

    Ok(())
}

pub async fn delete_transaction(pool: &SqlitePool, id: i64) -> AppResult<()> {
    sqlx::query("DELETE FROM transactions WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

// ----- Activity DB Operations -----

pub async fn add_activity(
    pool: &SqlitePool,
    start: &str,
    stop: &str,
    desc: &str,
) -> AppResult<i64> {
    let result = sqlx::query("INSERT INTO activities (start_time, stop_time, description) VALUES (?, ?, ?)")
        .bind(start)
        .bind(stop)
        .bind(desc)
        .execute(pool)
        .await?;
    Ok(result.last_insert_rowid())
}

pub async fn get_activity(pool: &SqlitePool, id: i64) -> AppResult<Option<Activity>> {
    let row = sqlx::query("SELECT id, start_time, stop_time, description FROM activities WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?;

    match row {
        Some(row) => {
            let id: i64 = row.try_get("id")?;
            let start: String = row.try_get("start_time")?;
            let stop: String = row.try_get("stop_time")?;
            let desc: String = row.try_get("description")?;
            Ok(Some(Activity {
                id: Some(id),
                start_time: start,
                stop_time: stop,
                description: desc,
            }))
        }
        None => Ok(None),
    }
}

pub async fn list_activities(pool: &SqlitePool) -> AppResult<Vec<Activity>> {
    let rows = sqlx::query("SELECT id, start_time, stop_time, description FROM activities")
        .fetch_all(pool)
        .await?;

    let mut activities = Vec::new();
    for row in rows {
        let id: i64 = row.try_get("id")?;
        let start: String = row.try_get("start_time")?;
        let stop: String = row.try_get("stop_time")?;
        let desc: String = row.try_get("description")?;
        activities.push(Activity {
            id: Some(id),
            start_time: start,
            stop_time: stop,
            description: desc,
        });
    }
    Ok(activities)
}

pub async fn update_activity(
    pool: &SqlitePool,
    id: i64,
    start: Option<&str>,
    stop: Option<&str>,
    desc: Option<&str>,
) -> AppResult<()> {
    if start.is_none() && stop.is_none() && desc.is_none() {
        return Err(crate::error::AppError::ValidationError(
            "Nothing to update. Provide start_time, stop_time, or description.".to_string(),
        ));
    }

    if let Some(start) = start {
        sqlx::query("UPDATE activities SET start_time = ? WHERE id = ?")
            .bind(start)
            .bind(id)
            .execute(pool)
            .await?;
    }

    if let Some(stop) = stop {
        sqlx::query("UPDATE activities SET stop_time = ? WHERE id = ?")
            .bind(stop)
            .bind(id)
            .execute(pool)
            .await?;
    }

    if let Some(desc) = desc {
        sqlx::query("UPDATE activities SET description = ? WHERE id = ?")
            .bind(desc)
            .bind(id)
            .execute(pool)
            .await?;
    }

    Ok(())
}

pub async fn delete_activity(pool: &SqlitePool, id: i64) -> AppResult<()> {
    sqlx::query("DELETE FROM activities WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}
