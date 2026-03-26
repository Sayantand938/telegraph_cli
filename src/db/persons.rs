use sqlx::{SqlitePool, Row};
use crate::{error::AppResult, types::Person, impl_person_junction};

pub async fn upsert_person(pool: &SqlitePool, name: &str) -> AppResult<i64> {
    sqlx::query("INSERT OR IGNORE INTO persons (name) VALUES (?)")
        .bind(name)
        .execute(pool)
        .await?;

    let row = sqlx::query("SELECT id FROM persons WHERE name = ?")
        .bind(name)
        .fetch_one(pool)
        .await?;

    let id: i64 = row.try_get("id")?;
    Ok(id)
}

pub async fn list_persons(pool: &SqlitePool) -> AppResult<Vec<Person>> {
    let rows = sqlx::query("SELECT id, name FROM persons ORDER BY name")
        .fetch_all(pool)
        .await?;

    let mut persons = Vec::new();
    for row in rows {
        let id: i64 = row.try_get("id")?;
        let name: String = row.try_get("name")?;
        persons.push(Person { id: Some(id), name });
    }
    Ok(persons)
}

pub async fn delete_person(pool: &SqlitePool, id: i64) -> AppResult<()> {
    sqlx::query("DELETE FROM transaction_persons WHERE person_id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    sqlx::query("DELETE FROM activity_persons WHERE person_id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    sqlx::query("DELETE FROM persons WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}

// Use macros to generate junction table operations
impl_person_junction!(transaction, transaction_persons, transaction);
impl_person_junction!(activity, activity_persons, activity);
impl_person_junction!(todo, todo_persons, todo);
impl_person_junction!(journal, journal_persons, journal);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::init_tables;
    use crate::db::transactions::add_transaction;

    async fn create_test_pool() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        init_tables(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn test_upsert_person_new() {
        let pool = create_test_pool().await;
        let id = upsert_person(&pool, "John Doe").await.unwrap();
        assert!(id > 0);
    }

    #[tokio::test]
    async fn test_upsert_person_duplicate() {
        let pool = create_test_pool().await;
        let id1 = upsert_person(&pool, "Duplicate").await.unwrap();
        let id2 = upsert_person(&pool, "Duplicate").await.unwrap();
        assert_eq!(id1, id2);
    }

    #[tokio::test]
    async fn test_list_persons_sorted() {
        let pool = create_test_pool().await;
        upsert_person(&pool, "Zoe").await.unwrap();
        upsert_person(&pool, "Alice").await.unwrap();
        upsert_person(&pool, "Mike").await.unwrap();

        let persons = list_persons(&pool).await.unwrap();
        assert_eq!(persons.len(), 3);
        assert_eq!(persons[0].name, "Alice");
        assert_eq!(persons[1].name, "Mike");
        assert_eq!(persons[2].name, "Zoe");
    }

    #[tokio::test]
    async fn test_set_transaction_persons() {
        let pool = create_test_pool().await;

        let tx_id = add_transaction(&pool, 100.0, "test", "Test", None, None).await.unwrap();
        let person1_id = upsert_person(&pool, "Person1").await.unwrap();
        let person2_id = upsert_person(&pool, "Person2").await.unwrap();

        set_transaction_persons(&pool, tx_id, &[person1_id, person2_id]).await.unwrap();

        let persons = get_transaction_persons(&pool, tx_id).await.unwrap();
        assert_eq!(persons.len(), 2);
    }

    #[tokio::test]
    async fn test_delete_person() {
        let pool = create_test_pool().await;
        let id = upsert_person(&pool, "To Delete").await.unwrap();
        delete_person(&pool, id).await.unwrap();
        let persons = list_persons(&pool).await.unwrap();
        assert!(persons.is_empty());
    }
}
