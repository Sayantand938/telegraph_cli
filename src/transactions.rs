use sqlx::{SqlitePool, Row};
use chrono::Utc;
use crate::error::AppResult;

pub async fn add_transaction(
    pool: &SqlitePool,
    amount: f64,
    kind: &str,
    description: &str,
) -> AppResult<()> {
    sqlx::query("INSERT INTO transactions (amount, kind, description, timestamp) VALUES (?, ?, ?, ?)")
        .bind(amount)
        .bind(kind)
        .bind(description)
        .bind(Utc::now().to_rfc3339())
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn list_transactions(pool: &SqlitePool, kind_filter: Option<&str>) -> AppResult<()> {
    let rows = if let Some(kind) = kind_filter {
        sqlx::query("SELECT id, amount, kind, description, timestamp FROM transactions WHERE kind = ?")
            .bind(kind)
            .fetch_all(pool)
            .await?
    } else {
        sqlx::query("SELECT id, amount, kind, description, timestamp FROM transactions")
            .fetch_all(pool)
            .await?
    };

    for row in rows {
        let id: i64 = row.try_get("id")?;
        let amount: f64 = row.try_get("amount")?;
        let kind: String = row.try_get("kind")?;
        let desc: String = row.try_get("description")?;
        let ts: String = row.try_get("timestamp")?;
        println!("[{}] {} {} - {} ({})", id, ts, kind, desc, amount);
    }
    Ok(())
}

pub async fn update_transaction(
    pool: &SqlitePool,
    id: i64,
    amount: Option<f64>,
    kind: Option<&str>,
    desc: Option<&str>,
) -> AppResult<()> {
    if amount.is_none() && kind.is_none() && desc.is_none() {
        return Err(anyhow::anyhow!(
            "Nothing to update. Provide --amount, --kind, or --desc."
        ));
    }

    if let Some(amount) = amount {
        sqlx::query("UPDATE transactions SET amount = ? WHERE id = ?")
            .bind(amount)
            .bind(id)
            .execute(pool)
            .await?;
    }

    if let Some(kind) = kind {
        sqlx::query("UPDATE transactions SET kind = ? WHERE id = ?")
            .bind(kind)
            .bind(id)
            .execute(pool)
            .await?;
    }

    if let Some(desc) = desc {
        sqlx::query("UPDATE transactions SET description = ? WHERE id = ?")
            .bind(desc)
            .bind(id)
            .execute(pool)
            .await?;
    }

    Ok(())
}

pub async fn delete_transaction(pool: &SqlitePool, id: i64) -> AppResult<()> {
    sqlx::query("DELETE FROM transactions WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn handle_transaction(
    pool: &SqlitePool,
    _domain: &str,
    tool: &str,
    amount: Option<f64>,
    kind: Option<String>,
    desc: Option<String>,
    id: Option<i64>,
) -> AppResult<()> {
    match tool {
        "create" => {
            let amount = amount.ok_or_else(|| anyhow::anyhow!("--amount is required"))?;
            let kind = kind.ok_or_else(|| anyhow::anyhow!("--kind is required"))?;
            let desc = desc.unwrap_or_default();
            add_transaction(pool, amount, &kind, &desc).await?;
            println!("Transaction added!");
        }
        "list" => list_transactions(pool, kind.as_deref()).await?,
        "update" => {
            let id = id.ok_or_else(|| anyhow::anyhow!("--id is required for update"))?;
            update_transaction(pool, id, amount, kind.as_deref(), desc.as_deref()).await?;
            println!("Transaction updated!");
        }
        "delete" => {
            let id = id.ok_or_else(|| anyhow::anyhow!("--id is required for delete"))?;
            delete_transaction(pool, id).await?;
            println!("Transaction deleted!");
        }
        _ => return Err(anyhow::anyhow!("Unknown tool/action: {}", tool)),
    }
    Ok(())
}
