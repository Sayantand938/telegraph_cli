use sqlx::SqlitePool;
use serde_json::{Value, json};
use crate::error::AppResult;
use crate::db;

/// Trait for domain entities that can be processed by the generic request processor
#[async_trait::async_trait]
pub trait DomainEntity: Sized + serde::Serialize + serde::de::DeserializeOwned {
    const DOMAIN_NAME: &'static str;
    const CREATE_SUCCESS_MSG: fn(i64) -> String;
    const GET_SUCCESS_MSG: fn(i64) -> String;
    const UPDATE_SUCCESS_MSG: fn(i64) -> String;
    const DELETE_SUCCESS_MSG: fn(i64) -> String;
    const LIST_SUCCESS_MSG: fn(usize) -> String;

    type CreateArgs: serde::de::DeserializeOwned;
    type ListArgs: serde::de::DeserializeOwned + Default;
    type UpdateArgs: serde::de::DeserializeOwned;

    async fn create(pool: &SqlitePool, args: Self::CreateArgs) -> AppResult<(i64, Vec<i64>, Vec<i64>)>;
    async fn get(pool: &SqlitePool, id: i64) -> AppResult<Option<Self>>;
    async fn list(pool: &SqlitePool, args: Self::ListArgs) -> AppResult<Vec<Self>>;
    async fn update(pool: &SqlitePool, args: Self::UpdateArgs) -> AppResult<()>;
    async fn delete(pool: &SqlitePool, id: i64) -> AppResult<()>;
}

/// Generic request processor for all domains
pub async fn process_domain_request<E: DomainEntity>(
    pool: &SqlitePool,
    tool: &str,
    args: &Value,
) -> AppResult<(Option<Value>, String)> {
    match tool {
        t if t == format!("create_{}", E::DOMAIN_NAME) => {
            let create_args = parse_create_args::<E>(args)?;
            let (id, tag_ids, person_ids) = E::create(pool, create_args).await?;
            
            // Apply tags if provided
            if !tag_ids.is_empty() {
                apply_tags::<E>(pool, id, &tag_ids).await?;
            }
            
            // Apply persons if provided
            if !person_ids.is_empty() {
                apply_persons::<E>(pool, id, &person_ids).await?;
            }
            
            Ok((Some(json!({"id": id})), E::CREATE_SUCCESS_MSG(id)))
        }
        t if t == format!("get_{}", E::DOMAIN_NAME) => {
            let id = extract_id(args)?;
            match E::get(pool, id).await? {
                Some(entity) => Ok((Some(serde_json::to_value(&entity)?), E::GET_SUCCESS_MSG(id))),
                None => Err(crate::error::AppError::ValidationError(
                    format!("{} #{} not found", E::DOMAIN_NAME, id)
                )),
            }
        }
        t if t == format!("list_{}", E::DOMAIN_NAME) || t == format!("list_{}s", E::DOMAIN_NAME) => {
            let list_args = parse_list_args::<E>(args)?;
            let entities = E::list(pool, list_args).await?;
            let count = entities.len();
            Ok((Some(serde_json::to_value(&entities)?), E::LIST_SUCCESS_MSG(count)))
        }
        t if t == format!("update_{}", E::DOMAIN_NAME) => {
            let update_args = parse_update_args::<E>(args)?;
            E::update(pool, update_args).await?;
            let id = get_update_id::<E>(args)?;
            Ok((None, E::UPDATE_SUCCESS_MSG(id)))
        }
        t if t == format!("delete_{}", E::DOMAIN_NAME) => {
            let id = extract_id(args)?;
            E::delete(pool, id).await?;
            Ok((None, E::DELETE_SUCCESS_MSG(id)))
        }
        _ => Err(crate::error::AppError::ValidationError(
            format!("Unknown {} tool: {}", E::DOMAIN_NAME, tool)
        )),
    }
}

/// Helper to parse create args from JSON
fn parse_create_args<E: DomainEntity>(args: &Value) -> AppResult<E::CreateArgs> {
    serde_json::from_value(args.clone())
        .map_err(|e| crate::error::AppError::ValidationError(
            format!("Failed to parse create args for {}: {}", E::DOMAIN_NAME, e)
        ))
}

