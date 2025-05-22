use serde::{Deserialize, Serialize};
use sqlx::Row;
use sqlx::{FromRow, SqlitePool};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i64,
    pub student_number: String,
    pub email: String,
    pub password_hash: String,
    pub admin: bool, //if admin is false, your role (student/tutor/lecturer) is linked to whatever module you are in
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
    //Creates a new User in the users table
    pub async fn create(
        pool: &SqlitePool,
        student_number: &str,
        email: &str,
        password_hash: &str,
        admin: bool,
    ) -> sqlx::Result<Self> {
        let record: User = sqlx::query_as::<_, User>(
            "INSERT INTO users (student_number, email, password_hash, admin)
             VALUES (?, ?, ?, ?)
             RETURNING id, student_number, email, password_hash, admin",
        )
        .bind(student_number)
        .bind(email)
        .bind(password_hash)
        .bind(admin)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    //Deletes a user by id
    pub async fn delete_by_id(pool: &SqlitePool, id: i64) -> sqlx::Result<()> {
        sqlx::query("DELETE FROM users WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    //Returns a user by ID
    pub async fn find_by_id(pool: &SqlitePool, id: i64) -> sqlx::Result<Option<User>> {
        sqlx::query_as::<_, User>(
            "SELECT id, student_number, email, password_hash, admin FROM users WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    //Returns a user by student number
    pub async fn find_by_student_number(
        pool: &SqlitePool,
        student_number: &str,
    ) -> sqlx::Result<Option<User>> {
        sqlx::query_as::<_, User>(
            "SELECT id, student_number, email, password_hash, admin FROM users WHERE student_number = ?",
        )
        .bind(student_number)
        .fetch_optional(pool)
        .await
    }

    //Okay this one is cofusing -> It returns the modules that a user is in and what roles he has in each module
    //Note this is not tested yet
    //It returns multiple UserModuleRole
    pub async fn get_module_roles(
        pool: &SqlitePool,
        user_id: i64,
    ) -> sqlx::Result<Vec<UserModuleRole>> {
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
