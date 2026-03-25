use sqlx::{SqlitePool, Row};
use std::path::PathBuf;
use crate::error::AppResult;
use crate::{Transaction, Activity, Category, Place};

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

// ----- Category DB Operations -----

/// Upsert category: insert if not exists, return existing/new id
pub async fn upsert_category(pool: &SqlitePool, name: &str) -> AppResult<i64> {
    // Try to insert, ignore conflict
    sqlx::query("INSERT OR IGNORE INTO categories (name) VALUES (?)")
        .bind(name)
        .execute(pool)
        .await?;

    // Get the id (either existing or newly inserted)
    let row = sqlx::query("SELECT id FROM categories WHERE name = ?")
        .bind(name)
        .fetch_one(pool)
        .await?;

    let id: i64 = row.try_get("id")?;
    Ok(id)
}

pub async fn get_category(pool: &SqlitePool, id: i64) -> AppResult<Option<Category>> {
    let row = sqlx::query("SELECT id, name FROM categories WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?;

    match row {
        Some(row) => {
            let id: i64 = row.try_get("id")?;
            let name: String = row.try_get("name")?;
            Ok(Some(Category { id: Some(id), name }))
        }
        None => Ok(None),
    }
}

pub async fn list_categories(pool: &SqlitePool) -> AppResult<Vec<Category>> {
    let rows = sqlx::query("SELECT id, name FROM categories ORDER BY name")
        .fetch_all(pool)
        .await?;

    let mut categories = Vec::new();
    for row in rows {
        let id: i64 = row.try_get("id")?;
        let name: String = row.try_get("name")?;
        categories.push(Category { id: Some(id), name });
    }
    Ok(categories)
}

pub async fn delete_category(pool: &SqlitePool, id: i64) -> AppResult<()> {
    // First set category_id to NULL for all referencing records
    sqlx::query("UPDATE transactions SET category_id = NULL WHERE category_id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    sqlx::query("UPDATE activities SET category_id = NULL WHERE category_id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    // Then delete the category
    sqlx::query("DELETE FROM categories WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}

// ----- Place DB Operations -----

/// Upsert place: insert if not exists, return existing/new id
pub async fn upsert_place(pool: &SqlitePool, name: &str) -> AppResult<i64> {
    // Try to insert, ignore conflict
    sqlx::query("INSERT OR IGNORE INTO places (name) VALUES (?)")
        .bind(name)
        .execute(pool)
        .await?;

    // Get the id (either existing or newly inserted)
    let row = sqlx::query("SELECT id FROM places WHERE name = ?")
        .bind(name)
        .fetch_one(pool)
        .await?;

    let id: i64 = row.try_get("id")?;
    Ok(id)
}

pub async fn list_places(pool: &SqlitePool) -> AppResult<Vec<Place>> {
    let rows = sqlx::query("SELECT id, name FROM places ORDER BY name")
        .fetch_all(pool)
        .await?;

    let mut places = Vec::new();
    for row in rows {
        let id: i64 = row.try_get("id")?;
        let name: String = row.try_get("name")?;
        places.push(Place { id: Some(id), name });
    }
    Ok(places)
}

pub async fn delete_place(pool: &SqlitePool, id: i64) -> AppResult<()> {
    // First set place_id to NULL for all referencing records
    sqlx::query("UPDATE transactions SET place_id = NULL WHERE place_id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    sqlx::query("UPDATE activities SET place_id = NULL WHERE place_id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    // Then delete the place
    sqlx::query("DELETE FROM places WHERE id = ?")
        .bind(id)
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
    category_id: Option<i64>,
    place_id: Option<i64>,
) -> AppResult<i64> {
    let result = sqlx::query("INSERT INTO transactions (amount, kind, description, timestamp, category_id, place_id) VALUES (?, ?, ?, ?, ?, ?)")
        .bind(amount)
        .bind(kind)
        .bind(description)
        .bind(chrono::Utc::now().to_rfc3339())
        .bind(category_id)
        .bind(place_id)
        .execute(pool)
        .await?;
    Ok(result.last_insert_rowid())
}

pub async fn get_transaction(pool: &SqlitePool, id: i64) -> AppResult<Option<Transaction>> {
    let row = sqlx::query("SELECT id, amount, kind, description, timestamp, category_id, place_id FROM transactions WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?;

    match row {
        Some(row) => {
            let id: i64 = row.try_get("id")?;
            let amount: f64 = row.try_get("amount")?;
            let kind: String = row.try_get("kind")?;
            let desc: String = row.try_get("description")?;
            let cat_id: Option<i64> = row.try_get("category_id").ok().flatten();
            let place_id: Option<i64> = row.try_get("place_id").ok().flatten();
            Ok(Some(Transaction {
                id: Some(id),
                amount,
                kind,
                description: desc,
                category_id: cat_id,
                place_id,
                category_name: None,
                place_name: None,
            }))
        }
        None => Ok(None),
    }
}

pub async fn list_transactions(
    pool: &SqlitePool,
    kind_filter: Option<&str>,
    category_id: Option<i64>,
    place_id: Option<i64>,
) -> AppResult<Vec<Transaction>> {
    let mut query = String::from("SELECT id, amount, kind, description, timestamp, category_id, place_id FROM transactions");
    let mut conditions = Vec::new();
    
    if kind_filter.is_some() {
        conditions.push("kind = ?");
    }
    if category_id.is_some() {
        conditions.push("category_id = ?");
    }
    if place_id.is_some() {
        conditions.push("place_id = ?");
    }
    
    if !conditions.is_empty() {
        query.push_str(" WHERE ");
        query.push_str(&conditions.join(" AND "));
    }

    let rows = if kind_filter.is_some() && category_id.is_some() && place_id.is_some() {
        sqlx::query(&query)
            .bind(kind_filter.unwrap())
            .bind(category_id.unwrap())
            .bind(place_id.unwrap())
            .fetch_all(pool)
            .await?
    } else if kind_filter.is_some() && category_id.is_some() {
        sqlx::query(&query)
            .bind(kind_filter.unwrap())
            .bind(category_id.unwrap())
            .fetch_all(pool)
            .await?
    } else if kind_filter.is_some() && place_id.is_some() {
        sqlx::query(&query)
            .bind(kind_filter.unwrap())
            .bind(place_id.unwrap())
            .fetch_all(pool)
            .await?
    } else if category_id.is_some() && place_id.is_some() {
        sqlx::query(&query)
            .bind(category_id.unwrap())
            .bind(place_id.unwrap())
            .fetch_all(pool)
            .await?
    } else if kind_filter.is_some() {
        sqlx::query(&query)
            .bind(kind_filter.unwrap())
            .fetch_all(pool)
            .await?
    } else if category_id.is_some() {
        sqlx::query(&query)
            .bind(category_id.unwrap())
            .fetch_all(pool)
            .await?
    } else if place_id.is_some() {
        sqlx::query(&query)
            .bind(place_id.unwrap())
            .fetch_all(pool)
            .await?
    } else {
        sqlx::query(&query).fetch_all(pool).await?
    };

    let mut transactions = Vec::new();
    for row in rows {
        let id: i64 = row.try_get("id")?;
        let amount: f64 = row.try_get("amount")?;
        let kind: String = row.try_get("kind")?;
        let desc: String = row.try_get("description")?;
        let cat_id: Option<i64> = row.try_get("category_id").ok().flatten();
        let place_id: Option<i64> = row.try_get("place_id").ok().flatten();
        transactions.push(Transaction {
            id: Some(id),
            amount,
            kind,
            description: desc,
            category_id: cat_id,
            place_id,
            category_name: None,
            place_name: None,
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
    category_id: Option<i64>,
    place_id: Option<i64>,
) -> AppResult<()> {
    if amount.is_none() && kind.is_none() && desc.is_none() && category_id.is_none() && place_id.is_none() {
        return Err(crate::error::AppError::ValidationError(
            "Nothing to update. Provide amount, kind, description, category, or place.".to_string(),
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

    if let Some(cat_id) = category_id {
        sqlx::query("UPDATE transactions SET category_id = ? WHERE id = ?")
            .bind(cat_id)
            .bind(id)
            .execute(pool)
            .await?;
    }

    if let Some(p_id) = place_id {
        sqlx::query("UPDATE transactions SET place_id = ? WHERE id = ?")
            .bind(p_id)
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
    category_id: Option<i64>,
    place_id: Option<i64>,
) -> AppResult<i64> {
    let result = sqlx::query("INSERT INTO activities (start_time, stop_time, description, category_id, place_id) VALUES (?, ?, ?, ?, ?)")
        .bind(start)
        .bind(stop)
        .bind(desc)
        .bind(category_id)
        .bind(place_id)
        .execute(pool)
        .await?;
    Ok(result.last_insert_rowid())
}

pub async fn get_activity(pool: &SqlitePool, id: i64) -> AppResult<Option<Activity>> {
    let row = sqlx::query("SELECT id, start_time, stop_time, description, category_id, place_id FROM activities WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?;

    match row {
        Some(row) => {
            let id: i64 = row.try_get("id")?;
            let start: String = row.try_get("start_time")?;
            let stop: String = row.try_get("stop_time")?;
            let desc: String = row.try_get("description")?;
            let cat_id: Option<i64> = row.try_get("category_id").ok().flatten();
            let place_id: Option<i64> = row.try_get("place_id").ok().flatten();
            Ok(Some(Activity {
                id: Some(id),
                start_time: start,
                stop_time: stop,
                description: desc,
                category_id: cat_id,
                place_id,
                category_name: None,
                place_name: None,
            }))
        }
        None => Ok(None),
    }
}

pub async fn list_activities(
    pool: &SqlitePool,
    category_id: Option<i64>,
    place_id: Option<i64>,
) -> AppResult<Vec<Activity>> {
    let mut query = String::from("SELECT id, start_time, stop_time, description, category_id, place_id FROM activities");
    let mut conditions = Vec::new();
    
    if category_id.is_some() {
        conditions.push("category_id = ?");
    }
    if place_id.is_some() {
        conditions.push("place_id = ?");
    }
    
    if !conditions.is_empty() {
        query.push_str(" WHERE ");
        query.push_str(&conditions.join(" AND "));
    }

    let rows = if category_id.is_some() && place_id.is_some() {
        sqlx::query(&query)
            .bind(category_id.unwrap())
            .bind(place_id.unwrap())
            .fetch_all(pool)
            .await?
    } else if category_id.is_some() {
        sqlx::query(&query)
            .bind(category_id.unwrap())
            .fetch_all(pool)
            .await?
    } else if place_id.is_some() {
        sqlx::query(&query)
            .bind(place_id.unwrap())
            .fetch_all(pool)
            .await?
    } else {
        sqlx::query(&query).fetch_all(pool).await?
    };

    let mut activities = Vec::new();
    for row in rows {
        let id: i64 = row.try_get("id")?;
        let start: String = row.try_get("start_time")?;
        let stop: String = row.try_get("stop_time")?;
        let desc: String = row.try_get("description")?;
        let cat_id: Option<i64> = row.try_get("category_id").ok().flatten();
        let place_id: Option<i64> = row.try_get("place_id").ok().flatten();
        activities.push(Activity {
            id: Some(id),
            start_time: start,
            stop_time: stop,
            description: desc,
            category_id: cat_id,
            place_id,
            category_name: None,
            place_name: None,
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
    category_id: Option<i64>,
    place_id: Option<i64>,
) -> AppResult<()> {
    if start.is_none() && stop.is_none() && desc.is_none() && category_id.is_none() && place_id.is_none() {
        return Err(crate::error::AppError::ValidationError(
            "Nothing to update. Provide start_time, stop_time, description, category, or place.".to_string(),
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

    if let Some(cat_id) = category_id {
        sqlx::query("UPDATE activities SET category_id = ? WHERE id = ?")
            .bind(cat_id)
            .bind(id)
            .execute(pool)
            .await?;
    }

    if let Some(p_id) = place_id {
        sqlx::query("UPDATE activities SET place_id = ? WHERE id = ?")
            .bind(p_id)
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
