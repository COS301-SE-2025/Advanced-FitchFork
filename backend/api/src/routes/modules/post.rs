//! Module creation routes.
//!
//! Provides the `POST /api/modules` endpoint for creating new university modules.  
//! Only accessible by admin users. Responses follow the standard `ApiResponse` format.

use crate::response::ApiResponse;
use crate::routes::modules::common::{ModuleRequest, ModuleResponse};
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use chrono::{Datelike, Utc};
use db::models::module::Model as Module;
use util::state::AppState;
use validator::Validate;

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
pub async fn create(
    State(state): State<AppState>,
    Json(req): Json<ModuleRequest>,
) -> impl IntoResponse {
    let db = state.db();

    if let Err(validation_errors) = req.validate() {
        let error_message = common::format_validation_errors(&validation_errors);
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<ModuleResponse>::error(error_message)),
        );
    }

    if !req.code.is_match(r"^[A-Z]{3}\d{3}$") {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<ModuleResponse>::error(
                "Code format must be ABC123 (3 uppercase letters followed by 3 digits)",
            )),
        );
    }

    let current_year = Utc::now().year();
    if req.year < current_year.into() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<ModuleResponse>::error(format!(
                "Year must be {} or later",
                current_year
            ))),
        );
    }

    if req.credits < 0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<ModuleResponse>::error(
                "Credits must be a positive number",
            )),
        );
    }

    if req.credits > i32::MAX as i64 {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<ModuleResponse>::error(
                "Credits value is too large",
            )),
        );
    }

    match ModuleService::create(CreateModule {
        id: None,
        code: req.code,
        year: req.year,
        description: req.description,
        credits: req.credits,
    })
    .await
    {
        Ok(module) => {
            let response = ModuleResponse::from(module);
            (
                StatusCode::CREATED,
                Json(ApiResponse::success(
                    response,
                    "Module created successfully",
                )),
            )
        }
        Err(e) => {
            if let sea_orm::DbErr::Exec(err) = &e {
                if err
                    .to_string()
                    .contains("UNIQUE constraint failed: modules.code")
                {
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
