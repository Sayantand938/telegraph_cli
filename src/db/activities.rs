use sqlx::{SqlitePool, Row};
use crate::error::AppResult;
use crate::types::Activity;
use super::{get_activity_tags, get_activity_persons};

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

            // Fetch tags and persons
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

    let rows = if category_id.is_some() && place_id.is_some() {
        sqlx::query(&query)
            .bind(category_id.unwrap())
            .bind(place_id.unwrap())
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

    if let Some(start) = start {
        sqlx::query("UPDATE activities SET start_time = ? WHERE id = ?")
            .bind(start)
            .bind(id)
            .execute(pool)
            .await?;
    }

    if let Some(stop) = stop {
        sqlx::query("UPDATE activities SET stop_time = ? WHERE id = ?")
            .bind(stop)
            .bind(id)
            .execute(pool)
            .await?;
    }

    if let Some(desc) = desc {
        sqlx::query("UPDATE activities SET description = ? WHERE id = ?")
            .bind(desc)
            .bind(id)
            .execute(pool)
            .await?;
    }

    if let Some(cat_id) = category_id {
        sqlx::query("UPDATE activities SET category_id = ? WHERE id = ?")
            .bind(cat_id)
            .bind(id)
            .execute(pool)
            .await?;
    }

    if let Some(p_id) = place_id {
        sqlx::query("UPDATE activities SET place_id = ? WHERE id = ?")
            .bind(p_id)
            .bind(id)
            .execute(pool)
            .await?;
    }

    Ok(())
}

pub async fn delete_activity(pool: &SqlitePool, id: i64) -> AppResult<()> {
    sqlx::query("DELETE FROM activities WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}
