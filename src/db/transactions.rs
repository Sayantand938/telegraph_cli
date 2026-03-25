use sqlx::{SqlitePool, Row};
use crate::error::AppResult;
use crate::types::Transaction;
use super::{get_transaction_tags, get_transaction_persons};

pub async fn add_transaction(
    pool: &SqlitePool,
    amount: f64,
    kind: &str,
    description: &str,
    category_id: Option<i64>,
    place_id: Option<i64>,
) -> AppResult<i64> {
    let result = sqlx::query(
        "INSERT INTO transactions (amount, kind, description, timestamp, category_id, place_id) VALUES (?, ?, ?, ?, ?, ?)"
    )
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
    let row = sqlx::query(
        "SELECT id, amount, kind, description, timestamp, category_id, place_id FROM transactions WHERE id = ?"
    )
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
    let mut query = String::from(
        "SELECT id, amount, kind, description, timestamp, category_id, place_id FROM transactions"
    );
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
