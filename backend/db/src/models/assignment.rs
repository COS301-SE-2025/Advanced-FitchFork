use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Assignment {
    pub id: i64,
    pub module_id: i64,
    pub name: String,
    pub due_date: Option<String>,
}
