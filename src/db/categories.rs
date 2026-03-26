use sqlx::{SqlitePool, Row};
use crate::error::AppResult;
use crate::types::Category;

pub async fn upsert_category(pool: &SqlitePool, name: &str) -> AppResult<i64> {
    sqlx::query("INSERT OR IGNORE INTO categories (name) VALUES (?)")
        .bind(name)
        .execute(pool)
        .await?;

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
    sqlx::query("UPDATE transactions SET category_id = NULL WHERE category_id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    sqlx::query("UPDATE activities SET category_id = NULL WHERE category_id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    sqlx::query("DELETE FROM categories WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::init_tables;

    async fn create_test_pool() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        init_tables(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn test_upsert_category_new() {
        let pool = create_test_pool().await;
        let id = upsert_category(&pool, "Test Category").await.unwrap();
        assert!(id > 0);
    }

    #[tokio::test]
    async fn test_upsert_category_duplicate() {
        let pool = create_test_pool().await;
        let id1 = upsert_category(&pool, "Duplicate").await.unwrap();
        let id2 = upsert_category(&pool, "Duplicate").await.unwrap();
        assert_eq!(id1, id2);
    }

    #[tokio::test]
    async fn test_list_categories_empty() {
        let pool = create_test_pool().await;
        let categories = list_categories(&pool).await.unwrap();
        assert!(categories.is_empty());
    }

    #[tokio::test]
    async fn test_list_categories_sorted() {
        let pool = create_test_pool().await;
        upsert_category(&pool, "Zebra").await.unwrap();
        upsert_category(&pool, "Apple").await.unwrap();
        upsert_category(&pool, "Mango").await.unwrap();
        
        let categories = list_categories(&pool).await.unwrap();
        assert_eq!(categories.len(), 3);
        assert_eq!(categories[0].name, "Apple");
        assert_eq!(categories[1].name, "Mango");
        assert_eq!(categories[2].name, "Zebra");
    }

    #[tokio::test]
    async fn test_delete_category() {
        let pool = create_test_pool().await;
        let id = upsert_category(&pool, "To Delete").await.unwrap();
        
        let categories = list_categories(&pool).await.unwrap();
        assert_eq!(categories.len(), 1);
        
        delete_category(&pool, id).await.unwrap();
        
        let categories = list_categories(&pool).await.unwrap();
        assert!(categories.is_empty());
    }
}
