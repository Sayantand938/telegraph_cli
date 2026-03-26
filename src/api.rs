use serde::{Deserialize, Serialize};
use crate::error::AppResult;
use crate::db;
use crate::tracker::{process_transaction_request, process_activity_request, process_todo_request, process_journal_request, Tracker};

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
            // Unified search operations
            "search_transaction" | "search_transactions" |
            "search_todo" | "search_todos" |
            "search_activity" | "search_activities" |
            "search_journal" | "search_journals" |
            "search_category" | "search_categories" |
            "search_place" | "search_places" |
            "search_tag" | "search_tags" |
            "search_person" | "search_persons" => {
                // Extract domain from tool name
                let domain = request.tool
                    .trim_start_matches("search_")
                    .trim_start_matches("search_")
                    .to_string();
                
                // Parse search parameters from request args
                let search_request = crate::tracker::SearchRequest {
                    domain,
                    filters: vec![], // Will be populated from args
                    order_by: request.args["order_by"].as_str().map(String::from),
                    order_direction: request.args["order"].as_str().unwrap_or("ASC").to_string(),
                    limit: request.args["limit"].as_i64().unwrap_or(100),
                    offset: request.args["offset"].as_i64(),
                };
                
                let (data, message) = crate::tracker::search_domain(self.pool(), &search_request).await?;
                Ok((Some(data), message))
            }
            
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

            // Todo operations
            "create_todo"
            | "get_todo"
            | "list_todos"
            | "update_todo"
            | "delete_todo"
            | "complete_todo" => {
                process_todo_request(self.pool(), &request.tool, &request.args).await
            }

            // Journal operations
            "create_journal"
            | "get_journal"
            | "list_journals"
            | "update_journal"
            | "delete_journal" => {
                process_journal_request(self.pool(), &request.tool, &request.args).await
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use crate::error::AppError;

    // ============== Request Tests ==============

    #[test]
    fn test_request_serialization() {
        let request = Request {
            tool: "create_transaction".to_string(),
            args: json!({"amount": 50.0, "kind": "shopping"}),
        };
        let serialized = serde_json::to_string(&request).unwrap();
        assert!(serialized.contains("create_transaction"));
        assert!(serialized.contains("amount"));

        let deserialized: Request = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.tool, "create_transaction");
    }

    #[test]
    fn test_request_deserialization() {
        let json_str = r#"{"tool": "list_transactions", "args": {"kind": "shopping"}}"#;
        let request: Request = serde_json::from_str(json_str).unwrap();
        assert_eq!(request.tool, "list_transactions");
        assert_eq!(request.args["kind"], "shopping");
    }

    // ============== Response Tests ==============

    #[test]
    fn test_response_success() {
        let response = Response::success(Some(json!({"id": 1})), "Created successfully");
        assert!(response.success);
        assert_eq!(response.data, Some(json!({"id": 1})));
        assert_eq!(response.message, Some("Created successfully".to_string()));
        assert!(response.error.is_none());
    }

    #[test]
    fn test_response_success_no_data() {
        let response = Response::success(None, "Deleted successfully");
        assert!(response.success);
        assert!(response.data.is_none());
        assert_eq!(response.message, Some("Deleted successfully".to_string()));
    }

    #[test]
    fn test_response_error() {
        let response = Response::error("Something went wrong");
        assert!(!response.success);
        assert!(response.data.is_none());
        assert_eq!(response.error, Some("Something went wrong".to_string()));
        assert!(response.message.is_none());
    }

    #[test]
    fn test_response_serialization() {
        let response = Response::success(Some(json!({"id": 1})), "OK");
        let serialized = serde_json::to_string(&response).unwrap();
        assert!(serialized.contains("\"success\":true"));
        assert!(serialized.contains("\"id\":1"));

        let deserialized: Response = serde_json::from_str(&serialized).unwrap();
        assert!(deserialized.success);
    }

    // ============== GetIdArgs Tests ==============

    #[test]
    fn test_get_id_args_serialization() {
        let args = GetIdArgs { id: 42 };
        let serialized = serde_json::to_string(&args).unwrap();
        assert_eq!(serialized, r#"{"id":42}"#);

        let deserialized: GetIdArgs = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.id, 42);
    }

    // ============== ListTransactionsArgs Tests ==============

    #[test]
    fn test_list_transactions_args_default() {
        let args = ListTransactionsArgs::default();
        assert!(args.kind.is_none());
        assert!(args.category_id.is_none());
        assert!(args.place_id.is_none());
    }

    #[test]
    fn test_list_transactions_args_serialization() {
        let args = ListTransactionsArgs {
            kind: Some("shopping".to_string()),
            category_id: Some(1),
            place_id: None,
        };
        let serialized = serde_json::to_string(&args).unwrap();
        assert!(serialized.contains("shopping"));
        assert!(serialized.contains("1"));
    }

    // ============== ListActivitiesArgs Tests ==============

    #[test]
    fn test_list_activities_args_default() {
        let args = ListActivitiesArgs::default();
        assert!(args.category_id.is_none());
        assert!(args.place_id.is_none());
    }

    #[test]
    fn test_list_activities_args_serialization() {
        let args = ListActivitiesArgs {
            category_id: Some(2),
            place_id: Some(3),
        };
        let serialized = serde_json::to_string(&args).unwrap();
        assert!(serialized.contains("2"));
        assert!(serialized.contains("3"));
    }

    // ============== UpdateTransactionArgs Tests ==============

    #[test]
    fn test_update_transaction_args_serialization() {
        let args = UpdateTransactionArgs {
            id: 1,
            amount: Some(100.0),
            kind: None,
            description: Some("Updated".to_string()),
            category_id: None,
            place_id: None,
            category_name: None,
            place_name: None,
        };
        let serialized = serde_json::to_string(&args).unwrap();
        assert!(serialized.contains("\"id\":1"));
        assert!(serialized.contains("100.0"));
        assert!(serialized.contains("Updated"));
    }

    #[test]
    fn test_update_transaction_args_with_renames() {
        let args = UpdateTransactionArgs {
            id: 1,
            amount: None,
            kind: None,
            description: None,
            category_id: None,
            place_id: None,
            category_name: Some("Food".to_string()),
            place_name: Some("Store".to_string()),
        };
        let serialized = serde_json::to_string(&args).unwrap();
        // Check that the renamed fields use the renamed keys
        assert!(serialized.contains("\"category\""));
        assert!(serialized.contains("\"place\""));
        assert!(serialized.contains("Food"));
        assert!(serialized.contains("Store"));
    }

    // ============== UpdateActivityArgs Tests ==============

    #[test]
    fn test_update_activity_args_serialization() {
        let args = UpdateActivityArgs {
            id: 1,
            start_time: Some("09:00".to_string()),
            stop_time: Some("10:00".to_string()),
            description: None,
            category_id: None,
            place_id: None,
            category_name: None,
            place_name: None,
        };
        let serialized = serde_json::to_string(&args).unwrap();
        assert!(serialized.contains("09:00"));
        assert!(serialized.contains("10:00"));
    }

    // ============== parse_request Tests ==============

    #[test]
    fn test_parse_request_valid() {
        let json_str = r#"{"tool": "list_categories", "args": {}}"#;
        let result = parse_request(json_str);
        assert!(result.is_ok());
        let request = result.unwrap();
        assert_eq!(request.tool, "list_categories");
    }

    #[test]
    fn test_parse_request_invalid_json() {
        let json_str = r#"{"tool": invalid}"#;
        let result = parse_request(json_str);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::JsonError(_)));
    }

    #[test]
    fn test_parse_request_missing_args() {
        // args field is required, but serde will accept null
        let json_str = r#"{"tool": "test", "args": null}"#;
        let result = parse_request(json_str);
        assert!(result.is_ok());
    }

    // ============== to_json Tests ==============

    #[test]
    fn test_to_json() {
        let value = json!({"key": "value", "num": 42});
        let result = to_json(&value);
        assert!(result.is_ok());
        let json_str = result.unwrap();
        assert!(json_str.contains("key"));
        assert!(json_str.contains("value"));
    }

    #[test]
    fn test_to_json_pretty() {
        let value = json!({"key": "value"});
        let result = to_json_pretty(&value);
        assert!(result.is_ok());
        let json_str = result.unwrap();
        // Pretty print should have newlines/indentation
        assert!(json_str.contains('\n') || json_str.contains("  "));
    }

    // ============== handle_json Tests ==============

    #[tokio::test]
    async fn test_handle_json_list_categories() {
        let request = r#"{"tool": "list_categories", "args": {}}"#;
        let response = handle_json(request, None).await;
        assert!(response.contains("\"success\":true"));
        assert!(response.contains("category"));
    }

    #[tokio::test]
    async fn test_handle_json_list_places() {
        let request = r#"{"tool": "list_places", "args": {}}"#;
        let response = handle_json(request, None).await;
        assert!(response.contains("\"success\":true"));
        assert!(response.contains("place"));
    }

    #[tokio::test]
    async fn test_handle_json_list_tags() {
        let request = r#"{"tool": "list_tags", "args": {}}"#;
        let response = handle_json(request, None).await;
        assert!(response.contains("\"success\":true"));
        assert!(response.contains("tag"));
    }

    #[tokio::test]
    async fn test_handle_json_list_persons() {
        let request = r#"{"tool": "list_persons", "args": {}}"#;
        let response = handle_json(request, None).await;
        assert!(response.contains("\"success\":true"));
        assert!(response.contains("person"));
    }

    #[tokio::test]
    async fn test_handle_json_invalid_tool() {
        let request = r#"{"tool": "unknown_tool", "args": {}}"#;
        let response = handle_json(request, None).await;
        assert!(response.contains("\"success\":false"));
        assert!(response.contains("error"));
    }

    #[tokio::test]
    async fn test_handle_json_invalid_json() {
        let request = r#"{"tool": invalid}"#;
        let response = handle_json(request, None).await;
        assert!(response.contains("\"success\":false"));
        assert!(response.contains("error"));
    }

    #[tokio::test]
    async fn test_handle_json_create_transaction() {
        let request = r#"{"tool": "create_transaction", "args": {"amount": 25.0, "kind": "test", "description": "Unit test transaction"}}"#;
        let response = handle_json(request, None).await;
        assert!(response.contains("\"success\":true"));
        assert!(response.contains("\"id\""));
    }

    #[tokio::test]
    async fn test_handle_json_create_activity() {
        let request = r#"{"tool": "create_activity", "args": {"start_time": "09:00", "stop_time": "10:00", "description": "Unit test activity"}}"#;
        let response = handle_json(request, None).await;
        assert!(response.contains("\"success\":true"));
        assert!(response.contains("\"id\""));
    }
}
