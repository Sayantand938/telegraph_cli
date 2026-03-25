use sqlx::{SqlitePool, Row};
use crate::error::AppResult;

pub async fn add_activity(
    pool: &SqlitePool,
    start: &str,
    stop: &str,
    desc: &str,
) -> AppResult<()> {
    sqlx::query("INSERT INTO activities (start_time, stop_time, description) VALUES (?, ?, ?)")
        .bind(start)
        .bind(stop)
        .bind(desc)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn list_activities(pool: &SqlitePool) -> AppResult<()> {
    let rows = sqlx::query("SELECT id, start_time, stop_time, description FROM activities")
        .fetch_all(pool)
        .await?;

    for row in rows {
        let id: i64 = row.try_get("id")?;
        let start: String = row.try_get("start_time")?;
        let stop: String = row.try_get("stop_time")?;
        let desc: String = row.try_get("description")?;
        println!("[{}] {} -> {} : {}", id, start, stop, desc);
    }
    Ok(())
}

pub async fn update_activity(
    pool: &SqlitePool,
    id: i64,
    start: Option<&str>,
    stop: Option<&str>,
    desc: Option<&str>,
) -> AppResult<()> {
    if start.is_none() && stop.is_none() && desc.is_none() {
        return Err(anyhow::anyhow!(
            "Nothing to update. Provide --start, --stop, or --desc."
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

    Ok(())
}

pub async fn delete_activity(pool: &SqlitePool, id: i64) -> AppResult<()> {
    sqlx::query("DELETE FROM activities WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn handle_activity(
    pool: &SqlitePool,
    _domain: &str,
    tool: &str,
    start: Option<String>,
    stop: Option<String>,
    activity_desc: Option<String>,
    id: Option<i64>,
) -> AppResult<()> {
    match tool {
        "create" => {
            let start = start.ok_or_else(|| anyhow::anyhow!("--start is required"))?;
            let stop = stop.ok_or_else(|| anyhow::anyhow!("--stop is required"))?;
            let desc = activity_desc.unwrap_or_default();
            add_activity(pool, &start, &stop, &desc).await?;
            println!("Activity added!");
        }
        "list" => list_activities(pool).await?,
        "update" => {
            let id = id.ok_or_else(|| anyhow::anyhow!("--id is required for update"))?;
            update_activity(pool, id, start.as_deref(), stop.as_deref(), activity_desc.as_deref())
                .await?;
            println!("Activity updated!");
        }
        "delete" => {
            let id = id.ok_or_else(|| anyhow::anyhow!("--id is required for delete"))?;
            delete_activity(pool, id).await?;
            println!("Activity deleted!");
        }
        _ => return Err(anyhow::anyhow!("Unknown tool/action: {}", tool)),
    }
    Ok(())
}
