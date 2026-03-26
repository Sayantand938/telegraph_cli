use sqlx::{SqlitePool, Row};
use crate::error::AppResult;
use crate::types::Activity;
use crate::db::{get_activity_tags, get_activity_persons};

pub async fn add_activity(
    pool: &SqlitePool,
    start: &str,
    stop: &str,
    desc: &str,
    category_id: Option<i64>,
    place_id: Option<i64>,
) -> AppResult<i64> {
    let result = sqlx::query(
        "INSERT INTO activities (start_time, stop_time, description, category_id, place_id) VALUES (?, ?, ?, ?, ?)"
    )
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
    let row = sqlx::query(
        "SELECT id, start_time, stop_time, description, category_id, place_id FROM activities WHERE id = ?"
    )
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

            // Fetch tags and persons via junction helpers
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
    let mut query = String::from(
        "SELECT id, start_time, stop_time, description, category_id, place_id FROM activities"
    );
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

    let rows = if conditions.is_empty() {
        sqlx::query(&query).fetch_all(pool).await?
    } else if conditions.len() == 1 {
        sqlx::query(&query)
            .bind(category_id.or(place_id).unwrap())
            .fetch_all(pool)
            .await?
    } else {
        sqlx::query(&query)
            .bind(category_id.unwrap())
            .bind(place_id.unwrap())
            .fetch_all(pool)
            .await?
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

    let mut updates = Vec::new();
    let mut bind_count = 0;

    if start.is_some() {
        updates.push(format!("start_time = ?{}", bind_count + 1));
        bind_count += 1;
    }
    if stop.is_some() {
        updates.push(format!("stop_time = ?{}", bind_count + 1));
        bind_count += 1;
    }
    if desc.is_some() {
        updates.push(format!("description = ?{}", bind_count + 1));
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

    let mut query = format!("UPDATE activities SET {} WHERE id = ?", updates.join(", "));
    for i in (1..=bind_count).rev() {
        query = query.replace(&format!("?{}", i), "?");
    }

    let mut db_query = sqlx::query(&query);
    if let Some(s) = start {
        db_query = db_query.bind(s);
    }
    if let Some(s) = stop {
        db_query = db_query.bind(s);
    }
    if let Some(d) = desc {
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

pub async fn delete_activity(pool: &SqlitePool, id: i64) -> AppResult<()> {
    sqlx::query("DELETE FROM activities WHERE id = ?")
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
    async fn test_add_activity_basic() {
        let pool = create_test_pool().await;
        let id = add_activity(&pool, "09:00", "10:00", "Test activity", None, None).await.unwrap();
        assert!(id > 0);
    }

    #[tokio::test]
    async fn test_add_activity_with_category() {
        let pool = create_test_pool().await;
        let cat_id = upsert_category(&pool, "Work").await.unwrap();
        let activity_id = add_activity(&pool, "09:00", "17:00", "Work day", Some(cat_id), None).await.unwrap();

        let activity = get_activity(&pool, activity_id).await.unwrap().unwrap();
        assert_eq!(activity.category_id, Some(cat_id));
    }

    #[tokio::test]
    async fn test_add_activity_with_place() {
        let pool = create_test_pool().await;
        let place_id = upsert_place(&pool, "Office").await.unwrap();
        let activity_id = add_activity(&pool, "09:00", "17:00", "Work day", None, Some(place_id)).await.unwrap();

        let activity = get_activity(&pool, activity_id).await.unwrap().unwrap();
        assert_eq!(activity.place_id, Some(place_id));
    }

    #[tokio::test]
    async fn test_get_activity_not_found() {
        let pool = create_test_pool().await;
        let result = get_activity(&pool, 999).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_list_activities_all() {
        let pool = create_test_pool().await;
        add_activity(&pool, "09:00", "10:00", "First", None, None).await.unwrap();
        add_activity(&pool, "11:00", "12:00", "Second", None, None).await.unwrap();
        add_activity(&pool, "14:00", "15:00", "Third", None, None).await.unwrap();

        let activities = list_activities(&pool, None, None).await.unwrap();
        assert_eq!(activities.len(), 3);
    }

    #[tokio::test]
    async fn test_update_activity_times() {
        let pool = create_test_pool().await;
        let id = add_activity(&pool, "09:00", "10:00", "Original", None, None).await.unwrap();

        update_activity(&pool, id, Some("10:00"), Some("11:00"), None, None, None).await.unwrap();

        let activity = get_activity(&pool, id).await.unwrap().unwrap();
        assert_eq!(activity.start_time, "10:00");
        assert_eq!(activity.stop_time, "11:00");
    }

    #[tokio::test]
    async fn test_update_activity_no_fields() {
        let pool = create_test_pool().await;
        let id = add_activity(&pool, "09:00", "10:00", "Original", None, None).await.unwrap();

        let result = update_activity(&pool, id, None, None, None, None, None).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Nothing to update"));
    }

    #[tokio::test]
    async fn test_delete_activity() {
        let pool = create_test_pool().await;
        let id = add_activity(&pool, "09:00", "10:00", "To Delete", None, None).await.unwrap();

        delete_activity(&pool, id).await.unwrap();

        let activity = get_activity(&pool, id).await.unwrap();
        assert!(activity.is_none());
    }
}
