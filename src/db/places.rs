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
