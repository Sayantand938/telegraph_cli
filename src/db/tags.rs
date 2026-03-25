use sqlx::{SqlitePool, Row};
use crate::error::AppResult;
use crate::types::Tag;

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
    sqlx::query("DELETE FROM transaction_tags WHERE transaction_id = ?")
        .bind(transaction_id)
        .execute(pool)
        .await?;

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
