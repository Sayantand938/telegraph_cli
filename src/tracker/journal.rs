use sqlx::SqlitePool;
use serde_json::Value;
use crate::error::AppResult;
use crate::db;

pub async fn process_journal_request(
    pool: &SqlitePool,
    tool: &str,
    args: &Value,
) -> AppResult<(Option<Value>, String)> {
    match tool {
        "create_journal" => {
            let content: String = serde_json::from_value(args["content"].clone())
                .map_err(|_| crate::error::AppError::ValidationError("Missing content".to_string()))?;
            let date: Option<String> = serde_json::from_value(args["date"].clone()).ok();
            
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
            
            let entry_id = db::add_journal_entry(
                pool,
                &content,
                date.as_deref(),
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
                db::set_journal_tags(pool, entry_id, &tag_ids).await?;
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
                db::set_journal_persons(pool, entry_id, &person_ids).await?;
            }
            
            Ok((Some(serde_json::json!({"id": entry_id})), format!("Journal entry #{} created", entry_id)))
        }
        
        "get_journal" => {
            let id: i64 = serde_json::from_value(args["id"].clone())
                .map_err(|_| crate::error::AppError::ValidationError("Missing id".to_string()))?;
            
            match db::get_journal_entry(pool, id).await? {
                Some(entry) => Ok((Some(serde_json::to_value(&entry)?), format!("Journal entry #{} found", id))),
                None => Err(crate::error::AppError::ValidationError(format!("Journal entry #{} not found", id))),
            }
        }
        
        "list_journals" => {
            let from_date: Option<String> = serde_json::from_value(args["from"].clone()).ok();
            let to_date: Option<String> = serde_json::from_value(args["to"].clone()).ok();
            let category_id: Option<i64> = serde_json::from_value(args["category_id"].clone()).ok();
            
            let entries = db::list_journal_entries(
                pool,
                from_date.as_deref(),
                to_date.as_deref(),
                category_id,
            ).await?;
            
            Ok((Some(serde_json::to_value(&entries)?), format!("{} journal entry(ies) found", entries.len())))
        }
        
        "search_journals" => {
            let query: String = serde_json::from_value(args["query"].clone())
                .map_err(|_| crate::error::AppError::ValidationError("Missing query".to_string()))?;
            let from_date: Option<String> = serde_json::from_value(args["from"].clone()).ok();
            let to_date: Option<String> = serde_json::from_value(args["to"].clone()).ok();
            
            let entries = db::search_journal_entries(
                pool,
                &query,
                from_date.as_deref(),
                to_date.as_deref(),
            ).await?;
            
            Ok((Some(serde_json::to_value(&entries)?), format!("{} journal entry(ies) found", entries.len())))
        }
        
        "update_journal" => {
            let id: i64 = serde_json::from_value(args["id"].clone())
                .map_err(|_| crate::error::AppError::ValidationError("Missing id".to_string()))?;
            
            let content: Option<String> = serde_json::from_value(args["content"].clone()).ok();
            let date: Option<String> = serde_json::from_value(args["date"].clone()).ok();
            
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
            
            db::update_journal_entry(
                pool,
                id,
                content.as_deref(),
                date.as_deref(),
                category_id,
                place_id,
            ).await?;
            
            Ok((None, format!("Journal entry #{} updated", id)))
        }
        
        "delete_journal" => {
            let id: i64 = serde_json::from_value(args["id"].clone())
                .map_err(|_| crate::error::AppError::ValidationError("Missing id".to_string()))?;
            
            db::delete_journal_entry(pool, id).await?;
            Ok((None, format!("Journal entry #{} deleted", id)))
        }
        
        _ => Err(crate::error::AppError::ValidationError(format!("Unknown journal tool: {}", tool))),
    }
}
