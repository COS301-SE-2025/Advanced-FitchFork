use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct TaskResponse {
    pub id: i64,
    pub task_number: i64,
    pub name: String,
    pub command: String,
    pub created_at: String,
    pub updated_at: String,
}