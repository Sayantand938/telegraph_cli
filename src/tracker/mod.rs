mod domain_processor;
mod transactions;
mod activities;
mod todos;
mod journal;
mod search;

use sqlx::SqlitePool;
use std::path::PathBuf;
use crate::error::AppResult;
use crate::db;

pub use transactions::process_transaction_request;
pub use activities::process_activity_request;
pub use todos::process_todo_request;
pub use journal::process_journal_request;
pub use search::{search_domain, SearchRequest};

#[derive(Clone)]
pub struct Tracker {
    pool: SqlitePool,
}

impl Tracker {
    pub async fn new(db_path: Option<PathBuf>) -> AppResult<Self> {
        let pool = db::connect_db(db_path).await?;
        db::init_tables(&pool).await?;
        Ok(Self { pool })
    }

    pub async fn from_pool(pool: SqlitePool) -> AppResult<Self> {
        db::init_tables(&pool).await?;
        Ok(Self { pool })
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}
