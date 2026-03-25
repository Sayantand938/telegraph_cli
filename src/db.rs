use sqlx::{SqlitePool, Row};
use std::path::PathBuf;
use crate::error::AppResult;
use crate::{Transaction, Activity, Category, Place, Tag, Person};

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

// ----- Tag DB Operations -----

pub async fn upsert_tag(pool: &SqlitePool, name: &str) -> AppResult<i64> {
    sqlx::query("INSERT OR IGNORE INTO tags (name) VALUES (?)")
        .bind(name)
        .execute(pool)
        .await?;

    let row = sqlx::query("SELECT id FROM tags WHERE name = ?")
        .bind(name)
        .fetch_one(pool)
        .await?;

    let id: i64 = row.try_get("id")?;
    Ok(id)
}

pub async fn list_tags(pool: &SqlitePool) -> AppResult<Vec<Tag>> {
    let rows = sqlx::query("SELECT id, name FROM tags ORDER BY name")
        .fetch_all(pool)
        .await?;

    let mut tags = Vec::new();
    for row in rows {
        let id: i64 = row.try_get("id")?;
        let name: String = row.try_get("name")?;
        tags.push(Tag { id: Some(id), name });
    }
    Ok(tags)
}

pub async fn delete_tag(pool: &SqlitePool, id: i64) -> AppResult<()> {
    sqlx::query("DELETE FROM transaction_tags WHERE tag_id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    sqlx::query("DELETE FROM activity_tags WHERE tag_id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    sqlx::query("DELETE FROM tags WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn set_transaction_tags(pool: &SqlitePool, transaction_id: i64, tag_ids: &[i64]) -> AppResult<()> {
    // Delete existing tags
    sqlx::query("DELETE FROM transaction_tags WHERE transaction_id = ?")
        .bind(transaction_id)
        .execute(pool)
        .await?;

    // Insert new tags
    for tag_id in tag_ids {
        sqlx::query("INSERT INTO transaction_tags (transaction_id, tag_id) VALUES (?, ?)")
            .bind(transaction_id)
            .bind(tag_id)
            .execute(pool)
            .await?;
    }

    Ok(())
}

pub async fn set_activity_tags(pool: &SqlitePool, activity_id: i64, tag_ids: &[i64]) -> AppResult<()> {
    sqlx::query("DELETE FROM activity_tags WHERE activity_id = ?")
        .bind(activity_id)
        .execute(pool)
        .await?;

    for tag_id in tag_ids {
        sqlx::query("INSERT INTO activity_tags (activity_id, tag_id) VALUES (?, ?)")
            .bind(activity_id)
            .bind(tag_id)
            .execute(pool)
            .await?;
    }

    Ok(())
}

pub async fn get_transaction_tags(pool: &SqlitePool, transaction_id: i64) -> AppResult<Vec<Tag>> {
    let rows = sqlx::query(
        "SELECT t.id, t.name FROM tags t 
         INNER JOIN transaction_tags tt ON t.id = tt.tag_id 
         WHERE tt.transaction_id = ?"
    )
    .bind(transaction_id)
    .fetch_all(pool)
    .await?;

    let mut tags = Vec::new();
    for row in rows {
        let id: i64 = row.try_get("id")?;
        let name: String = row.try_get("name")?;
        tags.push(Tag { id: Some(id), name });
    }
    Ok(tags)
}

pub async fn get_activity_tags(pool: &SqlitePool, activity_id: i64) -> AppResult<Vec<Tag>> {
    let rows = sqlx::query(
        "SELECT t.id, t.name FROM tags t 
         INNER JOIN activity_tags at ON t.id = at.tag_id 
         WHERE at.activity_id = ?"
    )
    .bind(activity_id)
    .fetch_all(pool)
    .await?;

    let mut tags = Vec::new();
    for row in rows {
        let id: i64 = row.try_get("id")?;
        let name: String = row.try_get("name")?;
        tags.push(Tag { id: Some(id), name });
    }
    Ok(tags)
}

// ----- Person DB Operations -----

pub async fn upsert_person(pool: &SqlitePool, name: &str) -> AppResult<i64> {
    sqlx::query("INSERT OR IGNORE INTO persons (name) VALUES (?)")
        .bind(name)
        .execute(pool)
        .await?;

    let row = sqlx::query("SELECT id FROM persons WHERE name = ?")
        .bind(name)
        .fetch_one(pool)
        .await?;

    let id: i64 = row.try_get("id")?;
    Ok(id)
}

pub async fn list_persons(pool: &SqlitePool) -> AppResult<Vec<Person>> {
    let rows = sqlx::query("SELECT id, name FROM persons ORDER BY name")
        .fetch_all(pool)
        .await?;

    let mut persons = Vec::new();
    for row in rows {
        let id: i64 = row.try_get("id")?;
        let name: String = row.try_get("name")?;
        persons.push(Person { id: Some(id), name });
    }
    Ok(persons)
}

pub async fn delete_person(pool: &SqlitePool, id: i64) -> AppResult<()> {
    sqlx::query("DELETE FROM transaction_persons WHERE person_id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    sqlx::query("DELETE FROM activity_persons WHERE person_id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    sqlx::query("DELETE FROM persons WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn set_transaction_persons(pool: &SqlitePool, transaction_id: i64, person_ids: &[i64]) -> AppResult<()> {
    sqlx::query("DELETE FROM transaction_persons WHERE transaction_id = ?")
        .bind(transaction_id)
        .execute(pool)
        .await?;

    for person_id in person_ids {
        sqlx::query("INSERT INTO transaction_persons (transaction_id, person_id) VALUES (?, ?)")
            .bind(transaction_id)
            .bind(person_id)
            .execute(pool)
            .await?;
    }

    Ok(())
}

pub async fn set_activity_persons(pool: &SqlitePool, activity_id: i64, person_ids: &[i64]) -> AppResult<()> {
    sqlx::query("DELETE FROM activity_persons WHERE activity_id = ?")
        .bind(activity_id)
        .execute(pool)
        .await?;

    for person_id in person_ids {
        sqlx::query("INSERT INTO activity_persons (activity_id, person_id) VALUES (?, ?)")
            .bind(activity_id)
            .bind(person_id)
            .execute(pool)
            .await?;
    }

    Ok(())
}

pub async fn get_transaction_persons(pool: &SqlitePool, transaction_id: i64) -> AppResult<Vec<Person>> {
    let rows = sqlx::query(
        "SELECT p.id, p.name FROM persons p 
         INNER JOIN transaction_persons tp ON p.id = tp.person_id 
         WHERE tp.transaction_id = ?"
    )
    .bind(transaction_id)
    .fetch_all(pool)
    .await?;

    let mut persons = Vec::new();
    for row in rows {
        let id: i64 = row.try_get("id")?;
        let name: String = row.try_get("name")?;
        persons.push(Person { id: Some(id), name });
    }
    Ok(persons)
}

pub async fn get_activity_persons(pool: &SqlitePool, activity_id: i64) -> AppResult<Vec<Person>> {
    let rows = sqlx::query(
        "SELECT p.id, p.name FROM persons p 
         INNER JOIN activity_persons ap ON p.id = ap.person_id 
         WHERE ap.activity_id = ?"
    )
    .bind(activity_id)
    .fetch_all(pool)
    .await?;

    let mut persons = Vec::new();
    for row in rows {
        let id: i64 = row.try_get("id")?;
        let name: String = row.try_get("name")?;
        persons.push(Person { id: Some(id), name });
    }
    Ok(persons)
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
            
            // Fetch tags and persons
            let tags = get_transaction_tags(pool, id).await?;
            let persons = get_transaction_persons(pool, id).await?;
            
            Ok(Some(Transaction {
                id: Some(id),
                amount,
                kind,
                description: desc,
                category_id: cat_id,
                place_id,
                category_name: None,
                place_name: None,
                tag_names: tags.into_iter().map(|t| t.name).collect(),
                person_names: persons.into_iter().map(|p| p.name).collect(),
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
            tag_names: Vec::new(),
            person_names: Vec::new(),
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
            
            // Fetch tags and persons
            let tags = get_activity_tags(pool, id).await?;
            let persons = get_activity_persons(pool, id).await?;
            
            Ok(Some(Activity {
                id: Some(id),
                start_time: start,
                stop_time: stop,
                description: desc,
                category_id: cat_id,
                place_id,
                category_name: None,
                place_name: None,
                tag_names: tags.into_iter().map(|t| t.name).collect(),
                person_names: persons.into_iter().map(|p| p.name).collect(),
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
            tag_names: Vec::new(),
            person_names: Vec::new(),
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
