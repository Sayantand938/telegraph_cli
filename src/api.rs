use serde::{Deserialize, Serialize};
use crate::error::AppResult;
use crate::db;
use crate::tracker::{process_transaction_request, process_activity_request, Tracker};

// ============== Request/Response Types ==============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub tool: String,
    pub args: serde_json::Value,
}

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

// ============== Tracker Implementation ==============

impl Tracker {
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
                let cats = db::list_categories(self.pool()).await?;
                Ok((Some(serde_json::to_value(&cats)?), format!("{} category(ies) found", cats.len())))
            }
            "delete_category" => {
                let args: GetIdArgs = serde_json::from_value(request.args.clone())?;
                db::delete_category(self.pool(), args.id).await?;
                Ok((None, format!("Category #{} deleted", args.id)))
            }

            // Place operations
            "list_places" => {
                let places = db::list_places(self.pool()).await?;
                Ok((Some(serde_json::to_value(&places)?), format!("{} place(s) found", places.len())))
            }
            "delete_place" => {
                let args: GetIdArgs = serde_json::from_value(request.args.clone())?;
                db::delete_place(self.pool(), args.id).await?;
                Ok((None, format!("Place #{} deleted", args.id)))
            }

            // Tag operations
            "list_tags" => {
                let tags = db::list_tags(self.pool()).await?;
                Ok((Some(serde_json::to_value(&tags)?), format!("{} tag(s) found", tags.len())))
            }
            "delete_tag" => {
                let args: GetIdArgs = serde_json::from_value(request.args.clone())?;
                db::delete_tag(self.pool(), args.id).await?;
                Ok((None, format!("Tag #{} deleted", args.id)))
            }

            // Person operations
            "list_persons" => {
                let persons = db::list_persons(self.pool()).await?;
                Ok((Some(serde_json::to_value(&persons)?), format!("{} person(s) found", persons.len())))
            }
            "delete_person" => {
                let args: GetIdArgs = serde_json::from_value(request.args.clone())?;
                db::delete_person(self.pool(), args.id).await?;
                Ok((None, format!("Person #{} deleted", args.id)))
            }

            // Transaction operations
            "create_transaction"
            | "get_transaction"
            | "list_transactions"
            | "update_transaction"
            | "delete_transaction" => {
                process_transaction_request(self.pool(), &request.tool, &request.args).await
            }

            // Activity operations
            "create_activity"
            | "get_activity"
            | "list_activities"
            | "update_activity"
            | "delete_activity" => {
                process_activity_request(self.pool(), &request.tool, &request.args).await
            }

            _ => Err(crate::error::AppError::ValidationError(format!("Unknown tool: {}", request.tool))),
        }
    }
}

// ============== JSON Helper Functions ==============

pub fn parse_request(json: &str) -> AppResult<Request> {
    serde_json::from_str(json).map_err(|e| crate::error::AppError::JsonError(e.to_string()))
}

pub fn to_json<T: Serialize>(value: &T) -> AppResult<String> {
    serde_json::to_string(value).map_err(|e| crate::error::AppError::JsonError(e.to_string()))
}

pub fn to_json_pretty<T: Serialize>(value: &T) -> AppResult<String> {
    serde_json::to_string_pretty(value).map_err(|e| crate::error::AppError::JsonError(e.to_string()))
}

/// Convenience function: JSON in, JSON out
pub async fn handle_json(json_request: &str, db_path: Option<std::path::PathBuf>) -> String {
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
