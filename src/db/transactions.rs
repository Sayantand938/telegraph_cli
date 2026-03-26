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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{init_tables, upsert_category, upsert_place};

    async fn create_test_pool() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        init_tables(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn test_add_transaction_basic() {
        let pool = create_test_pool().await;
        let id = add_transaction(&pool, 100.0, "shopping", "Test purchase", None, None).await.unwrap();
        assert!(id > 0);
    }

    #[tokio::test]
    async fn test_add_transaction_with_category() {
        let pool = create_test_pool().await;
        let cat_id = upsert_category(&pool, "Food").await.unwrap();
        let tx_id = add_transaction(&pool, 50.0, "shopping", "Groceries", Some(cat_id), None).await.unwrap();
        
        let tx = get_transaction(&pool, tx_id).await.unwrap().unwrap();
        assert_eq!(tx.category_id, Some(cat_id));
    }

    #[tokio::test]
    async fn test_add_transaction_with_place() {
        let pool = create_test_pool().await;
        let place_id = upsert_place(&pool, "Supermarket").await.unwrap();
        let tx_id = add_transaction(&pool, 75.0, "shopping", "Groceries", None, Some(place_id)).await.unwrap();
        
        let tx = get_transaction(&pool, tx_id).await.unwrap().unwrap();
        assert_eq!(tx.place_id, Some(place_id));
    }

    #[tokio::test]
    async fn test_get_transaction_not_found() {
        let pool = create_test_pool().await;
        let result = get_transaction(&pool, 999).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_list_transactions_all() {
        let pool = create_test_pool().await;
        add_transaction(&pool, 10.0, "test", "First", None, None).await.unwrap();
        add_transaction(&pool, 20.0, "test", "Second", None, None).await.unwrap();
        add_transaction(&pool, 30.0, "other", "Third", None, None).await.unwrap();
        
        let transactions = list_transactions(&pool, None, None, None).await.unwrap();
        assert_eq!(transactions.len(), 3);
    }

    #[tokio::test]
    async fn test_list_transactions_by_kind() {
        let pool = create_test_pool().await;
        add_transaction(&pool, 10.0, "shopping", "First", None, None).await.unwrap();
        add_transaction(&pool, 20.0, "entertainment", "Second", None, None).await.unwrap();
        add_transaction(&pool, 30.0, "shopping", "Third", None, None).await.unwrap();
        
        let transactions = list_transactions(&pool, Some("shopping"), None, None).await.unwrap();
        assert_eq!(transactions.len(), 2);
        for tx in &transactions {
            assert_eq!(tx.kind, "shopping");
        }
    }

    #[tokio::test]
    async fn test_update_transaction_amount() {
        let pool = create_test_pool().await;
        let id = add_transaction(&pool, 100.0, "test", "Original", None, None).await.unwrap();
        
        update_transaction(&pool, id, Some(200.0), None, None, None, None).await.unwrap();
        
        let tx = get_transaction(&pool, id).await.unwrap().unwrap();
        assert_eq!(tx.amount, 200.0);
    }

    #[tokio::test]
    async fn test_update_transaction_no_fields() {
        let pool = create_test_pool().await;
        let id = add_transaction(&pool, 100.0, "test", "Original", None, None).await.unwrap();
        
        let result = update_transaction(&pool, id, None, None, None, None, None).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Nothing to update"));
    }

    #[tokio::test]
    async fn test_delete_transaction() {
        let pool = create_test_pool().await;
        let id = add_transaction(&pool, 100.0, "test", "To Delete", None, None).await.unwrap();
        
        delete_transaction(&pool, id).await.unwrap();
        
        let tx = get_transaction(&pool, id).await.unwrap();
        assert!(tx.is_none());
    }
}
