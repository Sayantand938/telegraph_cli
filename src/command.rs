//! Command Interface - Unified entry point for CLI, FFI, Flutter, etc.
//!
//! This module provides a simple text-based command interface that can be used
//! by any caller (CLI, FFI, Flutter, web backend, etc.)
//!
//! # Format
//! Commands follow a simple pattern: `<domain> <action> [args...]`
//!
//! # Examples
//!
//! Create transaction:
//! ```ignore
//! "transaction create --amount 50.0 --kind shopping --desc Groceries"
//! ```
//!
//! List todos:
//! ```ignore
//! "todo list --status pending --priority high"
//! ```
//!
//! Search journal:
//! ```ignore
//! "journal search --query meeting"
//! ```

use crate::tracker::Tracker;
use crate::error::{AppError, AppResult};
use serde_json::{json, Value};

/// Parse a command string and execute it
/// 
/// # Arguments
/// * `command` - Command string like "transaction list --kind shopping"
/// * `tracker` - Tracker instance
/// 
/// # Returns
/// * `AppResult<Value>` - JSON response
pub async fn execute_command(command: &str, tracker: &Tracker) -> AppResult<Value> {
    let parts: Vec<&str> = command.split_whitespace().collect();
    
    if parts.is_empty() {
        return Err(AppError::ValidationError("Empty command".to_string()));
    }
    
    let (domain, action, args) = parse_command(&parts)?;
    
    let (tool, request_args) = build_request(domain, action, args)?;
    
    let request = crate::Request {
        tool,
        args: request_args,
    };
    
    let response = tracker.handle(&request).await;
    
    Ok(json!({
        "success": response.success,
        "message": response.message,
        "error": response.error,
        "data": response.data,
    }))
}

/// Parse command parts into domain, action, and arguments
fn parse_command<'a>(parts: &'a [&'a str]) -> AppResult<(&'a str, &'a str, Vec<&'a str>)> {
    if parts.len() < 2 {
        return Err(AppError::ValidationError(
            "Command must have at least domain and action".to_string()
        ));
    }
    
    let domain = parts[0];
    let action = parts[1];
    let args = parts[2..].to_vec();
    
    Ok((domain, action, args))
}

/// Build the tool name and args from parsed command
fn build_request(domain: &str, action: &str, args: Vec<&str>) -> AppResult<(String, Value)> {
    // Special handling for search command
    if domain == "search" {
        return build_search_request(action, args);
    }
    
    // Map domain to tool name (matching api.rs conventions)
    let tool = match (domain, action) {
        ("category" | "categories", "list") => "list_categories".to_string(),
        ("category" | "categories", "delete") => "delete_category".to_string(),
        ("place" | "places", "list") => "list_places".to_string(),
        ("place" | "places", "delete") => "delete_place".to_string(),
        ("tag" | "tags", "list") => "list_tags".to_string(),
        ("tag" | "tags", "delete") => "delete_tag".to_string(),
        ("person" | "persons", "list") => "list_persons".to_string(),
        ("person" | "persons", "delete") => "delete_person".to_string(),
        ("transaction" | "transactions", "create") => "create_transaction".to_string(),
        ("transaction" | "transactions", "get") => "get_transaction".to_string(),
        ("transaction" | "transactions", "list") => "list_transactions".to_string(),
        ("transaction" | "transactions", "update") => "update_transaction".to_string(),
        ("transaction" | "transactions", "delete") => "delete_transaction".to_string(),
        ("activity" | "activities", "create") => "create_activity".to_string(),
        ("activity" | "activities", "get") => "get_activity".to_string(),
        ("activity" | "activities", "list") => "list_activities".to_string(),
        ("activity" | "activities", "update") => "update_activity".to_string(),
        ("activity" | "activities", "delete") => "delete_activity".to_string(),
        ("todo" | "todos", "create") => "create_todo".to_string(),
        ("todo" | "todos", "get") => "get_todo".to_string(),
        ("todo" | "todos", "list") => "list_todos".to_string(),
        ("todo" | "todos", "update") => "update_todo".to_string(),
        ("todo" | "todos", "delete") => "delete_todo".to_string(),
        ("todo" | "todos", "complete") => "complete_todo".to_string(),
        ("journal" | "journals", "create") => "create_journal".to_string(),
        ("journal" | "journals", "get") => "get_journal".to_string(),
        ("journal" | "journals", "list") => "list_journals".to_string(),
        ("journal" | "journals", "search") => "search_journals".to_string(),
        ("journal" | "journals", "update") => "update_journal".to_string(),
        ("journal" | "journals", "delete") => "delete_journal".to_string(),
        // Search command - unified search across domains
        ("search", _) => format!("search_{}", action),
        _ => format!("{}_{}", action, domain),
    };
    
    let mut args_map = serde_json::Map::new();
    
    // Parse key-value arguments
    let mut i = 0;
    while i < args.len() {
        if args[i].starts_with("--") {
            let key = args[i].trim_start_matches("--");
            let value = if i + 1 < args.len() && !args[i + 1].starts_with("--") {
                i += 1;
                args[i]
            } else {
                "true"
            };
            
            // Try to parse as number, otherwise use string
            let json_value: Value = if let Ok(num) = value.parse::<f64>() {
                json!(num)
            } else if let Ok(num) = value.parse::<i64>() {
                json!(num)
            } else {
                json!(value)
            };
            
            args_map.insert(key.to_string(), json_value);
        }
        i += 1;
    }
    
    Ok((tool, Value::Object(args_map)))
}

