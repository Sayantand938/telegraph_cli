use sqlx::{SqlitePool, Row};
use crate::error::AppResult;
use crate::types::JournalEntry;
use super::{get_journal_tags, get_journal_persons};

pub async fn add_journal_entry(
    pool: &SqlitePool,
    content: &str,
    date: Option<&str>,
    category_id: Option<i64>,
    place_id: Option<i64>,
) -> AppResult<i64> {
    let result = sqlx::query(
        "INSERT INTO journal_entries (content, date, category_id, place_id, created_at) 
         VALUES (?, ?, ?, ?, ?)"
    )
    .bind(content)
    .bind(date)
    .bind(category_id)
    .bind(place_id)
    .bind(chrono::Utc::now().to_rfc3339())
    .execute(pool)
    .await?;
    Ok(result.last_insert_rowid())
}

pub async fn get_journal_entry(pool: &SqlitePool, id: i64) -> AppResult<Option<JournalEntry>> {
    let row = sqlx::query(
        "SELECT id, content, date, category_id, place_id, created_at FROM journal_entries WHERE id = ?"
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    match row {
        Some(row) => {
            let id: i64 = row.try_get("id")?;
            let content: String = row.try_get("content")?;
            let date: Option<String> = row.try_get("date").ok().flatten();
            let cat_id: Option<i64> = row.try_get("category_id").ok().flatten();
            let place_id: Option<i64> = row.try_get("place_id").ok().flatten();
            let created_at: String = row.try_get("created_at")?;

            // Fetch tags and persons
            let tags = get_journal_tags(pool, id).await?;
            let persons = get_journal_persons(pool, id).await?;

            Ok(Some(JournalEntry {
                id: Some(id),
                content,
                date,
                category_id: cat_id,
                place_id,
                category_name: None,
                place_name: None,
                tag_names: tags.into_iter().map(|t| t.name).collect(),
                person_names: persons.into_iter().map(|p| p.name).collect(),
                created_at,
            }))
        }
        None => Ok(None),
    }
}

pub async fn list_journal_entries(
    pool: &SqlitePool,
    from_date: Option<&str>,
    to_date: Option<&str>,
    category_id: Option<i64>,
) -> AppResult<Vec<JournalEntry>> {
    let mut query = String::from(
        "SELECT id, content, date, category_id, place_id, created_at FROM journal_entries WHERE 1=1"
    );
    
    if from_date.is_some() {
        query.push_str(" AND (date IS NULL OR date >= ?)");
    }
    if to_date.is_some() {
        query.push_str(" AND (date IS NULL OR date <= ?)");
    }
    if category_id.is_some() {
        query.push_str(" AND category_id = ?");
    }
    
    query.push_str(" ORDER BY date DESC, created_at DESC");

    let mut db_query = sqlx::query_as::<_, (i64, String, Option<String>, Option<i64>, Option<i64>, String)>(&query);
    
    if let Some(from) = from_date {
        db_query = db_query.bind(from);
    }
    if let Some(to) = to_date {
        db_query = db_query.bind(to);
    }
    if let Some(cat_id) = category_id {
        db_query = db_query.bind(cat_id);
    }

    let rows = db_query.fetch_all(pool).await?;

    let mut entries = Vec::new();
    for row in rows {
        entries.push(JournalEntry {
            id: Some(row.0),
            content: row.1,
            date: row.2,
            category_id: row.3,
            place_id: row.4,
            category_name: None,
            place_name: None,
            tag_names: Vec::new(),
            person_names: Vec::new(),
            created_at: row.5,
        });
    }
    Ok(entries)
}

pub async fn search_journal_entries(
    pool: &SqlitePool,
    query: &str,
    from_date: Option<&str>,
    to_date: Option<&str>,
) -> AppResult<Vec<JournalEntry>> {
    // Use FTS5 for full-text search
    let mut sql = String::from(
        "SELECT je.id, je.content, je.date, je.category_id, je.place_id, je.created_at 
         FROM journal_entries je
         INNER JOIN journal_fts fts ON je.id = fts.rowid
         WHERE journal_fts MATCH ?"
    );

    if from_date.is_some() || to_date.is_some() {
        sql.push_str(" AND 1=1");
        if let Some(from) = from_date {
            sql.push_str(&format!(" AND (je.date IS NULL OR je.date >= '{}')", from));
        }
        if let Some(to) = to_date {
            sql.push_str(&format!(" AND (je.date IS NULL OR je.date <= '{}')", to));
        }
    }

    sql.push_str(" ORDER BY rank, je.date DESC, je.created_at DESC");

    let rows = sqlx::query(&sql)
        .bind(query)
        .fetch_all(pool)
        .await?;

    let mut entries = Vec::new();
    for row in rows {
        let id: i64 = row.try_get("id")?;
        let content: String = row.try_get("content")?;
        let date: Option<String> = row.try_get("date").ok().flatten();
        let cat_id: Option<i64> = row.try_get("category_id").ok().flatten();
        let place_id: Option<i64> = row.try_get("place_id").ok().flatten();
        let created_at: String = row.try_get("created_at")?;

        entries.push(JournalEntry {
            id: Some(id),
            content,
            date,
            category_id: cat_id,
            place_id,
            category_name: None,
            place_name: None,
            tag_names: Vec::new(),
            person_names: Vec::new(),
            created_at,
        });
    }
    Ok(entries)
}

pub async fn update_journal_entry(
    pool: &SqlitePool,
    id: i64,
    content: Option<&str>,
    date: Option<&str>,
    category_id: Option<i64>,
    place_id: Option<i64>,
) -> AppResult<()> {
    if content.is_none() && date.is_none() && category_id.is_none() && place_id.is_none() {
        return Err(crate::error::AppError::ValidationError(
            "Nothing to update. Provide content, date, category, or place.".to_string(),
        ));
    }

    if let Some(c) = content {
        sqlx::query("UPDATE journal_entries SET content = ? WHERE id = ?")
            .bind(c)
            .bind(id)
            .execute(pool)
            .await?;
    }

    if let Some(d) = date {
        sqlx::query("UPDATE journal_entries SET date = ? WHERE id = ?")
            .bind(d)
            .bind(id)
            .execute(pool)
            .await?;
    }

    if let Some(cat_id) = category_id {
        sqlx::query("UPDATE journal_entries SET category_id = ? WHERE id = ?")
            .bind(cat_id)
            .bind(id)
            .execute(pool)
            .await?;
    }

    if let Some(p_id) = place_id {
        sqlx::query("UPDATE journal_entries SET place_id = ? WHERE id = ?")
            .bind(p_id)
            .bind(id)
            .execute(pool)
            .await?;
    }

    Ok(())
}

pub async fn delete_journal_entry(pool: &SqlitePool, id: i64) -> AppResult<()> {
    sqlx::query("DELETE FROM journal_entries WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{init_tables, upsert_category};

    async fn create_test_pool() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        init_tables(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn test_add_journal_entry_basic() {
        let pool = create_test_pool().await;
        let id = add_journal_entry(&pool, "Test entry", None, None, None).await.unwrap();
        assert!(id > 0);
    }

    #[tokio::test]
    async fn test_add_journal_entry_with_date() {
        let pool = create_test_pool().await;
        let id = add_journal_entry(&pool, "Past event", Some("2026-03-20"), None, None).await.unwrap();
        
        let entry = get_journal_entry(&pool, id).await.unwrap().unwrap();
        assert_eq!(entry.date, Some("2026-03-20".to_string()));
    }

    #[tokio::test]
    async fn test_add_journal_entry_with_category() {
        let pool = create_test_pool().await;
        let cat_id = upsert_category(&pool, "Personal").await.unwrap();
        let entry_id = add_journal_entry(&pool, "Family note", None, Some(cat_id), None).await.unwrap();
        
        let entry = get_journal_entry(&pool, entry_id).await.unwrap().unwrap();
        assert_eq!(entry.category_id, Some(cat_id));
    }

    #[tokio::test]
    async fn test_get_journal_entry_not_found() {
        let pool = create_test_pool().await;
        let result = get_journal_entry(&pool, 999).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_list_journal_entries() {
        let pool = create_test_pool().await;
        add_journal_entry(&pool, "First", Some("2026-03-20"), None, None).await.unwrap();
        add_journal_entry(&pool, "Second", Some("2026-03-21"), None, None).await.unwrap();
        add_journal_entry(&pool, "Third", None, None, None).await.unwrap();
        
        let entries = list_journal_entries(&pool, None, None, None).await.unwrap();
        assert_eq!(entries.len(), 3);
    }

    #[tokio::test]
    async fn test_list_journal_entries_by_date() {
        let pool = create_test_pool().await;
        add_journal_entry(&pool, "Old", Some("2026-03-20"), None, None).await.unwrap();
        add_journal_entry(&pool, "New", Some("2026-03-25"), None, None).await.unwrap();
        
        let entries = list_journal_entries(&pool, Some("2026-03-24"), None, None).await.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].content, "New");
    }

    #[tokio::test]
    async fn test_update_journal_entry_content() {
        let pool = create_test_pool().await;
        let id = add_journal_entry(&pool, "Original", None, None, None).await.unwrap();
        
        update_journal_entry(&pool, id, Some("Updated content"), None, None, None).await.unwrap();
        
        let entry = get_journal_entry(&pool, id).await.unwrap().unwrap();
        assert_eq!(entry.content, "Updated content");
    }

    #[tokio::test]
    async fn test_update_journal_entry_no_fields() {
        let pool = create_test_pool().await;
        let id = add_journal_entry(&pool, "Test", None, None, None).await.unwrap();
        
        let result = update_journal_entry(&pool, id, None, None, None, None).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Nothing to update"));
    }

    #[tokio::test]
    async fn test_delete_journal_entry() {
        let pool = create_test_pool().await;
        let id = add_journal_entry(&pool, "To delete", None, None, None).await.unwrap();
        
        delete_journal_entry(&pool, id).await.unwrap();
        
        let entry = get_journal_entry(&pool, id).await.unwrap();
        assert!(entry.is_none());
    }
}
