use axum::{http::StatusCode, response::IntoResponse, Json};
use db::{
    models::module::Module,
    pool,
};
use serde::{Deserialize, Serialize};
use validator::Validate;
use chrono::{Utc, Datelike};

use crate::response::ApiResponse;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateModuleRequest {
    #[validate(regex(
        path = "MODULE_CODE_REGEX",
        message = "Module code must be in format ABC123"
    ))]
    pub code: String,

    #[validate(range(min = 2024, message = "Year must be current year or later"))]
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

impl From<Module> for ModuleResponse {
    fn from(module: Module) -> Self {
        Self {
            id: module.id,
            code: module.code,
            year: module.year,
            description: module.description,
            credits: module.credits,
            created_at: module.created_at,
            updated_at: module.updated_at,
        }
    }
}

lazy_static::lazy_static! {
    static ref MODULE_CODE_REGEX: regex::Regex = regex::Regex::new("^[A-Z]{3}\\d{3}$").unwrap();
}

/// POST /api/modules
///
/// Create a new university module. Only accessible by admin users.
///
/// ### Request Body
/// ```json
/// {
///   "code": "COS301",
///   "year": 2025,
///   "description": "Advanced Software Engineering",
///   "credits": 16
/// }
/// ```
///
/// ### Validation Rules
/// * `code`: required, must be uppercase alphanumeric (e.g., `^[A-Z]{3}\d{3}$`), unique
/// * `year`: required, must be the current year or later
/// * `description`: optional, max length 1000 characters
/// * `credits`: required, must be a positive integer
///
/// ### Responses
///
/// - `201 Created`  
/// ```json
/// {
///   "success": true,
///   "data": {
///     "id": 1,
///     "code": "COS301",
///     "year": 2025,
///     "description": "Advanced Software Engineering",
///     "credits": 16,
///     "created_at": "2025-05-23T18:00:00Z",
///     "updated_at": "2025-05-23T18:00:00Z"
///   },
///   "message": "Module created successfully"
/// }
/// ```
///
/// - `400 Bad Request` (validation failure)  
/// ```json
/// {
///   "success": false,
///   "message": "Invalid input: code format must be ABC123 and credits must be a positive number"
/// }
/// ```
///
/// - `403 Forbidden` (missing admin role)  
/// ```json
/// {
///   "success": false,
///   "message": "You do not have permission to perform this action"
/// }
/// ```
///
/// - `409 Conflict` (duplicate code)  
/// ```json
/// {
///   "success": false,
///   "message": "A module with this code already exists"
/// }
/// ```
///
/// - `500 Internal Server Error`  
/// ```json
/// {
///   "success": false,
///   "message": "Database error: detailed error here"
/// }
/// ```
pub async fn create(Json(req): Json<CreateModuleRequest>) -> impl IntoResponse {
    if let Err(validation_errors) = req.validate() {
        let error_message = common::format_validation_errors(&validation_errors);
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<ModuleResponse>::error(error_message)),
        );
    }

    let current_year = Utc::now().year();
    if req.year < current_year {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<ModuleResponse>::error(format!(
                "Year must be {} or later",
                current_year
            ))),
        );
    }

    let pool = pool::get();

    match Module::create(
        Some(pool),
        &req.code,
        req.year,
        req.description.as_deref(),
        req.credits,
    )
    .await
    {
        Ok(module) => {
            let response = ModuleResponse::from(module);
            (
                StatusCode::CREATED,
                Json(ApiResponse::success(response, "Module created successfully")),
            )
        }
        Err(e) => {
            if let Some(db_err) = e.as_database_error() {
                let msg = db_err.message();
                if msg.contains("modules.code") {
                    return (
                        StatusCode::CONFLICT,
                        Json(ApiResponse::<ModuleResponse>::error(
                            "A module with this code already exists",
                        )),
                    );
                }
            }

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<ModuleResponse>::error(format!(
                    "Database error: {}",
                    e
                ))),
            )
        }
    }
}