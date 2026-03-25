mod db;
mod error;
mod ffi;

use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::path::PathBuf;

pub use error::{AppError, AppResult};

// ============== Data Types ==============

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Category {
    pub id: Option<i64>,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Place {
    pub id: Option<i64>,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Tag {
    pub id: Option<i64>,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Person {
    pub id: Option<i64>,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Transaction {
    pub id: Option<i64>,
    pub amount: f64,
    pub kind: String,
    pub description: String,
    pub category_id: Option<i64>,
    pub place_id: Option<i64>,
    #[serde(rename = "category", default)]
    pub category_name: Option<String>,
    #[serde(rename = "place", default)]
    pub place_name: Option<String>,
    #[serde(rename = "tags", default)]
    pub tag_names: Vec<String>,
    #[serde(rename = "persons", default)]
    pub person_names: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Activity {
    pub id: Option<i64>,
    pub start_time: String,
    pub stop_time: String,
    pub description: String,
    pub category_id: Option<i64>,
    pub place_id: Option<i64>,
    #[serde(rename = "category", default)]
    pub category_name: Option<String>,
    #[serde(rename = "place", default)]
    pub place_name: Option<String>,
    #[serde(rename = "tags", default)]
    pub tag_names: Vec<String>,
    #[serde(rename = "persons", default)]
    pub person_names: Vec<String>,
}

// ============== Request Types ==============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub tool: String,
    pub args: serde_json::Value,
}

// ============== Response Types ==============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub message: Option<String>,
    pub error: Option<String>,
}

impl Response {
    pub fn success(data: Option<serde_json::Value>, message: impl Into<String>) -> Self {
        Self { success: true, data, error: None, message: Some(message.into()) }
    }

    pub fn error(msg: impl Into<String>) -> Self {
        Self { success: false, data: None, error: Some(msg.into()), message: None }
    }
}

// ============== Single Entry Point ==============

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

    /// Single entry point: accepts request, returns response
    pub async fn handle(&self, request: &Request) -> Response {
        match self.process_request(request).await {
            Ok((data, message)) => Response::success(data, message),
            Err(e) => Response::error(e.to_string()),
        }
    }

    async fn process_request(&self, request: &Request) -> AppResult<(Option<serde_json::Value>, String)> {
        match request.tool.as_str() {
            // Category operations
            "list_categories" => {
                let cats = db::list_categories(&self.pool).await?;
                Ok((Some(serde_json::to_value(&cats)?), format!("{} category(ies) found", cats.len())))
            }
            "delete_category" => {
                let args: GetIdArgs = serde_json::from_value(request.args.clone())?;
                db::delete_category(&self.pool, args.id).await?;
                Ok((None, format!("Category #{} deleted", args.id)))
            }

            // Place operations
            "list_places" => {
                let places = db::list_places(&self.pool).await?;
                Ok((Some(serde_json::to_value(&places)?), format!("{} place(s) found", places.len())))
            }
            "delete_place" => {
                let args: GetIdArgs = serde_json::from_value(request.args.clone())?;
                db::delete_place(&self.pool, args.id).await?;
                Ok((None, format!("Place #{} deleted", args.id)))
            }

            // Tag operations
            "list_tags" => {
                let tags = db::list_tags(&self.pool).await?;
                Ok((Some(serde_json::to_value(&tags)?), format!("{} tag(s) found", tags.len())))
            }
            "delete_tag" => {
                let args: GetIdArgs = serde_json::from_value(request.args.clone())?;
                db::delete_tag(&self.pool, args.id).await?;
                Ok((None, format!("Tag #{} deleted", args.id)))
            }

            // Person operations
            "list_persons" => {
                let persons = db::list_persons(&self.pool).await?;
                Ok((Some(serde_json::to_value(&persons)?), format!("{} person(s) found", persons.len())))
            }
            "delete_person" => {
                let args: GetIdArgs = serde_json::from_value(request.args.clone())?;
                db::delete_person(&self.pool, args.id).await?;
                Ok((None, format!("Person #{} deleted", args.id)))
            }

            // Transaction operations
            "create_transaction" => {
                let tx: Transaction = serde_json::from_value(request.args.clone())?;
                let category_id = if let Some(cat_name) = tx.category_name {
                    Some(db::upsert_category(&self.pool, &cat_name).await?)
                } else {
                    None
                };
                let place_id = if let Some(place_name) = tx.place_name {
                    Some(db::upsert_place(&self.pool, &place_name).await?)
                } else {
                    None
                };
                let id = db::add_transaction(&self.pool, tx.amount, &tx.kind, &tx.description, category_id, place_id).await?;
                
                // Handle tags (many-to-many)
                if !tx.tag_names.is_empty() {
                    let mut tag_ids = Vec::new();
                    for tag_name in &tx.tag_names {
                        tag_ids.push(db::upsert_tag(&self.pool, tag_name).await?);
                    }
                    db::set_transaction_tags(&self.pool, id, &tag_ids).await?;
                }
                
                // Handle persons (many-to-many)
                if !tx.person_names.is_empty() {
                    let mut person_ids = Vec::new();
                    for person_name in &tx.person_names {
                        person_ids.push(db::upsert_person(&self.pool, person_name).await?);
                    }
                    db::set_transaction_persons(&self.pool, id, &person_ids).await?;
                }
                
                Ok((Some(serde_json::json!({ "id": id })), format!("Transaction #{} created", id)))
            }
            "get_transaction" => {
                let args: GetIdArgs = serde_json::from_value(request.args.clone())?;
                let tx = db::get_transaction(&self.pool, args.id).await?;
                match tx {
                    Some(_) => Ok((serde_json::to_value(tx).ok(), "Transaction found".to_string())),
                    None => Ok((None, "Transaction not found".to_string())),
                }
            }
            "list_transactions" => {
                let args: ListTransactionsArgs = serde_json::from_value(request.args.clone()).unwrap_or_default();
                let txs = db::list_transactions(&self.pool, args.kind.as_deref(), args.category_id, args.place_id).await?;
                let count = txs.len();
                Ok((Some(serde_json::to_value(txs)?), format!("{} transaction(s) found", count)))
            }
            "update_transaction" => {
                let args: UpdateTransactionArgs = serde_json::from_value(request.args.clone())?;
                let category_id = if let Some(cat_name) = args.category_name {
                    Some(db::upsert_category(&self.pool, &cat_name).await?)
                } else {
                    args.category_id
                };
                let place_id = if let Some(place_name) = args.place_name {
                    Some(db::upsert_place(&self.pool, &place_name).await?)
                } else {
                    args.place_id
                };
                db::update_transaction(&self.pool, args.id, args.amount, args.kind.as_deref(), args.description.as_deref(), category_id, place_id).await?;
                Ok((None, format!("Transaction #{} updated", args.id)))
            }
            "delete_transaction" => {
                let args: GetIdArgs = serde_json::from_value(request.args.clone())?;
                db::delete_transaction(&self.pool, args.id).await?;
                Ok((None, format!("Transaction #{} deleted", args.id)))
            }

            // Activity operations
            "create_activity" => {
                let activity: Activity = serde_json::from_value(request.args.clone())?;
                let category_id = if let Some(cat_name) = activity.category_name {
                    Some(db::upsert_category(&self.pool, &cat_name).await?)
                } else {
                    None
                };
                let place_id = if let Some(place_name) = activity.place_name {
                    Some(db::upsert_place(&self.pool, &place_name).await?)
                } else {
                    None
                };
                let id = db::add_activity(&self.pool, &activity.start_time, &activity.stop_time, &activity.description, category_id, place_id).await?;
                
                // Handle tags (many-to-many)
                if !activity.tag_names.is_empty() {
                    let mut tag_ids = Vec::new();
                    for tag_name in &activity.tag_names {
                        tag_ids.push(db::upsert_tag(&self.pool, tag_name).await?);
                    }
                    db::set_activity_tags(&self.pool, id, &tag_ids).await?;
                }
                
                // Handle persons (many-to-many)
                if !activity.person_names.is_empty() {
                    let mut person_ids = Vec::new();
                    for person_name in &activity.person_names {
                        person_ids.push(db::upsert_person(&self.pool, person_name).await?);
                    }
                    db::set_activity_persons(&self.pool, id, &person_ids).await?;
                }
                
                Ok((Some(serde_json::json!({ "id": id })), format!("Activity #{} created", id)))
            }
            "get_activity" => {
                let args: GetIdArgs = serde_json::from_value(request.args.clone())?;
                let activity = db::get_activity(&self.pool, args.id).await?;
                match activity {
                    Some(_) => Ok((serde_json::to_value(activity).ok(), "Activity found".to_string())),
                    None => Ok((None, "Activity not found".to_string())),
                }
            }
            "list_activities" => {
                let args: ListActivitiesArgs = serde_json::from_value(request.args.clone()).unwrap_or_default();
                let activities = db::list_activities(&self.pool, args.category_id, args.place_id).await?;
                let count = activities.len();
                Ok((Some(serde_json::to_value(activities)?), format!("{} activity(ies) found", count)))
            }
            "update_activity" => {
                let args: UpdateActivityArgs = serde_json::from_value(request.args.clone())?;
                let category_id = if let Some(cat_name) = args.category_name {
                    Some(db::upsert_category(&self.pool, &cat_name).await?)
                } else {
                    args.category_id
                };
                let place_id = if let Some(place_name) = args.place_name {
                    Some(db::upsert_place(&self.pool, &place_name).await?)
                } else {
                    args.place_id
                };
                db::update_activity(&self.pool, args.id, args.start_time.as_deref(), args.stop_time.as_deref(), args.description.as_deref(), category_id, place_id).await?;
                Ok((None, format!("Activity #{} updated", args.id)))
            }
            "delete_activity" => {
                let args: GetIdArgs = serde_json::from_value(request.args.clone())?;
                db::delete_activity(&self.pool, args.id).await?;
                Ok((None, format!("Activity #{} deleted", args.id)))
            }

            _ => Err(AppError::ValidationError(format!("Unknown tool: {}", request.tool))),
        }
    }
}

// ============== Arg Types ==============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetIdArgs {
    pub id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListTransactionsArgs {
    pub kind: Option<String>,
    pub category_id: Option<i64>,
    pub place_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListActivitiesArgs {
    pub category_id: Option<i64>,
    pub place_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTransactionArgs {
    pub id: i64,
    pub amount: Option<f64>,
    pub kind: Option<String>,
    pub description: Option<String>,
    pub category_id: Option<i64>,
    pub place_id: Option<i64>,
    #[serde(rename = "category")]
    pub category_name: Option<String>,
    #[serde(rename = "place")]
    pub place_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateActivityArgs {
    pub id: i64,
    pub start_time: Option<String>,
    pub stop_time: Option<String>,
    pub description: Option<String>,
    pub category_id: Option<i64>,
    pub place_id: Option<i64>,
    #[serde(rename = "category")]
    pub category_name: Option<String>,
    #[serde(rename = "place")]
    pub place_name: Option<String>,
}

// ============== JSON Helper Functions ==============

pub fn parse_request(json: &str) -> AppResult<Request> {
    serde_json::from_str(json).map_err(|e| AppError::JsonError(e.to_string()))
}

pub fn to_json<T: Serialize>(value: &T) -> AppResult<String> {
    serde_json::to_string(value).map_err(|e| AppError::JsonError(e.to_string()))
}

pub fn to_json_pretty<T: Serialize>(value: &T) -> AppResult<String> {
    serde_json::to_string_pretty(value).map_err(|e| AppError::JsonError(e.to_string()))
}

/// Convenience function: JSON in, JSON out
pub async fn handle_json(json_request: &str, db_path: Option<PathBuf>) -> String {
    match parse_request(json_request) {
        Ok(request) => {
            match Tracker::new(db_path).await {
                Ok(tracker) => {
                    let response = tracker.handle(&request).await;
                    to_json(&response).unwrap_or_else(|e| format!(r#"{{"success":false,"error":"{}"}}"#, e))
                }
                Err(e) => format!(r#"{{"success":false,"error":"{}"}}"#, e),
            }
        }
        Err(e) => format!(r#"{{"success":false,"error":"{}"}}"#, e),
    }
}
