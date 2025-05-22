use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Module {
    pub id: i64,
    pub code: String,
    pub year: i32,
    pub description: Option<String>,
}

impl Module {
    //create new module
    pub async fn create(
        pool: Option<&SqlitePool>,
        code: &str,
        year: i32,
        description: Option<&str>,
    ) -> sqlx::Result<Self> {
        let pool = pool.unwrap_or_else(|| crate::pool::get());
        let record = sqlx::query_as::<_, Module>(
            r#"
            INSERT INTO modules (code, year, description)
            VALUES (?, ?, ?)
            RETURNING id, code, year, description
            "#,
        )
        .bind(code)
        .bind(year)
        .bind(description)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    //Delete module by id
    pub async fn delete_by_id(pool: Option<&SqlitePool>, id: i64) -> sqlx::Result<()> {
        let pool = pool.unwrap_or_else(|| crate::pool::get());
        sqlx::query("DELETE FROM modules WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    //get module by id
    pub async fn get_by_id(pool: Option<&SqlitePool>, id: i64) -> sqlx::Result<Option<Self>> {
        let pool = pool.unwrap_or_else(|| crate::pool::get());
        sqlx::query_as::<_, Module>("SELECT id, code, year, description FROM modules WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    //get all modules
    pub async fn get_all(pool: Option<&SqlitePool>) -> sqlx::Result<Vec<Self>> {
        let pool = pool.unwrap_or_else(|| crate::pool::get());
        sqlx::query_as::<_, Module>("SELECT id, code, year, description FROM modules")
            .fetch_all(pool)
            .await
    }
}

#[cfg(test)]
mod tests {
    use crate::models::module::Module;
    use crate::{create_test_db, delete_database};

    #[tokio::test]
    async fn test_module_create_and_find() {
        let pool = create_test_db(Some("test_module_create_and_find.db")).await;

        let code = "COS301";
        let year = 2025;
        let description = Some("Software Engineering");

        let created = Module::create(Some(&pool), code, year, description)
            .await
            .unwrap();

        assert_eq!(created.code, code);
        assert_eq!(created.year, year);
        assert_eq!(created.description.as_deref(), description);

        let found = Module::get_by_id(Some(&pool), created.id).await.unwrap();
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.code, code);
        assert_eq!(found.year, year);

        delete_database("test_module_create_and_find.db");
    }

    #[tokio::test]
    async fn test_module_deletion() {
        let pool = create_test_db(Some("test_module_deletion.db")).await;

        let module = Module::create(Some(&pool), "COS110", 2025, Some("Intro to CS"))
            .await
            .unwrap();
        let id = module.id;

        let found = Module::get_by_id(Some(&pool), id).await.unwrap();
        assert!(found.is_some());

        Module::delete_by_id(Some(&pool), id).await.unwrap();

        let after_delete = Module::get_by_id(Some(&pool), id).await.unwrap();
        assert!(after_delete.is_none());

        delete_database("test_module_deletion.db");
    }

    #[tokio::test]
    async fn test_module_get_all() {
        let pool = create_test_db(Some("test_module_get_all.db")).await;

        let m1 = Module::create(Some(&pool), "COS132", 2024, Some("Programming"))
            .await
            .unwrap();
        let m2 = Module::create(Some(&pool), "COS212", 2024, None)
            .await
            .unwrap();

        let all = Module::get_all(Some(&pool)).await.unwrap();
        let codes: Vec<String> = all.iter().map(|m| m.code.clone()).collect();

        assert!(codes.contains(&m1.code));
        assert!(codes.contains(&m2.code));
        assert!(all.len() >= 2);

        delete_database("test_module_get_all.db");
    }
}
