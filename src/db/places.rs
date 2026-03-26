use sqlx::{SqlitePool, Row};
use crate::error::AppResult;
use crate::types::Place;

pub async fn upsert_place(pool: &SqlitePool, name: &str) -> AppResult<i64> {
    sqlx::query("INSERT OR IGNORE INTO places (name) VALUES (?)")
        .bind(name)
        .execute(pool)
        .await?;

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
    sqlx::query("UPDATE transactions SET place_id = NULL WHERE place_id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    sqlx::query("UPDATE activities SET place_id = NULL WHERE place_id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    sqlx::query("DELETE FROM places WHERE id = ?")
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
    async fn test_upsert_place_new() {
        let pool = create_test_pool().await;
        let id = upsert_place(&pool, "Test Place").await.unwrap();
        assert!(id > 0);
    }

    #[tokio::test]
    async fn test_upsert_place_duplicate() {
        let pool = create_test_pool().await;
        let id1 = upsert_place(&pool, "Duplicate").await.unwrap();
        let id2 = upsert_place(&pool, "Duplicate").await.unwrap();
        assert_eq!(id1, id2);
    }

    #[tokio::test]
    async fn test_list_places_sorted() {
        let pool = create_test_pool().await;
        upsert_place(&pool, "Zoo").await.unwrap();
        upsert_place(&pool, "Airport").await.unwrap();
        upsert_place(&pool, "Mall").await.unwrap();
        
        let places = list_places(&pool).await.unwrap();
        assert_eq!(places.len(), 3);
        assert_eq!(places[0].name, "Airport");
        assert_eq!(places[1].name, "Mall");
        assert_eq!(places[2].name, "Zoo");
    }

    #[tokio::test]
    async fn test_delete_place() {
        let pool = create_test_pool().await;
        let id = upsert_place(&pool, "To Delete").await.unwrap();
        delete_place(&pool, id).await.unwrap();
        let places = list_places(&pool).await.unwrap();
        assert!(places.is_empty());
    }
}
