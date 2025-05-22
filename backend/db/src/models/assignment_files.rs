use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::SqlitePool;

//!NOTE
//! This currently does nothing
//! As in it's never called or seeded
//! The path environment is not used so you can "logically" save a file, but its not actually saved anywhere
//! There is no functionality to save, delete or retrieve the actual file
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AssignmentFiles {
    pub id: i64,
    pub assignment_id: i64,
    pub filename: String,
    pub path: String,
    pub uploaded_at: String,
}

impl AssignmentFiles {
    //Create AssignmentFile
    pub async fn create(
        pool: Option<&SqlitePool>,
        assignment_id: i64,
        filename: &str,
        path: &str,
        uploaded_at: &str,
    ) -> sqlx::Result<Self> {
        let pool = pool.unwrap_or_else(|| crate::pool::get());
        let record = sqlx::query_as::<_, Assignment>(
            r#"
            INSERT INTO assignment_files (
                assignment_id, filename, path, uploaded_at
            )
            VALUES (?, ?, ?, ?)
            RETURNING id, assignment_id, filename, path, uploaded_at
            "#,
        )
        .bind(assignment_id)
        .bind(filename)
        .bind(path)
        .bind(uploaded_at)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    //delete assignmentFile by id
    pub async fn delete_by_id(pool: Option<&SqlitePool>, id: i64) -> sqlx::Result<()> {
        let pool = pool.unwrap_or_else(|| crate::pool::get());
        sqlx::query("DELETE FROM assignment_files WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    //get all assignmentFile
    pub async fn get_all(pool: Option<&SqlitePool>) -> sqlx::Result<Vec<Self>> {
        let pool = pool.unwrap_or_else(|| crate::pool::get());
        sqlx::query_as::<_, Assignment>("SELECT * FROM assignment_files")
            .fetch_all(pool)
            .await
    }

    //get assignmentFile by id
    pub async fn get_by_id(pool: Option<&SqlitePool>, id: i64) -> sqlx::Result<Option<Self>> {
        let pool = pool.unwrap_or_else(|| crate::pool::get());
        sqlx::query_as::<_, Assignment>("SELECT * FROM assignment_files WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await
    }
}
