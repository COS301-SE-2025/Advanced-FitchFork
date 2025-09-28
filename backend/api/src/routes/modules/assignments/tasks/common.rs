use db::models::assignment_task::TaskType;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct TaskResponse {
    pub id: i64,
    pub task_number: i64,
    pub name: String,
    pub command: String,
    pub task_type: TaskType,
    pub created_at: String,
    pub updated_at: String,
}
