use axum::{http::StatusCode, response::IntoResponse, Json};
use db::{
    models::module::Module,
    pool,
};
use serde::{Deserialize, Serialize};
use validator::Validate;
use chrono::{Utc, Datelike};
use crate::response::ApiResponse;
use crate::auth::claims::AuthUser;
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


#[derive(Debug, Deserialize)]
pub struct ModifyUsersModuleRequest {
    pub user_ids: Vec<i64>,
}

#[derive(Debug, Serialize)]
pub struct ConflictData {
    pub already_assigned: Vec<i64>,
}

#[derive(Debug, Serialize)]
pub struct PersonnelResponse {
    pub id: i64,
    pub student_number: String,
    pub email: String,
    pub admin: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<db::models::user::User> for PersonnelResponse {
    fn from(user: db::models::user::User) -> Self {
        Self {
            id: user.id,
            student_number: user.student_number,
            email: user.email,
            admin: user.admin,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
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



/// POST /api/modules/:module_id/lecturers
///
/// Assign one or more users as lecturers to a module. Admin only.
///
/// ### Request Body
/// ```json
/// {
///   "user_ids": [1, 2]
/// }
/// ```
///
/// ### Validation Rules
/// - `user_ids`: must be a non-empty list of valid user IDs.
/// - All users must exist.
/// - Each user must not already be assigned as a lecturer.
///
/// ### Responses
///
/// - `200 OK`  
/// ```json
/// {
///   "success": true,
///   "data": {},
///   "message": "Lecturers assigned to module successfully"
/// }
/// ```
///
/// - `400 Bad Request`  
/// ```json
/// {
///   "success": false,
///   "message": "Request must include a non-empty list of user_ids"
/// }
/// ```
///
/// - `403 Forbidden`  
/// ```json
/// {
///   "success": false,
///   "message": "You do not have permission to perform this action"
/// }
/// ```
///
/// - `404 Not Found`  
/// ```json
/// {
///   "success": false,
///   "message": "User with ID 3 does not exist"
/// }
/// ```
///
/// - `409 Conflict`  
/// ```json
/// {
///   "success": false,
///   "message": "Some users are already lecturers for this module"
/// }
/// ```

pub async fn assign_lecturers(axum::extract::Path(module_id): axum::extract::Path<i64>, AuthUser(claims): AuthUser, Json(body): Json<ModifyUsersModuleRequest>, ) -> impl IntoResponse {
    if !claims.admin {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<()> {
                success: false,
                data: None,
                message: "You do not have permission to perform this action".into(),
            }),
        );
    }

    if body.user_ids.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()> {
                success: false,
                data: None,
                message: "Request must include a non-empty list of user_ids".into(),
            }),
        );
    }

    let pool = pool::get();

    let module_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM modules WHERE id = ?)"
    )
        .bind(module_id)
        .fetch_one(pool)
        .await
        .unwrap_or(false);

    if !module_exists {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()> {
                success: false,
                data: None,
                message: "Module not found".into(),
            }),
        );
    }

    let mut already_assigned = Vec::new();
    for &user_id in &body.user_ids {
        let user_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM users WHERE id = ?)"
        )
            .bind(user_id)
            .fetch_one(pool)
            .await
            .unwrap_or(false);

        if !user_exists {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<()> {
                    success: false,
                    data: None,
                    message: format!("User with ID {} does not exist", user_id),
                }),
            );
        }

        let is_already: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM module_lecturers WHERE module_id = ? AND user_id = ?)"
        )
            .bind(module_id)
            .bind(user_id)
            .fetch_one(pool)
            .await
            .unwrap_or(false);

        if is_already {
            already_assigned.push(user_id);
        } else {
            let _ = sqlx::query(
                "INSERT OR IGNORE INTO module_lecturers (module_id, user_id) VALUES (?, ?)"
            )
                .bind(module_id)
                .bind(user_id)
                .execute(pool)
                .await;
        }
    }

    if already_assigned.is_empty() {
        (
            StatusCode::OK,
            Json(ApiResponse::<()> {
                success: true,
                data: None,
                message: "Lecturers assigned to module successfully".into(),
            })
        )
    } else {
        (
            StatusCode::CONFLICT,
            Json(ApiResponse::<()> {
                success: false,
                data: None,
                message: "Some users are already lecturers for this module".into(),
            })
        )
    }
}

/// POST /api/modules/:module_id/students
///
/// Assign one or more users as students to a module. Admin only.
///
/// ### Request Body
/// ```json
/// {
///   "user_ids": [1, 2]
/// }
/// ```
///
/// ### Validation Rules
/// - `user_ids`: must be a non-empty list.
/// - All users must exist.
/// - No user may already be assigned as a student.
///
/// ### Responses
///
/// - `200 OK`  
/// ```json
/// {
///   "success": true,
///   "data": {},
///   "message": "Students assigned to module successfully"
/// }
/// ```
///
/// - `400 Bad Request`  
/// ```json
/// {
///   "success": false,
///   "message": "Request must include a non-empty list of user_ids"
/// }
/// ```
///
/// - `403 Forbidden`  
/// ```json
/// {
///   "success": false,
///   "message": "You do not have permission to perform this action"
/// }
/// ```
///
/// - `404 Not Found`  
/// ```json
/// {
///   "success": false,
///   "message": "User with ID 3 does not exist"
/// }
/// ```
///
/// - `409 Conflict`  
/// ```json
/// {
///   "success": false,
///   "message": "Some users are already students for this module"
/// }
/// ```

