use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::Row;
use sqlx::SqlitePool;

//TODO -> Make sure a user can only have one role per module
/// Represents a user in the system.
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i64,
    pub student_number: String,
    pub email: String,
    pub password_hash: String,
    pub admin: bool, //if admin is false, your role (student/tutor/lecturer) is linked to whatever module you are in
    pub created_at: String,
    pub updated_at: String,
}

//This is only used when you return all the modules a user has -> it needs to know what their role is per module depending on:
//module_lecturer
//module_tutor
//module_student
#[derive(Debug, Serialize, Deserialize)]
pub struct UserModuleRole {
    pub module_id: i64,
    pub module_code: String,
    pub module_year: i32,
    pub module_description: Option<String>,
    pub role: String, // "lecturer", "tutor", or "student"
}

impl User {
    /// Creates a new user in the database.
    ///
    /// # Arguments
    ///
    /// * `pool` - Optional database connection pool.
    /// * `student_number` - The student's university ID.
    /// * `email` - The user's email.
    /// * `password_hash` - The hashed password.
    /// * `admin` - Whether the user is an admin.
    ///
    /// # Returns
    ///
    /// Returns the newly created `User` record.
    pub async fn create(
        pool: Option<&SqlitePool>,
        student_number: &str,
        email: &str,
        password: &str,
        admin: bool,
    ) -> sqlx::Result<Self> {
        let password_hash = User::hash(password);
        let pool = pool.unwrap_or_else(|| crate::pool::get());
        let record: User = sqlx::query_as::<_, User>(
            "INSERT INTO users (student_number, email, password_hash, admin)
             VALUES (?, ?, ?, ?)
             RETURNING id, student_number, email, password_hash, admin, created_at, updated_at",
        )
        .bind(student_number)
        .bind(email)
        .bind(password_hash)
        .bind(admin)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    /// Deletes a user by their ID.
    ///
    /// # Arguments
    ///
    /// * `pool` - Optional database connection pool.
    /// * `id` - The ID of the user to delete.
    pub async fn delete_by_id(pool: Option<&SqlitePool>, id: i64) -> sqlx::Result<()> {
        let pool = pool.unwrap_or_else(|| crate::pool::get());
        sqlx::query("DELETE FROM users WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub fn hash(password: &str) -> String {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .expect("Failed to hash password")
            .to_string();
        password_hash
    }

    pub async fn verify(
        pool: Option<&SqlitePool>,
        student_number: &str,
        password: &str,
    ) -> sqlx::Result<Self> {
        // Grab the pool
        let pool = pool.unwrap_or_else(|| crate::pool::get());

        // Fetch the full user record (or Err if not found / on DB error)
        let user: User = sqlx::query_as::<_, User>(
            "SELECT id, student_number, email, password_hash, admin, created_at, updated_at
            FROM users
            WHERE student_number = ?",
        )
        .bind(student_number)
        .fetch_one(pool)
        .await?;

        // Parse the stored hash
        let parsed_hash = PasswordHash::new(&user.password_hash)
            .map_err(|e| sqlx::Error::Protocol(format!("Invalid stored password hash: {}", e)))?;

        // Verify the password
        if Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok()
        {
            Ok(user)
        } else {
            // Wrong password → treat as a protocol error
            Err(sqlx::Error::Protocol("Invalid credentials".into()))
        }
    }

    /// Fetches a user by their ID.
    ///
    /// # Arguments
    ///
    /// * `pool` - Optional database connection pool.
    /// * `id` - The ID of the user.
    ///
    /// # Returns
    ///
    /// Returns `Some(User)` if found, or `None` otherwise.
    pub async fn get_by_id(pool: Option<&SqlitePool>, id: i64) -> sqlx::Result<Option<User>> {
        let pool = pool.unwrap_or_else(|| crate::pool::get());
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    /// Fetches a user by their student number.
    ///
    /// # Arguments
    ///
    /// * `pool` - Optional database connection pool.
    /// * `student_number` - The student number to search for.
    ///
    /// # Returns
    ///
    /// Returns `Some(User)` if found, or `None` otherwise.
    pub async fn get_by_student_number(
        pool: Option<&SqlitePool>,
        student_number: &str,
    ) -> sqlx::Result<Option<User>> {
        let pool = pool.unwrap_or_else(|| crate::pool::get());
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE student_number = ?")
            .bind(student_number)
            .fetch_optional(pool)
            .await
    }

    //Okay this one is confusing -> It returns the modules that a user is in and what roles he has in each module
    //Note this is not tested yet
    //It returns multiple UserModuleRole

    /// Retrieves all modules a user is involved in and their corresponding role in each.
    ///
    /// # Arguments
    ///
    /// * `pool` - Optional database connection pool.
    /// * `user_id` - The ID of the user.
    ///
    /// # Returns
    ///
    /// A vector of `UserModuleRole` entries representing each module and the role the user has in it.
    ///
    /// # Notes
    ///
    /// This method checks the `module_lecturers`, `module_tutors`, and `module_students` tables.
    pub async fn get_module_roles(
        pool: Option<&SqlitePool>,
        user_id: i64,
    ) -> sqlx::Result<Vec<UserModuleRole>> {
        let pool = pool.unwrap_or_else(|| crate::pool::get());
        let mut roles = Vec::new();

        // Lecturer roles
        let rows = sqlx::query(
            r#"
            SELECT m.id as module_id, m.code as module_code, m.year as module_year, m.description as module_description
            FROM modules m
            INNER JOIN module_lecturers ml ON m.id = ml.module_id
            WHERE ml.user_id = ?
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        for row in rows {
            roles.push(UserModuleRole {
                module_id: row.get("module_id"),
                module_code: row.get("module_code"),
                module_year: row.get("module_year"),
                module_description: row.get("module_description"),
                role: "lecturer".to_string(),
            });
        }

        // Tutor roles
        let rows = sqlx::query(
            r#"
            SELECT m.id as module_id, m.code as module_code, m.year as module_year, m.description as module_description
            FROM modules m
            INNER JOIN module_tutors mt ON m.id = mt.module_id
            WHERE mt.user_id = ?
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        for row in rows {
            roles.push(UserModuleRole {
                module_id: row.get("module_id"),
                module_code: row.get("module_code"),
                module_year: row.get("module_year"),
                module_description: row.get("module_description"),
                role: "tutor".to_string(),
            });
        }

        // Student roles
        let rows = sqlx::query(
            r#"
            SELECT m.id as module_id, m.code as module_code, m.year as module_year, m.description as module_description
            FROM modules m
            INNER JOIN module_students ms ON m.id = ms.module_id
            WHERE ms.user_id = ?
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        for row in rows {
            roles.push(UserModuleRole {
                module_id: row.get("module_id"),
                module_code: row.get("module_code"),
                module_year: row.get("module_year"),
                module_description: row.get("module_description"),
                role: "student".to_string(),
            });
        }

        Ok(roles)
    }
}

#[cfg(test)]
mod tests {
    use crate::models::user::User;
    #[tokio::test]
    async fn test_user_create_and_find() {
        let pool = crate::create_test_db(Some("test_user_create_and_find.db")).await;
        let student_number = "u11111111";
        let email = "1231312@example.com";
        let password_hash = "hashed_password";

        let created_user = User::create(Some(&pool), &student_number, &email, password_hash, false)
            .await
            .unwrap();

        assert_eq!(created_user.student_number, student_number);
        assert_eq!(created_user.email, email);
        assert!(!created_user.admin);

        // Now fetch by ID
        let found_user = User::get_by_id(Some(&pool), created_user.id).await.unwrap();
        assert!(found_user.is_some());
        let found_user = found_user.unwrap();
        assert_eq!(found_user.email, email);

        pool.close().await;
        crate::delete_database("test_user_create_and_find.db");
    }

    #[tokio::test]
    async fn test_user_deletion() {
        let pool = crate::create_test_db(Some("test_user_deletion.db")).await;
        let user = User::create(Some(&pool), "u12345678", "delete@test.com", "hash", false)
            .await
            .unwrap();

        let user_id = user.id;

        let found = User::get_by_id(Some(&pool), user_id).await.unwrap();
        assert!(found.is_some());

        User::delete_by_id(Some(&pool), user_id).await.unwrap();

        let found_after_delete = User::get_by_id(Some(&pool), user_id).await.unwrap();
        assert!(found_after_delete.is_none());

        pool.close().await;
        crate::delete_database("test_user_deletion.db");
    }

    #[tokio::test]
    async fn test_get_by_student_number() {
        let pool = crate::create_test_db(Some("test_get_by_student_number.db")).await;
        let sn = "u99999999";
        User::create(Some(&pool), sn, "some@test.com", "hash", false)
            .await
            .unwrap();

        let found = User::get_by_student_number(Some(&pool), sn).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().student_number, sn);

        pool.close().await;
        crate::delete_database("test_get_by_student_number.db");
    }

    #[tokio::test]
    async fn test_hash_and_verify() {
        let pool = crate::create_test_db(Some("test_hash_and_verify.db")).await;
        let user = User::create(Some(&pool), "u12345678", "delete@test.com", "hash", false)
            .await
            .unwrap();

        let user_id = user.id;

        let found = User::get_by_id(Some(&pool), user_id).await.unwrap();
        assert!(found.is_some());

        let student_number = "u12345678";
        let password = "hash";

        let verified_user = User::verify(Some(&pool), student_number, password).await.unwrap();
        assert_eq!(verified_user.student_number, student_number);

        let unverified_user = User::verify(Some(&pool), "u99999999", "password").await;
        assert!(unverified_user.is_err(), "Expected Err but got {:?}", unverified_user);

        User::delete_by_id(Some(&pool), user_id).await.unwrap();

        pool.close().await;
        crate::delete_database("test_hash_and_verify.db");
    }

    #[tokio::test]
    async fn test_get_module_roles() {
        use crate::models::{
            module::Module, module_lecturer::ModuleLecturer, module_student::ModuleStudent,
            module_tutor::ModuleTutor, user::User,
        };

        let pool = crate::create_test_db(Some("test_get_module_roles.db")).await;

        // Create a user
        let user = User::create(Some(&pool), "u00000001", "roleuser@test.com", "pw", false)
            .await
            .unwrap();

        // Create 3 modules
        let lecturer_mod = Module::create(Some(&pool), "COS700", 2025, Some("Advanced Topics"))
            .await
            .unwrap();
        let tutor_mod = Module::create(Some(&pool), "COS701", 2025, Some("AI Practicals"))
            .await
            .unwrap();
        let student_mod = Module::create(Some(&pool), "COS702", 2025, Some("Seminar"))
            .await
            .unwrap();

        // Add user to modules with different roles
        ModuleLecturer::create(Some(&pool), lecturer_mod.id, user.id)
            .await
            .unwrap();
        ModuleTutor::create(Some(&pool), tutor_mod.id, user.id)
            .await
            .unwrap();
        ModuleStudent::create(Some(&pool), student_mod.id, user.id)
            .await
            .unwrap();

        // Fetch roles
        let roles = User::get_module_roles(Some(&pool), user.id).await.unwrap();

        // Verify results
        assert_eq!(roles.len(), 3);

        let mut roles_map = std::collections::HashMap::new();
        for role in roles {
            roles_map.insert(role.module_code.clone(), role.role.clone());
        }

        assert_eq!(
            roles_map.get("COS700").map(String::as_str),
            Some("lecturer")
        );
        assert_eq!(roles_map.get("COS701").map(String::as_str), Some("tutor"));
        assert_eq!(roles_map.get("COS702").map(String::as_str), Some("student"));

        pool.close().await;
        crate::delete_database("test_get_module_roles.db");
    }
}
