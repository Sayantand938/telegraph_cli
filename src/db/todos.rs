use sqlx::{SqlitePool, Row};
use crate::error::AppResult;
use crate::types::Todo;
use crate::db::{get_todo_tags, get_todo_persons};

pub async fn add_todo(
    pool: &SqlitePool,
    description: &str,
    status: &str,
    priority: Option<&str>,
    due_date: Option<&str>,
    category_id: Option<i64>,
    place_id: Option<i64>,
) -> AppResult<i64> {
    let result = sqlx::query(
        "INSERT INTO todos (description, status, priority, due_date, created_at, category_id, place_id)
         VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(description)
    .bind(status)
    .bind(priority)
    .bind(due_date)
    .bind(chrono::Utc::now().to_rfc3339())
    .bind(category_id)
    .bind(place_id)
    .execute(pool)
    .await?;
    Ok(result.last_insert_rowid())
}

pub async fn get_todo(pool: &SqlitePool, id: i64) -> AppResult<Option<Todo>> {
    let row = sqlx::query(
        "SELECT id, description, status, priority, due_date, created_at, completed_at, category_id, place_id
         FROM todos WHERE id = ?"
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    match row {
        Some(row) => {
            let id: i64 = row.try_get("id")?;
            let description: String = row.try_get("description")?;
            let status: String = row.try_get("status")?;
            let priority: Option<String> = row.try_get("priority").ok().flatten();
            let due_date: Option<String> = row.try_get("due_date").ok().flatten();
            let created_at: String = row.try_get("created_at")?;
            let completed_at: Option<String> = row.try_get("completed_at").ok().flatten();
            let cat_id: Option<i64> = row.try_get("category_id").ok().flatten();
            let place_id: Option<i64> = row.try_get("place_id").ok().flatten();

            // Fetch tags and persons via junction helpers
            let tags = get_todo_tags(pool, id).await?;
            let persons = get_todo_persons(pool, id).await?;

            Ok(Some(Todo {
                id: Some(id),
                description,
                status,
                priority,
                due_date,
                created_at,
                completed_at,
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

pub async fn list_todos(
    pool: &SqlitePool,
    status: Option<&str>,
    priority: Option<&str>,
    category_id: Option<i64>,
    place_id: Option<i64>,
) -> AppResult<Vec<Todo>> {
    let mut query = String::from(
        "SELECT id, description, status, priority, due_date, created_at, completed_at, category_id, place_id FROM todos"
    );
    let mut conditions = Vec::new();

    if status.is_some() {
        conditions.push("status = ?");
    }
    if priority.is_some() {
        conditions.push("priority = ?");
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

    // Build query dynamically based on number of conditions
    let rows = if conditions.is_empty() {
        sqlx::query(&query).fetch_all(pool).await?
    } else {
        let mut db_query = sqlx::query(&query);
        if let Some(s) = status {
            db_query = db_query.bind(s);
        }
        if let Some(p) = priority {
            db_query = db_query.bind(p);
        }
        if let Some(cat) = category_id {
            db_query = db_query.bind(cat);
        }
        if let Some(place) = place_id {
            db_query = db_query.bind(place);
        }
        db_query.fetch_all(pool).await?
    };

    let mut todos = Vec::new();
    for row in rows {
        let id: i64 = row.try_get("id")?;
        let description: String = row.try_get("description")?;
        let status: String = row.try_get("status")?;
        let priority: Option<String> = row.try_get("priority").ok().flatten();
        let due_date: Option<String> = row.try_get("due_date").ok().flatten();
        let created_at: String = row.try_get("created_at")?;
        let completed_at: Option<String> = row.try_get("completed_at").ok().flatten();
        let cat_id: Option<i64> = row.try_get("category_id").ok().flatten();
        let place_id: Option<i64> = row.try_get("place_id").ok().flatten();
        todos.push(Todo {
            id: Some(id),
            description,
            status,
            priority,
            due_date,
            created_at,
            completed_at,
            category_id: cat_id,
            place_id,
            category_name: None,
            place_name: None,
            tag_names: Vec::new(),
            person_names: Vec::new(),
        });
    }
    Ok(todos)
}

pub async fn update_todo(
    pool: &SqlitePool,
    id: i64,
    description: Option<&str>,
    status: Option<&str>,
    priority: Option<&str>,
    due_date: Option<&str>,
    category_id: Option<i64>,
    place_id: Option<i64>,
) -> AppResult<()> {
    if description.is_none() && status.is_none() && priority.is_none()
        && due_date.is_none() && category_id.is_none() && place_id.is_none() {
        return Err(crate::error::AppError::ValidationError(
            "Nothing to update. Provide description, status, priority, due_date, category, or place.".to_string(),
        ));
    }

    let mut updates = Vec::new();
    let mut bind_count = 0;

    if description.is_some() {
        updates.push(format!("description = ?{}", bind_count + 1));
        bind_count += 1;
    }
    if status.is_some() {
        updates.push(format!("status = ?{}", bind_count + 1));
        bind_count += 1;
    }
    if priority.is_some() {
        updates.push(format!("priority = ?{}", bind_count + 1));
        bind_count += 1;
    }
    if due_date.is_some() {
        updates.push(format!("due_date = ?{}", bind_count + 1));
        bind_count += 1;
    }
    if category_id.is_some() {
        updates.push(format!("category_id = ?{}", bind_count + 1));
        bind_count += 1;
    }
    if place_id.is_some() {
        updates.push(format!("place_id = ?{}", bind_count + 1));
        bind_count += 1;
    }

    let mut query = format!("UPDATE todos SET {} WHERE id = ?", updates.join(", "));
    for i in (1..=bind_count).rev() {
        query = query.replace(&format!("?{}", i), "?");
    }

    let mut db_query = sqlx::query(&query);
    if let Some(d) = description {
        db_query = db_query.bind(d);
    }
    if let Some(s) = status {
        db_query = db_query.bind(s);
    }
    if let Some(p) = priority {
        db_query = db_query.bind(p);
    }
    if let Some(d) = due_date {
        db_query = db_query.bind(d);
    }
    if let Some(cat_id) = category_id {
        db_query = db_query.bind(cat_id);
    }
    if let Some(p_id) = place_id {
        db_query = db_query.bind(p_id);
    }
    db_query = db_query.bind(id);

    db_query.execute(pool).await?;
    Ok(())
}

pub async fn complete_todo(pool: &SqlitePool, id: i64) -> AppResult<()> {
    sqlx::query("UPDATE todos SET status = 'completed', completed_at = ? WHERE id = ?")
        .bind(chrono::Utc::now().to_rfc3339())
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete_todo(pool: &SqlitePool, id: i64) -> AppResult<()> {
    sqlx::query("DELETE FROM todos WHERE id = ?")
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
    async fn test_add_todo_basic() {
        let pool = create_test_pool().await;
        let id = add_todo(&pool, "Test task", "pending", None, None, None, None).await.unwrap();
        assert!(id > 0);
    }

    #[tokio::test]
    async fn test_add_todo_with_priority_and_due() {
        let pool = create_test_pool().await;
        let id = add_todo(&pool, "Important task", "pending", Some("high"), Some("2026-04-01"), None, None).await.unwrap();

        let todo = get_todo(&pool, id).await.unwrap().unwrap();
        assert_eq!(todo.priority, Some("high".to_string()));
        assert_eq!(todo.due_date, Some("2026-04-01".to_string()));
    }

    #[tokio::test]
    async fn test_add_todo_with_category() {
        let pool = create_test_pool().await;
        let cat_id = upsert_category(&pool, "Work").await.unwrap();
        let todo_id = add_todo(&pool, "Work task", "pending", None, None, Some(cat_id), None).await.unwrap();

        let todo = get_todo(&pool, todo_id).await.unwrap().unwrap();
        assert_eq!(todo.category_id, Some(cat_id));
    }

    #[tokio::test]
    async fn test_add_todo_with_place() {
        let pool = create_test_pool().await;
        let place_id = upsert_place(&pool, "Office").await.unwrap();
        let todo_id = add_todo(&pool, "Office task", "pending", None, None, None, Some(place_id)).await.unwrap();

        let todo = get_todo(&pool, todo_id).await.unwrap().unwrap();
        assert_eq!(todo.place_id, Some(place_id));
    }

    #[tokio::test]
    async fn test_get_todo_not_found() {
        let pool = create_test_pool().await;
        let result = get_todo(&pool, 999).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_list_todos_all() {
        let pool = create_test_pool().await;
        add_todo(&pool, "First", "pending", None, None, None, None).await.unwrap();
        add_todo(&pool, "Second", "in_progress", None, None, None, None).await.unwrap();
        add_todo(&pool, "Third", "completed", None, None, None, None).await.unwrap();

        let todos = list_todos(&pool, None, None, None, None).await.unwrap();
        assert_eq!(todos.len(), 3);
    }

    #[tokio::test]
    async fn test_list_todos_by_status() {
        let pool = create_test_pool().await;
        add_todo(&pool, "First", "pending", None, None, None, None).await.unwrap();
        add_todo(&pool, "Second", "in_progress", None, None, None, None).await.unwrap();
        add_todo(&pool, "Third", "pending", None, None, None, None).await.unwrap();

        let todos = list_todos(&pool, Some("pending"), None, None, None).await.unwrap();
        assert_eq!(todos.len(), 2);
        for todo in &todos {
            assert_eq!(todo.status, "pending");
        }
    }

    #[tokio::test]
    async fn test_list_todos_by_priority() {
        let pool = create_test_pool().await;
        add_todo(&pool, "First", "pending", Some("high"), None, None, None).await.unwrap();
        add_todo(&pool, "Second", "pending", Some("low"), None, None, None).await.unwrap();
        add_todo(&pool, "Third", "pending", Some("high"), None, None, None).await.unwrap();

        let todos = list_todos(&pool, None, Some("high"), None, None).await.unwrap();
        assert_eq!(todos.len(), 2);
    }

    #[tokio::test]
    async fn test_update_todo_description() {
        let pool = create_test_pool().await;
        let id = add_todo(&pool, "Original", "pending", None, None, None, None).await.unwrap();

        update_todo(&pool, id, Some("Updated description"), None, None, None, None, None).await.unwrap();

        let todo = get_todo(&pool, id).await.unwrap().unwrap();
        assert_eq!(todo.description, "Updated description");
    }

    #[tokio::test]
    async fn test_update_todo_status() {
        let pool = create_test_pool().await;
        let id = add_todo(&pool, "Task", "pending", None, None, None, None).await.unwrap();

        update_todo(&pool, id, None, Some("in_progress"), None, None, None, None).await.unwrap();

        let todo = get_todo(&pool, id).await.unwrap().unwrap();
        assert_eq!(todo.status, "in_progress");
    }

    #[tokio::test]
    async fn test_update_todo_no_fields() {
        let pool = create_test_pool().await;
        let id = add_todo(&pool, "Task", "pending", None, None, None, None).await.unwrap();

        let result = update_todo(&pool, id, None, None, None, None, None, None).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Nothing to update"));
    }

    #[tokio::test]
    async fn test_complete_todo() {
        let pool = create_test_pool().await;
        let id = add_todo(&pool, "Task to complete", "pending", None, None, None, None).await.unwrap();

        complete_todo(&pool, id).await.unwrap();

        let todo = get_todo(&pool, id).await.unwrap().unwrap();
        assert_eq!(todo.status, "completed");
        assert!(todo.completed_at.is_some());
    }

    #[tokio::test]
    async fn test_delete_todo() {
        let pool = create_test_pool().await;
        let id = add_todo(&pool, "To delete", "pending", None, None, None, None).await.unwrap();

        delete_todo(&pool, id).await.unwrap();

        let todo = get_todo(&pool, id).await.unwrap();
        assert!(todo.is_none());
    }
}
