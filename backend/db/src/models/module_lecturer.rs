use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};

/// Represents the relationship between a module and a lecturer (user).
///
/// This struct maps to a row in the `module_lecturers` table, which associates
/// a user (assumed to be a lecturer) with a specific module.
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ModuleLecturer {
    pub module_id: i64,
    pub user_id: i64,
}

impl ModuleLecturer {
    /// Inserts a new lecturer-module association into the database.
    ///
    /// # Arguments
    ///
    /// * `pool` - Optional database connection pool. If `None`, a default pool is retrieved.
    /// * `module_id` - The ID of the module.
    /// * `user_id` - The ID of the user to be added as a lecturer to the module.
    ///
    /// # Returns
    ///
    /// Returns the newly created `ModuleLecturer` object.
    ///
    /// # Errors
    ///
    /// Returns a `sqlx::Error` if the insertion fails.
    pub async fn create(
        pool: Option<&SqlitePool>,
        module_id: i64,
        user_id: i64,
    ) -> sqlx::Result<Self> {
        let pool = pool.unwrap_or_else(|| crate::pool::get());
        sqlx::query("INSERT INTO module_lecturers (module_id, user_id) VALUES (?, ?)")
            .bind(module_id)
            .bind(user_id)
            .execute(pool)
            .await?;

        Ok(Self { module_id, user_id })
    }

    /// Deletes a specific lecturer-module association.
    ///
    /// This does not delete the user or module themselves, only the association.
    ///
    /// # Arguments
    ///
    /// * `pool` - Optional database connection pool. If `None`, a default pool is used.
    /// * `module_id` - The module ID to disassociate.
    /// * `user_id` - The user ID to disassociate.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the deletion is successful.
    ///
    /// # Errors
    ///
    /// Returns a `sqlx::Error` if the deletion fails.
    pub async fn delete(
        pool: Option<&SqlitePool>,
        module_id: i64,
        user_id: i64,
    ) -> sqlx::Result<()> {
        let pool = pool.unwrap_or_else(|| crate::pool::get());
        sqlx::query("DELETE FROM module_lecturers WHERE module_id = ? AND user_id = ?")
            .bind(module_id)
            .bind(user_id)
            .execute(pool)
            .await?;

        Ok(())
    }

    /// Retrieves all module-lecturer associations in the system.
    ///
    /// # Arguments
    ///
    /// * `pool` - Optional database connection pool. If `None`, a default pool is used.
    ///
    /// # Returns
    ///
    /// A vector of all `ModuleLecturer` records in the database.
    ///
    /// # Errors
    ///
    /// Returns a `sqlx::Error` if the query fails.
    pub async fn get_all(pool: Option<&SqlitePool>) -> sqlx::Result<Vec<Self>> {
        let pool = pool.unwrap_or_else(|| crate::pool::get());
        let records =
            sqlx::query_as::<_, ModuleLecturer>("SELECT module_id, user_id FROM module_lecturers")
                .fetch_all(pool)
                .await?;

        Ok(records)
    }

    /// Retrieves all lecturers associated with a specific module.
    ///
    /// # Arguments
    ///
    /// * `pool` - Optional database connection pool. If `None`, a default pool is used.
    /// * `module_id` - The ID of the module to look up.
    ///
    /// # Returns
    ///
    /// A vector of `ModuleLecturer` records associated with the given module.
    ///
    /// # Errors
    ///
    /// Returns a `sqlx::Error` if the query fails.
    pub async fn get_by_id(pool: Option<&SqlitePool>, module_id: i64) -> sqlx::Result<Vec<Self>> {
        let pool = pool.unwrap_or_else(|| crate::pool::get());
        let records = sqlx::query_as::<_, ModuleLecturer>(
            "SELECT module_id, user_id FROM module_lecturers WHERE module_id = ?",
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
    use crate::models::module_lecturer::ModuleLecturer;
    use crate::models::user::User;
    use crate::{create_test_db, delete_database};

    #[tokio::test]
    async fn test_module_lecturer_create_and_get() {
        let pool = create_test_db(Some("test_module_lecturer_create_and_get.db")).await;

        // Create module and user
        let module = Module::create(Some(&pool), "COS341", 2025, Some("Theory of Computation"))
            .await
            .unwrap();
        let user = User::create(Some(&pool), "u22222222", "lecturer1@test.com", "hash", true)
            .await
            .unwrap();

        // Create lecturer-module relation
        let relation = ModuleLecturer::create(Some(&pool), module.id, user.id)
            .await
            .unwrap();
        assert_eq!(relation.module_id, module.id);
        assert_eq!(relation.user_id, user.id);

        // Get all lecturers
        let all = ModuleLecturer::get_all(Some(&pool)).await.unwrap();
        assert!(all
            .iter()
            .any(|r| r.module_id == module.id && r.user_id == user.id));

        // Get by module ID
        let found = ModuleLecturer::get_by_id(Some(&pool), module.id)
            .await
            .unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].user_id, user.id);

        pool.close().await;
        delete_database("test_module_lecturer_create_and_get.db");
    }

    #[tokio::test]
    async fn test_module_lecturer_delete() {
        let pool = create_test_db(Some("test_module_lecturer_delete.db")).await;

        // Create module and user
        let module = Module::create(Some(&pool), "COS110", 2025, Some("Intro to Programming"))
            .await
            .unwrap();
        let user = User::create(Some(&pool), "u33333333", "lecturer2@test.com", "hash", true)
            .await
            .unwrap();

        // Create relation
        ModuleLecturer::create(Some(&pool), module.id, user.id)
            .await
            .unwrap();

        // Confirm it exists
        let before = ModuleLecturer::get_by_id(Some(&pool), module.id)
            .await
            .unwrap();
        assert_eq!(before.len(), 1);

        // Delete relation
        ModuleLecturer::delete(Some(&pool), module.id, user.id)
            .await
            .unwrap();

        // Confirm it's deleted
        let after = ModuleLecturer::get_by_id(Some(&pool), module.id)
            .await
            .unwrap();
        assert!(after.is_empty());

        pool.close().await;
        delete_database("test_module_lecturer_delete.db");
    }
}
