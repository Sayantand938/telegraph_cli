use sqlx::SqlitePool;
use serde_json::{Value, json};
use crate::error::AppResult;
use crate::db;
use crate::types::Transaction;
use crate::api::{GetIdArgs, ListTransactionsArgs, UpdateTransactionArgs};

pub async fn process_transaction_request(
    pool: &SqlitePool,
    tool: &str,
    args: &Value,
) -> AppResult<(Option<Value>, String)> {
    match tool {
        "create_transaction" => {
            let tx: Transaction = serde_json::from_value(args.clone())?;
            let category_id = if let Some(ref cat_name) = tx.category_name {
                Some(db::upsert_category(pool, cat_name).await?)
            } else {
                None
            };
            let place_id = if let Some(ref place_name) = tx.place_name {
                Some(db::upsert_place(pool, place_name).await?)
            } else {
                None
            };
            let id = db::add_transaction(pool, tx.amount, &tx.kind, &tx.description, category_id, place_id).await?;

            // Handle tags (many-to-many)
            if !tx.tag_names.is_empty() {
                let mut tag_ids = Vec::new();
                for tag_name in &tx.tag_names {
                    tag_ids.push(db::upsert_tag(pool, tag_name).await?);
                }
                db::set_transaction_tags(pool, id, &tag_ids).await?;
            }

            // Handle persons (many-to-many)
            if !tx.person_names.is_empty() {
                let mut person_ids = Vec::new();
                for person_name in &tx.person_names {
                    person_ids.push(db::upsert_person(pool, person_name).await?);
                }
                db::set_transaction_persons(pool, id, &person_ids).await?;
            }

            Ok((Some(json!({ "id": id })), format!("Transaction #{} created", id)))
        }
        "get_transaction" => {
            let args: GetIdArgs = serde_json::from_value(args.clone())?;
            let tx = db::get_transaction(pool, args.id).await?;
            match tx {
                Some(_) => Ok((serde_json::to_value(tx).ok(), "Transaction found".to_string())),
                None => Ok((None, "Transaction not found".to_string())),
            }
        }
        "list_transactions" => {
            let args: ListTransactionsArgs = serde_json::from_value(args.clone()).unwrap_or_default();
            let txs = db::list_transactions(pool, args.kind.as_deref(), args.category_id, args.place_id).await?;
            let count = txs.len();
            Ok((Some(serde_json::to_value(txs)?), format!("{} transaction(s) found", count)))
        }
        "update_transaction" => {
            let args: UpdateTransactionArgs = serde_json::from_value(args.clone())?;
            let category_id = if let Some(ref cat_name) = args.category_name {
                Some(db::upsert_category(pool, cat_name).await?)
            } else {
                args.category_id
            };
            let place_id = if let Some(ref place_name) = args.place_name {
                Some(db::upsert_place(pool, place_name).await?)
            } else {
                args.place_id
            };
            db::update_transaction(pool, args.id, args.amount, args.kind.as_deref(), args.description.as_deref(), category_id, place_id).await?;
            Ok((None, format!("Transaction #{} updated", args.id)))
        }
        "delete_transaction" => {
            let args: GetIdArgs = serde_json::from_value(args.clone())?;
            db::delete_transaction(pool, args.id).await?;
            Ok((None, format!("Transaction #{} deleted", args.id)))
        }
        _ => Err(crate::error::AppError::ValidationError(format!("Unknown transaction tool: {}", tool))),
    }
}
