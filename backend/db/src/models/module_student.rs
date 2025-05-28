use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};

use super::{module::Module, user::User};

/// Represents the relationship between a module and a student.
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ModuleStudent {
    pub module_id: i64,
    pub user_id: i64,
}

impl ModuleStudent {
    /// Creates a new student-module association in the database.
    ///
    /// # Arguments
    ///
    /// * `pool` - Optional database connection pool. If `None`, a default pool is used.
    /// * `module_id` - ID of the module to associate with.
    /// * `user_id` - ID of the user to associate as a student.
    ///
    /// # Returns
    ///
    /// Returns the newly created `ModuleStudent` instance.
    ///
    /// # Errors
    ///
    /// Returns a `sqlx::Error` if the database insertion fails.
    pub async fn create(
        pool: Option<&SqlitePool>,
        module_id: i64,
        user_id: i64,
    ) -> sqlx::Result<Self> {
        let pool = pool.unwrap_or_else(|| crate::pool::get());
        sqlx::query("INSERT INTO module_students (module_id, user_id) VALUES (?, ?)")
            .bind(module_id)
            .bind(user_id)
            .execute(pool)
            .await?;

        Ok(Self { module_id, user_id })
    }

    /// Deletes a specific student-module association from the database.
    ///
    /// This does not delete the user or module, only the relationship.
    ///
    /// # Arguments
    ///
    /// * `pool` - Optional database connection pool. If `None`, a default pool is used.
    /// * `module_id` - ID of the module to disassociate.
    /// * `user_id` - ID of the user to disassociate as a student.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the deletion was successful.
    ///
    /// # Errors
    ///
    /// Returns a `sqlx::Error` if the deletion query fails.
    pub async fn delete(
        pool: Option<&SqlitePool>,
        module_id: i64,
        user_id: i64,
    ) -> sqlx::Result<()> {
        let pool = pool.unwrap_or_else(|| crate::pool::get());
        sqlx::query("DELETE FROM module_students WHERE module_id = ? AND user_id = ?")
            .bind(module_id)
            .bind(user_id)
            .execute(pool)
            .await?;

        Ok(())
    }

    /// Retrieves all student-module associations from the database.
    ///
    /// # Arguments
    ///
    /// * `pool` - Optional database connection pool. If `None`, a default pool is used.
    ///
    /// # Returns
    ///
    /// A vector of all `ModuleStudent` records.
    ///
    /// # Errors
    ///
    /// Returns a `sqlx::Error` if the query fails.
    pub async fn get_all(pool: Option<&SqlitePool>) -> sqlx::Result<Vec<Self>> {
        let pool = pool.unwrap_or_else(|| crate::pool::get());
        let records =
            sqlx::query_as::<_, ModuleStudent>("SELECT module_id, user_id FROM module_students")
                .fetch_all(pool)
                .await?;

        Ok(records)
    }

    /// Retrieves all students associated with a specific module.
    ///
    /// # Arguments
    ///
    /// * `pool` - Optional database connection pool. If `None`, a default pool is used.
    /// * `module_id` - ID of the module whose students should be fetched.
    ///
    /// # Returns
    ///
    /// A vector of `ModuleStudent` records associated with the given module.
    ///
    /// # Errors
    ///
    /// Returns a `sqlx::Error` if the query fails.
    pub async fn get_by_id(pool: Option<&SqlitePool>, module_id: i64) -> sqlx::Result<Vec<Self>> {
        let pool = pool.unwrap_or_else(|| crate::pool::get());
        let records = sqlx::query_as::<_, ModuleStudent>(
            "SELECT module_id, user_id FROM module_students WHERE module_id = ?",
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
            INNER JOIN module_students ON users.id = module_students.user_id
            WHERE module_students.module_id = ?;
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
            "
        SELECT modules.*
        FROM module_students
        INNER JOIN modules ON module_students.module_id = modules.id
        WHERE module_students.user_id = ?
        ",
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
    use crate::models::module_student::ModuleStudent;
    use crate::models::user::User;
    use crate::{create_test_db, delete_database};

    #[tokio::test]
    async fn test_module_student_create_and_get() {
        let pool = create_test_db(Some("test_module_student_create_and_get.db")).await;

        // Create module and user
        let module = Module::create(Some(&pool), "COS333", 2025, Some("Networks"), 16)
            .await
            .unwrap();
        let user = User::create(Some(&pool), "u55555555", "student1@test.com", "hash", false)
            .await
            .unwrap();

        // Create student-module relation
        let relation = ModuleStudent::create(Some(&pool), module.id, user.id)
            .await
            .unwrap();
        assert_eq!(relation.module_id, module.id);
        assert_eq!(relation.user_id, user.id);

        // Get all relations
        let all = ModuleStudent::get_all(Some(&pool)).await.unwrap();
        assert!(all
            .iter()
            .any(|r| r.module_id == module.id && r.user_id == user.id));

        // Get by module ID
        let found = ModuleStudent::get_by_id(Some(&pool), module.id)
            .await
            .unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].user_id, user.id);

        pool.close().await;
        delete_database("test_module_student_create_and_get.db");
    }

    #[tokio::test]
    async fn test_module_student_delete() {
        let pool = create_test_db(Some("test_module_student_delete.db")).await;

        // Create module and user
        let module = Module::create(Some(&pool), "COS151", 2025, Some("Foundations of CS"), 16)
            .await
            .unwrap();
        let user = User::create(Some(&pool), "u44444444", "student2@test.com", "hash", false)
            .await
            .unwrap();

        // Create relation
        ModuleStudent::create(Some(&pool), module.id, user.id)
            .await
            .unwrap();

        // Verify it's inserted
        let before = ModuleStudent::get_by_id(Some(&pool), module.id)
            .await
            .unwrap();
        assert_eq!(before.len(), 1);

        // Delete
        ModuleStudent::delete(Some(&pool), module.id, user.id)
            .await
            .unwrap();

        // Verify deletion
        let after = ModuleStudent::get_by_id(Some(&pool), module.id)
            .await
            .unwrap();
        assert!(after.is_empty());

        pool.close().await;
        delete_database("test_module_student_delete.db");
    }
}
