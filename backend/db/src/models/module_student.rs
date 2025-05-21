use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ModuleStudent {
    pub module_id: i64,
    pub user_id: i64,
}
