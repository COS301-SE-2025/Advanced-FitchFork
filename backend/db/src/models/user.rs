use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i64,
    pub student_number: String,
    pub email: String,
    pub password_hash: String,
    pub admin: bool, //if admin is false, your role (student/tutor/lecturer) is linked to whatever module you are in
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
}
