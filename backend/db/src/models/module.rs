use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Module {
    pub id: i64,
    pub code: String,
    pub year: i32,
    pub description: Option<String>,
}