/// Execute command with database path (convenience function for FFI)
/// 
/// Creates a new Tracker, executes command, returns JSON string
pub async fn execute_command_with_db(command: &str, db_path: Option<&str>) -> String {
    let db_path_opt = db_path.map(|p| std::path::PathBuf::from(p));
    
    match Tracker::new(db_path_opt).await {
        Ok(tracker) => {
            match execute_command(command, &tracker).await {
                Ok(result) => serde_json::to_string(&result).unwrap_or_else(|e| {
                    json!({
                        "success": false,
                        "error": format!("JSON serialization error: {}", e)
                    }).to_string()
                }),
                Err(e) => json!({
                    "success": false,
                    "error": e.to_string()
                }).to_string(),
            }
        }
        Err(e) => json!({
            "success": false,
            "error": format!("Failed to initialize: {}", e)
        }).to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tracker::Tracker;
    
    #[tokio::test]
    async fn test_execute_command_list_categories() {
        let tracker = Tracker::new(None).await.unwrap();
        let result = execute_command("category list", &tracker).await.unwrap();
        
        assert_eq!(result["success"], true);
        assert!(result["data"].is_array());
    }
    
    #[tokio::test]
    async fn test_execute_command_create_transaction() {
        let tracker = Tracker::new(None).await.unwrap();
        let result = execute_command(
            "transaction create --amount 50.0 --kind shopping --description Test",
            &tracker
        ).await.unwrap();

        assert_eq!(result["success"], true);
        assert!(result["data"]["id"].is_number());
    }

    #[tokio::test]
    async fn test_execute_command_list_transactions() {
        let tracker = Tracker::new(None).await.unwrap();
        let result = execute_command("transaction list", &tracker).await.unwrap();

        println!("Result: {:?}", result);
        assert_eq!(result["success"], true);
        assert!(result["data"].is_array());
    }
    
    #[tokio::test]
    async fn test_execute_command_invalid() {
        let tracker = Tracker::new(None).await.unwrap();
        let result = execute_command("invalid command", &tracker).await.unwrap();
        
        assert_eq!(result["success"], false);
        assert!(result["error"].is_string());
    }
    
    #[tokio::test]
    async fn test_execute_command_with_db() {
        let result = execute_command_with_db("category list", None).await;
        let parsed: Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["success"], true);
    }
}

/// Build search request from command arguments
fn build_search_request(domain: &str, args: Vec<&str>) -> AppResult<(String, Value)> {
    let mut args_map = serde_json::Map::new();
    
    // Parse arguments
    let mut i = 0;
    while i < args.len() {
        if !args[i].starts_with("--") {
            i += 1;
            continue;
        }
        
        let key = args[i].trim_start_matches("--");
        let value = if i + 1 < args.len() && !args[i + 1].starts_with("--") {
            i += 1;
            args[i]
        } else {
            "true"
        };
        
        match key {
            "where" => {
                // Parse WHERE clause and add individual filters
                let filters = parse_where_to_json(value)?;
                args_map.insert("filters".to_string(), filters);
            }
            "order-by" | "order_by" => {
                args_map.insert("order_by".to_string(), json!(value));
            }
            "order" => {
                args_map.insert("order".to_string(), json!(value));
            }
            "limit" => {
                args_map.insert("limit".to_string(), json!(value.parse::<i64>().unwrap_or(100)));
            }
            "offset" => {
                args_map.insert("offset".to_string(), json!(value.parse::<i64>().unwrap_or(0)));
            }
            // Dynamic filter - treat as column = value
            _ => {
                let column = key.replace("-", "_");
                args_map.insert(column, json!(value));
            }
        }
        i += 1;
    }
    
    let tool = format!("search_{}", domain);
    Ok((tool, Value::Object(args_map)))
}

/// Parse WHERE clause to JSON array
fn parse_where_to_json(clause: &str) -> AppResult<Value> {
    let mut filters = serde_json::Map::new();
    
    // Split by AND
    for condition in clause.split(" AND ") {
        let condition = condition.trim();
        
        // Parse operator
        if let Some(idx) = condition.find("!=") {
            let col = condition[..idx].trim();
            let val = condition[idx+2..].trim().trim_matches('\'');
            filters.insert(col.to_string(), json!(val));
        } else if let Some(idx) = condition.find(">=") {
            let col = condition[..idx].trim();
            let val = condition[idx+2..].trim().trim_matches('\'');
            filters.insert(format!("{}_gte", col), json!(val));
        } else if let Some(idx) = condition.find("<=") {
            let col = condition[..idx].trim();
            let val = condition[idx+2..].trim().trim_matches('\'');
            filters.insert(format!("{}_lte", col), json!(val));
        } else if let Some(idx) = condition.find(">") {
            let col = condition[..idx].trim();
            let val = condition[idx+1..].trim().trim_matches('\'');
            filters.insert(format!("{}_gt", col), json!(val));
        } else if let Some(idx) = condition.find("<") {
            let col = condition[..idx].trim();
            let val = condition[idx+1..].trim().trim_matches('\'');
            filters.insert(format!("{}_lt", col), json!(val));
        } else if let Some(idx) = condition.find("=") {
            let col = condition[..idx].trim();
            let val = condition[idx+1..].trim().trim_matches('\'');
            filters.insert(col.to_string(), json!(val));
        }
    }
    
    Ok(Value::Object(filters))
}
