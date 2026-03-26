use sqlx::SqlitePool;
use serde_json::Value;
use crate::error::AppResult;
use crate::db;

pub async fn process_todo_request(
    pool: &SqlitePool,
    tool: &str,
    args: &Value,
) -> AppResult<(Option<Value>, String)> {
    match tool {
        "create_todo" => {
            let description: String = serde_json::from_value(args["description"].clone())
                .map_err(|_| crate::error::AppError::ValidationError("Missing description".to_string()))?;
            let status: String = serde_json::from_value(args["status"].clone()).unwrap_or_else(|_| "pending".to_string());
            let priority: Option<String> = serde_json::from_value(args["priority"].clone()).ok();
            let due_date: Option<String> = serde_json::from_value(args["due_date"].clone()).ok();
            
            // Handle category by name or id
            let category_id = if args["category"].is_string() {
                let cat_name = args["category"].as_str().unwrap();
                Some(db::upsert_category(pool, cat_name).await?)
            } else if args["category_id"].is_i64() {
                args["category_id"].as_i64()
            } else {
                None
            };
            
            // Handle place by name or id
            let place_id = if args["place"].is_string() {
                let place_name = args["place"].as_str().unwrap();
                Some(db::upsert_place(pool, place_name).await?)
            } else if args["place_id"].is_i64() {
                args["place_id"].as_i64()
            } else {
                None
            };
            
            let todo_id = db::add_todo(
                pool,
                &description,
                &status,
                priority.as_deref(),
                due_date.as_deref(),
                category_id,
                place_id,
            ).await?;
            
            // Handle tags
            if let Some(tags) = args["tags"].as_array() {
                let mut tag_ids = Vec::new();
                for tag_val in tags {
                    if let Some(tag_name) = tag_val.as_str() {
                        let tag_id = db::upsert_tag(pool, tag_name).await?;
                        tag_ids.push(tag_id);
                    }
                }
                db::set_todo_tags(pool, todo_id, &tag_ids).await?;
            }
            
            // Handle persons
            if let Some(persons) = args["persons"].as_array() {
                let mut person_ids = Vec::new();
                for person_val in persons {
                    if let Some(person_name) = person_val.as_str() {
                        let person_id = db::upsert_person(pool, person_name).await?;
                        person_ids.push(person_id);
                    }
                }
                db::set_todo_persons(pool, todo_id, &person_ids).await?;
            }
            
            Ok((Some(serde_json::json!({"id": todo_id})), format!("Todo #{} created", todo_id)))
        }
        
        "get_todo" => {
            let id: i64 = serde_json::from_value(args["id"].clone())
                .map_err(|_| crate::error::AppError::ValidationError("Missing id".to_string()))?;
            
            match db::get_todo(pool, id).await? {
                Some(todo) => Ok((Some(serde_json::to_value(&todo)?), format!("Todo #{} found", id))),
                None => Err(crate::error::AppError::ValidationError(format!("Todo #{} not found", id))),
            }
        }
        
        "list_todos" => {
            let status: Option<String> = serde_json::from_value(args["status"].clone()).ok();
            let priority: Option<String> = serde_json::from_value(args["priority"].clone()).ok();
            let category_id: Option<i64> = serde_json::from_value(args["category_id"].clone()).ok();
            let place_id: Option<i64> = serde_json::from_value(args["place_id"].clone()).ok();
            
            let todos = db::list_todos(
                pool,
                status.as_deref(),
                priority.as_deref(),
                category_id,
                place_id,
            ).await?;
            
            Ok((Some(serde_json::to_value(&todos)?), format!("{} todo(s) found", todos.len())))
        }
        
        "update_todo" => {
            let id: i64 = serde_json::from_value(args["id"].clone())
                .map_err(|_| crate::error::AppError::ValidationError("Missing id".to_string()))?;
            
            let description: Option<String> = serde_json::from_value(args["description"].clone()).ok();
            let status: Option<String> = serde_json::from_value(args["status"].clone()).ok();
            let priority: Option<String> = serde_json::from_value(args["priority"].clone()).ok();
            let due_date: Option<String> = serde_json::from_value(args["due_date"].clone()).ok();
            
            // Handle category by name or id
            let category_id = if args["category"].is_string() {
                let cat_name = args["category"].as_str().unwrap();
                Some(db::upsert_category(pool, cat_name).await?)
            } else if args["category_id"].is_i64() {
                args["category_id"].as_i64()
            } else {
                None
            };
            
            // Handle place by name or id
            let place_id = if args["place"].is_string() {
                let place_name = args["place"].as_str().unwrap();
                Some(db::upsert_place(pool, place_name).await?)
            } else if args["place_id"].is_i64() {
                args["place_id"].as_i64()
            } else {
                None
            };
            
            db::update_todo(
                pool,
                id,
                description.as_deref(),
                status.as_deref(),
                priority.as_deref(),
                due_date.as_deref(),
                category_id,
                place_id,
            ).await?;
            
            Ok((None, format!("Todo #{} updated", id)))
        }
        
        "complete_todo" => {
            let id: i64 = serde_json::from_value(args["id"].clone())
                .map_err(|_| crate::error::AppError::ValidationError("Missing id".to_string()))?;
            
            db::complete_todo(pool, id).await?;
            Ok((None, format!("Todo #{} completed", id)))
        }
        
        "delete_todo" => {
            let id: i64 = serde_json::from_value(args["id"].clone())
                .map_err(|_| crate::error::AppError::ValidationError("Missing id".to_string()))?;
            
            db::delete_todo(pool, id).await?;
            Ok((None, format!("Todo #{} deleted", id)))
        }
        
        _ => Err(crate::error::AppError::ValidationError(format!("Unknown todo tool: {}", tool))),
    }
}