pub async fn assign_students(axum::extract::Path(module_id): axum::extract::Path<i64>, AuthUser(claims): AuthUser, Json(body): Json<ModifyUsersModuleRequest>, ) -> impl IntoResponse {
    if !claims.admin {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<()> {
                success: false,
                data: None,
                message: "You do not have permission to perform this action".into(),
            }),
        );
    }

    if body.user_ids.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()> {
                success: false,
                data: None,
                message: "Request must include a non-empty list of user_ids".into(),
            }),
        );
    }

    let pool = pool::get();

    let module_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM modules WHERE id = ?)"
    )
        .bind(module_id)
        .fetch_one(pool)
        .await
        .unwrap_or(false);

    if !module_exists {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()> {
                success: false,
                data: None,
                message: "Module not found".into(),
            }),
        );
    }

    let mut already_assigned = Vec::new();
    for &user_id in &body.user_ids {
        let user_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM users WHERE id = ?)"
        )
            .bind(user_id)
            .fetch_one(pool)
            .await
            .unwrap_or(false);

        if !user_exists {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<()> {
                    success: false,
                    data: None,
                    message: format!("User with ID {} does not exist", user_id),
                }),
            );
        }

        let is_already: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM module_students WHERE module_id = ? AND user_id = ?)"
        ).bind(module_id).bind(user_id).fetch_one(pool).await.unwrap_or(false);

        if is_already {
            already_assigned.push(user_id);
        } else {
            let _ = sqlx::query(
                "INSERT OR IGNORE INTO module_students (module_id, user_id) VALUES (?, ?)"
            ).bind(module_id).bind(user_id).execute(pool).await;
        }
    }

    if already_assigned.is_empty() {
        (
            StatusCode::OK,
            Json(ApiResponse::<()> {
                success: true,
                data: None,
                message: "Students assigned to module successfully".into(),
            })
        )
    } else {
        (
            StatusCode::CONFLICT,
            Json(ApiResponse::<()> {
                success: false,
                data: None,
                message: "Some users are already lecturers for this module".into(),
            })
        )
    }
}


/// POST /api/modules/:module_id/tutors
///
/// Assign one or more users as tutors to a module. Admin only.
///
/// ### Request Body
/// ```json
/// {
///   "user_ids": [1, 2]
/// }
/// ```
///
/// ### Validation Rules
/// - `user_ids`: must be a non-empty list.
/// - All users must exist.
/// - Each user must not already be a tutor in this module.
///
/// ### Responses
///
/// - `200 OK`  
/// ```json
/// {
///   "success": true,
///   "data": {},
///   "message": "Tutors assigned to module successfully"
/// }
/// ```
///
/// - `400 Bad Request`  
/// ```json
/// {
///   "success": false,
///   "message": "Request must include a non-empty list of user_ids"
/// }
/// ```
///
/// - `403 Forbidden`  
/// ```json
/// {
///   "success": false,
///   "message": "You do not have permission to perform this action"
/// }
/// ```
///
/// - `404 Not Found`  
/// ```json
/// {
///   "success": false,
///   "message": "User with ID 3 does not exist"
/// }
/// ```
///
/// - `409 Conflict`  
/// ```json
/// {
///   "success": false,
///   "message": "Some users are already tutors for this module"
/// }
/// ```

pub async fn assign_tutors(axum::extract::Path(module_id): axum::extract::Path<i64>, AuthUser(claims): AuthUser, Json(body): Json<ModifyUsersModuleRequest>, ) -> impl axum::response::IntoResponse {
    if !claims.admin {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<()>::error("You do not have permission to perform this action")),
        );
    }

    if body.user_ids.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error("Request must include a non-empty list of user_ids")),
        );
    }

    let pool = db::pool::get();

    let module_exists = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM modules WHERE id = ?)")
        .bind(module_id)
        .fetch_one(pool)
        .await
        .unwrap_or(false);

    if !module_exists {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Module not found")),
        );
    }

    let mut already_assigned = Vec::new();

    for &user_id in &body.user_ids {
        let user_exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE id = ?)")
                .bind(user_id)
                .fetch_one(pool)
                .await
                .unwrap_or(false);

        if !user_exists {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<()>::error(format!("User with ID {} does not exist", user_id))),
            );
        }

        let is_already = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM module_tutors WHERE module_id = ? AND user_id = ?)",
        )
            .bind(module_id)
            .bind(user_id)
            .fetch_one(pool)
            .await
            .unwrap_or(false);

        if is_already {
            already_assigned.push(user_id);
        } else {
            let _ = sqlx::query(
                "INSERT OR IGNORE INTO module_tutors (module_id, user_id) VALUES (?, ?)",
            )
                .bind(module_id)
                .bind(user_id)
                .execute(pool)
                .await;
        }
    }

    if already_assigned.is_empty() {
        (
            StatusCode::OK,
            Json(ApiResponse::<()>::success((), "Tutors assigned to module successfully")),
        )
    } else {
        (
            StatusCode::CONFLICT,
            Json(ApiResponse::<()> {
                success: false,
                data: None,
                message: "Some users are already lecturers for this module".into(),
            })
        )
    }
}
