use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::SqlitePool;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Assignment {
    pub id: i64,
    pub module_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub assignment_type: String, //must be "Assignment" or "Practical"
    pub available_from: String,
    pub due_date: String,
    pub created_at: String,
    pub updated_at: String,
}

impl Assignment {
    //Create Assignment
    pub async fn create(
        pool: &SqlitePool,
        module_id: i64,
        name: &str,
        description: Option<&str>,
        assignment_type: &str,
        available_from: &str,
        due_date: &str,
    ) -> sqlx::Result<Self> {
        let record = sqlx::query_as::<_, Assignment>(
            r#"
            INSERT INTO assignments (
                module_id, name, description, assignment_type, 
                available_from, due_date
            )
            VALUES (?, ?, ?, ?, ?, ?)
            RETURNING id, module_id, name, description, assignment_type, 
                      available_from, due_date, created_at, updated_at
            "#,
        )
        .bind(module_id)
        .bind(name)
        .bind(description)
        .bind(assignment_type)
        .bind(available_from)
        .bind(due_date)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    //delete assignment by id
    pub async fn delete_by_id(pool: &SqlitePool, id: i64) -> sqlx::Result<()> {
        sqlx::query("DELETE FROM assignments WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }
}
