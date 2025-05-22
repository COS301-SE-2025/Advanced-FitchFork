use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ModuleTutor {
    pub module_id: i64,
    pub user_id: i64,
}

impl ModuleTutor {
    //Inset new tutor
    pub async fn create(
        pool: Option<&SqlitePool>,
        module_id: i64,
        user_id: i64,
    ) -> sqlx::Result<Self> {
        let pool = pool.unwrap_or_else(|| crate::pool::get());
        sqlx::query("INSERT INTO module_tutors (module_id, user_id) VALUES (?, ?)")
            .bind(module_id)
            .bind(user_id)
            .execute(pool)
            .await?;

        Ok(Self { module_id, user_id })
    }

    //Delete tutor relationship (not User themselves)
    pub async fn delete(
        pool: Option<&SqlitePool>,
        module_id: i64,
        user_id: i64,
    ) -> sqlx::Result<()> {
        let pool = pool.unwrap_or_else(|| crate::pool::get());
        sqlx::query("DELETE FROM module_tutors WHERE module_id = ? AND user_id = ?")
            .bind(module_id)
            .bind(user_id)
            .execute(pool)
            .await?;

        Ok(())
    }

    //Get all tutors (idk might be useful)
    pub async fn get_all(pool: Option<&SqlitePool>) -> sqlx::Result<Vec<Self>> {
        let pool = pool.unwrap_or_else(|| crate::pool::get());
        let records =
            sqlx::query_as::<_, ModuleTutor>("SELECT module_id, user_id FROM module_tutors")
                .fetch_all(pool)
                .await?;

        Ok(records)
    }

    //Get all tutors for a specific module
    pub async fn get_by_id(pool: Option<&SqlitePool>, module_id: i64) -> sqlx::Result<Vec<Self>> {
        let pool = pool.unwrap_or_else(|| crate::pool::get());
        let records = sqlx::query_as::<_, ModuleTutor>(
            "SELECT module_id, user_id FROM module_tutors WHERE module_id = ?",
        )
        .bind(module_id)
        .fetch_all(pool)
        .await?;

        Ok(records)
    }
}

#[cfg(test)]
mod tests {
    use crate::models::module::Module;
    use crate::models::module_tutor::ModuleTutor;
    use crate::models::user::User;
    use crate::{create_test_db, delete_database};

    #[tokio::test]
    async fn test_module_tutor_create_and_get() {
        let pool = create_test_db(Some("test_module_tutor_create_and_get.db")).await;

        // Create module and user
        let module = Module::create(Some(&pool), "COS314", 2025, Some("Compilers"))
            .await
            .unwrap();
        let user = User::create(Some(&pool), "u88888888", "tutor1@test.com", "hash", false)
            .await
            .unwrap();

        // Create tutor record
        let tutor = ModuleTutor::create(Some(&pool), module.id, user.id)
            .await
            .unwrap();
        assert_eq!(tutor.module_id, module.id);
        assert_eq!(tutor.user_id, user.id);

        // Get all tutors
        let all = ModuleTutor::get_all(Some(&pool)).await.unwrap();
        assert!(!all.is_empty());
        assert!(all
            .iter()
            .any(|t| t.module_id == module.id && t.user_id == user.id));

        // Get tutors by module ID
        let module_tutors = ModuleTutor::get_by_id(Some(&pool), module.id)
            .await
            .unwrap();
        assert_eq!(module_tutors.len(), 1);
        assert_eq!(module_tutors[0].user_id, user.id);

        delete_database("test_module_tutor_create_and_get.db");
    }

    #[tokio::test]
    async fn test_module_tutor_delete() {
        let pool = create_test_db(Some("test_module_tutor_delete.db")).await;

        // Create module and user
        let module = Module::create(Some(&pool), "COS226", 2025, Some("Discrete Structures"))
            .await
            .unwrap();
        let user = User::create(Some(&pool), "u77777777", "tutor2@test.com", "hash", false)
            .await
            .unwrap();

        // Create tutor record
        ModuleTutor::create(Some(&pool), module.id, user.id)
            .await
            .unwrap();

        // Ensure it exists
        let before = ModuleTutor::get_by_id(Some(&pool), module.id)
            .await
            .unwrap();
        assert_eq!(before.len(), 1);

        // Delete tutor relation
        ModuleTutor::delete(Some(&pool), module.id, user.id)
            .await
            .unwrap();

        // Ensure it is gone
        let after = ModuleTutor::get_by_id(Some(&pool), module.id)
            .await
            .unwrap();
        assert!(after.is_empty());

        delete_database("test_module_tutor_delete.db");
    }
}