/// Helper to parse list args from JSON
fn parse_list_args<E: DomainEntity>(args: &Value) -> AppResult<E::ListArgs> {
    serde_json::from_value::<E::ListArgs>(args.clone())
        .or_else(|_| Ok(E::ListArgs::default()))
}

/// Helper to parse update args from JSON  
fn parse_update_args<E: DomainEntity>(args: &Value) -> AppResult<E::UpdateArgs> {
    serde_json::from_value(args.clone())
        .map_err(|e| crate::error::AppError::ValidationError(
            format!("Failed to parse update args for {}: {}", E::DOMAIN_NAME, e)
        ))
}

/// Helper to get ID from args
fn extract_id(args: &Value) -> AppResult<i64> {
    args["id"].as_i64()
        .ok_or_else(|| crate::error::AppError::ValidationError("Missing id".to_string()))
}

/// Helper to get ID from update args
fn get_update_id<E: DomainEntity>(args: &Value) -> AppResult<i64> {
    extract_id(args)
}

/// Apply tags to entity based on domain type
async fn apply_tags<E: DomainEntity>(pool: &SqlitePool, id: i64, tag_ids: &[i64]) -> AppResult<()> {
    match E::DOMAIN_NAME {
        "transaction" => db::set_transaction_tags(pool, id, tag_ids).await,
        "activity" => db::set_activity_tags(pool, id, tag_ids).await,
        "todo" => db::set_todo_tags(pool, id, tag_ids).await,
        "journal" => db::set_journal_tags(pool, id, tag_ids).await,
        _ => Ok(()),
    }
}

/// Apply persons to entity based on domain type
async fn apply_persons<E: DomainEntity>(pool: &SqlitePool, id: i64, person_ids: &[i64]) -> AppResult<()> {
    match E::DOMAIN_NAME {
        "transaction" => db::set_transaction_persons(pool, id, person_ids).await,
        "activity" => db::set_activity_persons(pool, id, person_ids).await,
        "todo" => db::set_todo_persons(pool, id, person_ids).await,
        "journal" => db::set_journal_persons(pool, id, person_ids).await,
        _ => Ok(()),
    }
}

/// Helper to extract tag IDs from JSON args
pub fn extract_tag_ids(args: &Value) -> Vec<i64> {
    args["tags"].as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_i64())
                .collect()
        })
        .unwrap_or_default()
}

/// Helper to extract person IDs from JSON args
pub fn extract_person_ids(args: &Value) -> Vec<i64> {
    args["people"].as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_i64())
                .collect()
        })
        .unwrap_or_default()
}

/// Helper to extract tag names from JSON args
pub fn extract_tag_names(args: &Value) -> Vec<String> {
    args["tags"].as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect()
        })
        .unwrap_or_default()
}

/// Helper to extract person names from JSON args
pub fn extract_person_names(args: &Value) -> Vec<String> {
    args["people"].as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect()
        })
        .unwrap_or_default()
}

/// Helper to upsert tags by name and return IDs
pub async fn upsert_tags_by_name(pool: &SqlitePool, names: &[String]) -> AppResult<Vec<i64>> {
    let mut ids = Vec::new();
    for name in names {
        ids.push(db::upsert_tag(pool, name).await?);
    }
    Ok(ids)
}

/// Helper to upsert persons by name and return IDs
pub async fn upsert_persons_by_name(pool: &SqlitePool, names: &[String]) -> AppResult<Vec<i64>> {
    let mut ids = Vec::new();
    for name in names {
        ids.push(db::upsert_person(pool, name).await?);
    }
    Ok(ids)
}

/// Helper to resolve category from name or ID
pub async fn resolve_category(pool: &SqlitePool, name: Option<&String>, id: Option<i64>) -> AppResult<Option<i64>> {
    if let Some(cat_name) = name {
        Ok(Some(db::upsert_category(pool, cat_name).await?))
    } else {
        Ok(id)
    }
}

/// Helper to resolve place from name or ID
pub async fn resolve_place(pool: &SqlitePool, name: Option<&String>, id: Option<i64>) -> AppResult<Option<i64>> {
    if let Some(place_name) = name {
        Ok(Some(db::upsert_place(pool, place_name).await?))
    } else {
        Ok(id)
    }
}
