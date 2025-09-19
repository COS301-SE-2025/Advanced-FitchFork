//! Module and role request/response models.
//!
//! Provides data structures for:
//! - Module creation, validation, and response (`ModuleRequest`, `ModuleResponse`).
//! - Assigning users to modules (`ModifyUsersModuleRequest`, `EditRoleRequest`).
//! - User role responses and pagination (`RoleResponse`, `PaginatedRoleResponse`).
//!
//! Includes `From` implementations to convert database models into API-friendly responses.

use chrono::{Datelike, Utc};
use serde::{Deserialize, Serialize};
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

lazy_static::lazy_static! {
    static ref MODULE_CODE_REGEX: regex::Regex = regex::Regex::new("^[A-Z]{3}\\d{3}$").unwrap();
}

#[derive(Debug, Deserialize, Validate)]
pub struct ModuleRequest {
    #[validate(regex(
        path = &*MODULE_CODE_REGEX,
        message = "Module code must be in format ABC123"
    ))]
    pub code: String,

    #[validate(range(min = Utc::now().year(), message = "Year must be current year or later"))]
    pub year: i32,

    #[validate(length(max = 1000, message = "Description must be at most 1000 characters"))]
    pub description: Option<String>,

    #[validate(range(min = 1, message = "Credits must be a positive number"))]
    pub credits: i32,
}

#[derive(Debug, Serialize)]
pub struct ModuleResponse {
    pub id: i64,
    pub code: String,
    pub year: i32,
    pub description: Option<String>,
    pub credits: i32,
    pub created_at: String,
    pub updated_at: String,
}

impl From<db::models::module::Model> for ModuleResponse {
    fn from(module: db::models::module::Model) -> Self {
        Self {
            id: module.id,
            code: module.code,
            year: module.year,
            description: module.description,
            credits: module.credits,
            created_at: module.created_at.to_rfc3339(),
            updated_at: module.updated_at.to_rfc3339(),
        }
    }
}
