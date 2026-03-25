use sqlx::SqlitePool;
use serde_json::{Value, json};
use crate::error::AppResult;
use crate::db;
use crate::types::Activity;
use crate::api::{GetIdArgs, ListActivitiesArgs, UpdateActivityArgs};

pub async fn process_activity_request(
    pool: &SqlitePool,
    tool: &str,
    args: &Value,
) -> AppResult<(Option<Value>, String)> {
    match tool {
        "create_activity" => {
            let activity: Activity = serde_json::from_value(args.clone())?;
            let category_id = if let Some(ref cat_name) = activity.category_name {
                Some(db::upsert_category(pool, cat_name).await?)
            } else {
                None
            };
            let place_id = if let Some(ref place_name) = activity.place_name {
                Some(db::upsert_place(pool, place_name).await?)
            } else {
                None
            };
            let id = db::add_activity(pool, &activity.start_time, &activity.stop_time, &activity.description, category_id, place_id).await?;

            // Handle tags (many-to-many)
            if !activity.tag_names.is_empty() {
                let mut tag_ids = Vec::new();
                for tag_name in &activity.tag_names {
                    tag_ids.push(db::upsert_tag(pool, tag_name).await?);
                }
                db::set_activity_tags(pool, id, &tag_ids).await?;
            }

            // Handle persons (many-to-many)
            if !activity.person_names.is_empty() {
                let mut person_ids = Vec::new();
                for person_name in &activity.person_names {
                    person_ids.push(db::upsert_person(pool, person_name).await?);
                }
                db::set_activity_persons(pool, id, &person_ids).await?;
            }

            Ok((Some(json!({ "id": id })), format!("Activity #{} created", id)))
        }
        "get_activity" => {
            let args: GetIdArgs = serde_json::from_value(args.clone())?;
            let activity = db::get_activity(pool, args.id).await?;
            match activity {
                Some(_) => Ok((serde_json::to_value(activity).ok(), "Activity found".to_string())),
                None => Ok((None, "Activity not found".to_string())),
            }
        }
        "list_activities" => {
            let args: ListActivitiesArgs = serde_json::from_value(args.clone()).unwrap_or_default();
            let activities = db::list_activities(pool, args.category_id, args.place_id).await?;
            let count = activities.len();
            Ok((Some(serde_json::to_value(activities)?), format!("{} activity(ies) found", count)))
        }
        "update_activity" => {
            let args: UpdateActivityArgs = serde_json::from_value(args.clone())?;
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
            db::update_activity(pool, args.id, args.start_time.as_deref(), args.stop_time.as_deref(), args.description.as_deref(), category_id, place_id).await?;
            Ok((None, format!("Activity #{} updated", args.id)))
        }
        "delete_activity" => {
            let args: GetIdArgs = serde_json::from_value(args.clone())?;
            db::delete_activity(pool, args.id).await?;
            Ok((None, format!("Activity #{} deleted", args.id)))
        }
        _ => Err(crate::error::AppError::ValidationError(format!("Unknown activity tool: {}", tool))),
    }
}
