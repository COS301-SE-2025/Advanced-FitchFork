use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::Serialize;

/// Represents a user in the `users` table.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    /// Primary key ID (auto-incremented).
    #[sea_orm(primary_key)]
    pub id: i64,
    /// Unique student number.
    pub username: String,
    /// User's unique email address.
    pub email: String,
    /// Securely hashed password string.
    pub password_hash: String,
    /// Whether the user has admin privileges.
    pub admin: bool,
    /// Timestamp when the user was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp when the user was last updated.
    pub updated_at: DateTime<Utc>,
    //User profile picture
    pub profile_picture_path: Option<String>,
}

/// This enum would define relations if any exist. Currently unused.
#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        panic!("No RelationDef implemented")
    }
}

impl ActiveModelBehavior for ActiveModel {}

/// Struct returned by `get_module_roles`, summarizing a user's role in a module.
#[derive(Debug, Clone)]
pub struct UserModuleRole {
    pub module_id: i64,
    pub module_code: String,
    pub module_year: i32,
    pub module_description: Option<String>,
    pub module_credits: i32,
    pub module_created_at: String,
    pub module_updated_at: String,
    pub role: String,
}
