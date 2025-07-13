use serde::{Serialize, Deserialize};
use validator::Validate;

#[derive(Debug, Deserialize)]
pub struct ModifyUsersModuleRequest {
    pub user_ids: Vec<i64>,
}

#[derive(Debug, Serialize)]
pub struct RoleResponse {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub admin: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<db::models::user::Model> for RoleResponse {
    fn from(user: db::models::user::Model) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            admin: user.admin,
            created_at: user.created_at.to_rfc3339(),
            updated_at: user.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RoleQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub query: Option<String>,
    pub email: Option<String>,
    pub username: Option<String>,
    pub sort: Option<String>,
}

#[derive(serde::Serialize)]
pub struct PaginatedRoleResponse {
    pub users: Vec<RoleResponse>,
    pub page: u32,
    pub per_page: u32,
    pub total: u64,
}

#[derive(Debug, Deserialize, Validate)]
pub struct EditRoleRequest {
    #[validate(length(min = 1, message = "Request must include a non-empty list of user_ids"))]
    pub user_ids: Vec<i64>,
}