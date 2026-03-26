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

pub async fn set_todo_tags(pool: &SqlitePool, todo_id: i64, tag_ids: &[i64]) -> AppResult<()> {
    sqlx::query("DELETE FROM todo_tags WHERE todo_id = ?")
        .bind(todo_id)
        .execute(pool)
        .await?;

    for tag_id in tag_ids {
        sqlx::query("INSERT INTO todo_tags (todo_id, tag_id) VALUES (?, ?)")
            .bind(todo_id)
            .bind(tag_id)
            .execute(pool)
            .await?;
    }

    Ok(())
}

pub async fn get_todo_tags(pool: &SqlitePool, todo_id: i64) -> AppResult<Vec<Tag>> {
    let rows = sqlx::query(
        "SELECT t.id, t.name FROM tags t
         INNER JOIN todo_tags tt ON t.id = tt.tag_id
         WHERE tt.todo_id = ?"
    )
    .bind(todo_id)
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

pub async fn set_journal_tags(pool: &SqlitePool, journal_id: i64, tag_ids: &[i64]) -> AppResult<()> {
    sqlx::query("DELETE FROM journal_tags WHERE journal_id = ?")
        .bind(journal_id)
        .execute(pool)
        .await?;

    for tag_id in tag_ids {
        sqlx::query("INSERT INTO journal_tags (journal_id, tag_id) VALUES (?, ?)")
            .bind(journal_id)
            .bind(tag_id)
            .execute(pool)
            .await?;
    }

    Ok(())
}

pub async fn get_journal_tags(pool: &SqlitePool, journal_id: i64) -> AppResult<Vec<Tag>> {
    let rows = sqlx::query(
        "SELECT t.id, t.name FROM tags t
         INNER JOIN journal_tags jt ON t.id = jt.tag_id
         WHERE jt.journal_id = ?"
    )
    .bind(journal_id)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::init_tables;
    use crate::db::transactions::add_transaction;

    async fn create_test_pool() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        init_tables(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn test_upsert_tag_new() {
        let pool = create_test_pool().await;
        let id = upsert_tag(&pool, "Test Tag").await.unwrap();
        assert!(id > 0);
    }

    #[tokio::test]
    async fn test_upsert_tag_duplicate() {
        let pool = create_test_pool().await;
        let id1 = upsert_tag(&pool, "Duplicate").await.unwrap();
        let id2 = upsert_tag(&pool, "Duplicate").await.unwrap();
        assert_eq!(id1, id2);
    }

    #[tokio::test]
    async fn test_list_tags_sorted() {
        let pool = create_test_pool().await;
        upsert_tag(&pool, "Zebra").await.unwrap();
        upsert_tag(&pool, "Apple").await.unwrap();
        upsert_tag(&pool, "Mango").await.unwrap();
        
        let tags = list_tags(&pool).await.unwrap();
        assert_eq!(tags.len(), 3);
        assert_eq!(tags[0].name, "Apple");
        assert_eq!(tags[1].name, "Mango");
        assert_eq!(tags[2].name, "Zebra");
    }

    #[tokio::test]
    async fn test_set_transaction_tags() {
        let pool = create_test_pool().await;
        
        // Create a transaction
        let tx_id = add_transaction(&pool, 100.0, "test", "Test", None, None).await.unwrap();
        
        // Create tags
        let tag1_id = upsert_tag(&pool, "Tag1").await.unwrap();
        let tag2_id = upsert_tag(&pool, "Tag2").await.unwrap();
        
        // Set tags
        set_transaction_tags(&pool, tx_id, &[tag1_id, tag2_id]).await.unwrap();
        
        // Get tags
        let tags = get_transaction_tags(&pool, tx_id).await.unwrap();
        assert_eq!(tags.len(), 2);
    }

    #[tokio::test]
    async fn test_delete_tag() {
        let pool = create_test_pool().await;
        let id = upsert_tag(&pool, "To Delete").await.unwrap();
        delete_tag(&pool, id).await.unwrap();
        let tags = list_tags(&pool).await.unwrap();
        assert!(tags.is_empty());
    }
}
