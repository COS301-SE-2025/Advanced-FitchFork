use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub password: String,
    pub role: String,
}

impl User {
    pub async fn create(
        pool: &SqlitePool,
        email: &str,
        password: &str,
        role: &str,
    ) -> sqlx::Result<Self> {
        let record: User = sqlx::query_as::<_, User>(
            "INSERT INTO users (email, password, role) VALUES (?, ?, ?)
             RETURNING id, email, password, role",
        )
        .bind(email)
        .bind(password)
        .bind(role)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }
}
