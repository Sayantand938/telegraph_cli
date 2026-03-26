/// Macro to implement junction table operations for tags
/// 
/// Generates `set_{entity}_tags` and `get_{entity}_tags` functions
/// 
/// # Example
/// ```rust,ignore
/// impl_tag_junction!(transaction, transaction_tags, transaction);
/// ```
#[macro_export]
macro_rules! impl_tag_junction {
    ($entity:ident, $junction_table:ident, $entity_id_col:ident) => {
        paste::paste! {
            pub async fn [<set_ $entity _tags>](
                pool: &sqlx::SqlitePool,
                entity_id: i64,
                tag_ids: &[i64],
            ) -> $crate::error::AppResult<()> {
                sqlx::query(&format!(
                    "DELETE FROM {} WHERE {}_id = ?",
                    stringify!($junction_table),
                    stringify!($entity_id_col)
                ))
                .bind(entity_id)
                .execute(pool)
                .await?;

                for tag_id in tag_ids {
                    sqlx::query(&format!(
                        "INSERT INTO {} ({}_id, tag_id) VALUES (?, ?)",
                        stringify!($junction_table),
                        stringify!($entity_id_col)
                    ))
                    .bind(entity_id)
                    .bind(tag_id)
                    .execute(pool)
                    .await?;
                }
                Ok(())
            }

            pub async fn [<get_ $entity _tags>](
                pool: &sqlx::SqlitePool,
                entity_id: i64,
            ) -> $crate::error::AppResult<Vec<$crate::types::Tag>> {
                let rows = sqlx::query(&format!(
                    "SELECT t.id, t.name FROM tags t
                     INNER JOIN {} jt ON t.id = jt.tag_id
                     WHERE jt.{}_id = ?",
                    stringify!($junction_table),
                    stringify!($entity_id_col)
                ))
                .bind(entity_id)
                .fetch_all(pool)
                .await?;

                let mut tags = Vec::new();
                for row in rows {
                    let id: i64 = row.try_get("id")?;
                    let name: String = row.try_get("name")?;
                    tags.push($crate::types::Tag { id: Some(id), name });
                }
                Ok(tags)
            }
        }
    };
}

/// Macro to implement junction table operations for persons
/// 
/// Generates `set_{entity}_persons` and `get_{entity}_persons` functions
/// 
/// # Example
/// ```rust,ignore
/// impl_person_junction!(transaction, transaction_persons, transaction);
/// ```
#[macro_export]
macro_rules! impl_person_junction {
    ($entity:ident, $junction_table:ident, $entity_id_col:ident) => {
        paste::paste! {
            pub async fn [<set_ $entity _persons>](
                pool: &sqlx::SqlitePool,
                entity_id: i64,
                person_ids: &[i64],
            ) -> $crate::error::AppResult<()> {
                sqlx::query(&format!(
                    "DELETE FROM {} WHERE {}_id = ?",
                    stringify!($junction_table),
                    stringify!($entity_id_col)
                ))
                .bind(entity_id)
                .execute(pool)
                .await?;

                for person_id in person_ids {
                    sqlx::query(&format!(
                        "INSERT INTO {} ({}_id, person_id) VALUES (?, ?)",
                        stringify!($junction_table),
                        stringify!($entity_id_col)
                    ))
                    .bind(entity_id)
                    .bind(person_id)
                    .execute(pool)
                    .await?;
                }
                Ok(())
            }

            pub async fn [<get_ $entity _persons>](
                pool: &sqlx::SqlitePool,
                entity_id: i64,
            ) -> $crate::error::AppResult<Vec<$crate::types::Person>> {
                let rows = sqlx::query(&format!(
                    "SELECT p.id, p.name FROM persons p
                     INNER JOIN {} jp ON p.id = jp.person_id
                     WHERE jp.{}_id = ?",
                    stringify!($junction_table),
                    stringify!($entity_id_col)
                ))
                .bind(entity_id)
                .fetch_all(pool)
                .await?;

                let mut persons = Vec::new();
                for row in rows {
                    let id: i64 = row.try_get("id")?;
                    let name: String = row.try_get("name")?;
                    persons.push($crate::types::Person { id: Some(id), name });
                }
                Ok(persons)
            }
        }
    };
}
