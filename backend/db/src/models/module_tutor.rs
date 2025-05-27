use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};

use super::{module::Module, user::User};

#[derive(Debug, Serialize, Deserialize, FromRow)]

/// Represents the relationship between a module and a tutor.
pub struct ModuleTutor {
    pub module_id: i64,
    pub user_id: i64,
}

impl ModuleTutor {
    /// Creates a new tutor-module association in the database.
    ///
    /// # Arguments
    ///
    /// * `pool` - Optional reference to a `SqlitePool`. If `None`, the default pool is used.
    /// * `module_id` - The ID of the module to associate with.
    /// * `user_id` - The ID of the user to associate as a tutor.
    ///
    /// # Returns
    ///
    /// Returns the newly created `ModuleTutor` instance on success.
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
        sqlx::query("INSERT INTO module_tutors (module_id, user_id) VALUES (?, ?)")
            .bind(module_id)
            .bind(user_id)
            .execute(pool)
            .await?;

        Ok(Self { module_id, user_id })
    }

    /// Deletes a tutor-module association from the database.
    ///
    /// This does not delete the user or the module themselves, only their association.
    ///
    /// # Arguments
    ///
    /// * `pool` - Optional reference to a `SqlitePool`. If `None`, the default pool is used.
    /// * `module_id` - The ID of the module to disassociate.
    /// * `user_id` - The ID of the tutor to disassociate.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if deletion succeeds.
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
        sqlx::query("DELETE FROM module_tutors WHERE module_id = ? AND user_id = ?")
            .bind(module_id)
            .bind(user_id)
            .execute(pool)
            .await?;

        Ok(())
    }

    /// Retrieves all tutor-module associations from the database.
    ///
    /// # Arguments
    ///
    /// * `pool` - Optional reference to a `SqlitePool`. If `None`, the default pool is used.
    ///
    /// # Returns
    ///
    /// A vector of all `ModuleTutor` records.
    ///
    /// # Errors
    ///
    /// Returns a `sqlx::Error` if the query fails.
    pub async fn get_all(pool: Option<&SqlitePool>) -> sqlx::Result<Vec<Self>> {
        let pool = pool.unwrap_or_else(|| crate::pool::get());
        let records =
            sqlx::query_as::<_, ModuleTutor>("SELECT module_id, user_id FROM module_tutors")
                .fetch_all(pool)
                .await?;

        Ok(records)
    }

    /// Retrieves all tutors associated with a specific module.
    ///
    /// # Arguments
    ///
    /// * `pool` - Optional reference to a `SqlitePool`. If `None`, the default pool is used.
    /// * `module_id` - The ID of the module for which tutors should be fetched.
    ///
    /// # Returns
    ///
    /// A vector of `ModuleTutor` records associated with the specified module.
    ///
    /// # Errors
    ///
    /// Returns a `sqlx::Error` if the query fails.
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

    pub async fn get_details_by_id(
        pool: Option<&SqlitePool>,
        module_id: i64,
    ) -> sqlx::Result<Vec<User>> {
        let pool = pool.unwrap_or_else(|| crate::pool::get());
        let records = sqlx::query_as::<_, User>(
            "SELECT users.*
            FROM users
            INNER JOIN module_tutors ON users.id = module_tutors.user_id
            WHERE module_tutors.module_id = ?;
        ",
        )
        .bind(module_id)
        .fetch_all(pool)
        .await?;

        Ok(records)
    }

    pub async fn get_by_user_id(
        pool: Option<&SqlitePool>,
        user_id: i64,
    ) -> sqlx::Result<Vec<Module>> {
        let pool = pool.unwrap_or_else(|| crate::pool::get());
        let records = sqlx::query_as::<_, Module>(
            "SELECT modules.*
        FROM module_tutors
        INNER JOIN modules ON module_tutors.module_id = modules.id
        WHERE module_tutors.user_id = ?",
        )
        .bind(user_id)
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
        let module = Module::create(Some(&pool), "COS314", 2025, Some("Compilers"), 16)
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

        pool.close().await;
        delete_database("test_module_tutor_create_and_get.db");
    }

    #[tokio::test]
    async fn test_module_tutor_delete() {
        let pool = create_test_db(Some("test_module_tutor_delete.db")).await;

        // Create module and user
        let module = Module::create(Some(&pool), "COS226", 2025, Some("Discrete Structures"), 16)
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

        pool.close().await;
        delete_database("test_module_tutor_delete.db");
    }
}
