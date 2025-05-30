use axum::{
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use chrono::{Datelike, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

use db::{
    connect,
    models::{
        module::{Entity as ModuleEntity, Model as Module},
        user::Entity as UserEntity,
        user_module_role::{
            ActiveModel as RoleActiveModel,
            Column as RoleCol,
            Entity as RoleEntity,
            Role,
        },
    },
};

use crate::response::ApiResponse;

lazy_static::lazy_static! {
    static ref MODULE_CODE_REGEX: regex::Regex = regex::Regex::new("^[A-Z]{3}\\d{3}$").unwrap();
}

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
    pub already_assigned: Vec<i32>,
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

impl From<db::models::user::Model> for PersonnelResponse {
    fn from(user: db::models::user::Model) -> Self {
        Self {
            id: user.id,
            student_number: user.student_number,
            email: user.email,
            admin: user.admin,
            created_at: user.created_at.to_rfc3339(),
            updated_at: user.updated_at.to_rfc3339(),
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
            created_at: module.created_at.to_rfc3339(),
            updated_at: module.updated_at.to_rfc3339(),
        }
    }
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

    let db = connect().await;

    match Module::create(
        &db,
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
            if let sea_orm::DbErr::Exec(err) = &e {
                if err.to_string().contains("UNIQUE constraint failed: modules.code") {
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
///   "data": null,
///   "message": "Lecturers assigned to module successfully"
/// }
/// ```
///
/// - `400 Bad Request`  
/// ```json
/// {
///   "success": false,
///   "data": null,
///   "message": "Request must include a non-empty list of user_ids"
/// }
/// ```
///
/// - `403 Forbidden`  
/// ```json
/// {
///   "success": false,
///   "data": null,
///   "message": "You do not have permission to perform this action"
/// }
/// ```
///
/// - `404 Not Found`  
/// ```json
/// {
///   "success": false,
///   "data": null,
///   "message": "User with ID 3 does not exist"
/// }
/// ```
///
/// - `409 Conflict`  
/// ```json
/// {
///   "success": false,
///   "data": null,
///   "message": "Some users are already lecturers for this module"
/// }
/// ```
pub async fn assign_lecturers(
    axum::extract::Path(module_id): axum::extract::Path<i64>,
    Json(body): Json<ModifyUsersModuleRequest>,
) -> impl IntoResponse {
    use sea_orm::{ColumnTrait, Condition, EntityTrait, QueryFilter, Set, ActiveModelTrait};

    if body.user_ids.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error("Request must include a non-empty list of user_ids")),
        );
    }

    let db = connect().await;

    // Check if module exists
    let module = ModuleEntity::find_by_id(module_id).one(&db).await;
    if let Ok(None) | Err(_) = module {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Module not found")),
        );
    }

    let mut already_assigned = Vec::new();

    for &user_id in &body.user_ids {
        // Check if user exists
        match UserEntity::find_by_id(user_id).one(&db).await {
            Ok(Some(_)) => {}
            Ok(None) => {
                return (
                    StatusCode::NOT_FOUND,
                    Json(ApiResponse::<()>::error(&format!(
                        "User with ID {} does not exist",
                        user_id
                    ))),
                );
            }
            Err(_) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()>::error("Database error while checking user")),
                );
            }
        }

        // Check if already assigned as lecturer
        let exists = RoleEntity::find()
            .filter(
                Condition::all()
                    .add(RoleCol::UserId.eq(user_id))
                    .add(RoleCol::ModuleId.eq(module_id))
                    .add(RoleCol::Role.eq(Role::Lecturer)),
            )
            .one(&db)
            .await;

        match exists {
            Ok(Some(_)) => {
                already_assigned.push(user_id);
                continue;
            }
            Ok(None) => {
                // Insert assignment
                let new_role = RoleActiveModel {
                    user_id: Set(user_id),
                    module_id: Set(module_id),
                    role: Set(Role::Lecturer),
                    ..Default::default()
                };

                if let Err(_) = new_role.insert(&db).await {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiResponse::<()>::error("Failed to assign lecturer")),
                    );
                }
            }
            Err(_) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()>::error("Failed to query role assignment")),
                );
            }
        }
    }

    if already_assigned.is_empty() {
        (
            StatusCode::OK,
            Json(ApiResponse::<()>::success((), "Lecturers assigned to module successfully")),
        )
    } else {
        (
            StatusCode::CONFLICT,
            Json(ApiResponse::<()>::error("Some users are already lecturers for this module")),
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
/// - `user_ids`: must be a non-empty list of valid user IDs.
/// - All users must exist.
/// - Each user must not already be assigned as a student.
///
/// ### Responses
///
/// - `200 OK`  
/// ```json
/// {
///   "success": true,
///   "data": null,
///   "message": "Students assigned to module successfully"
/// }
/// ```
///
/// - `400 Bad Request`  
/// ```json
/// {
///   "success": false,
///   "data": null,
///   "message": "Request must include a non-empty list of user_ids"
/// }
/// ```
///
/// - `403 Forbidden`  
/// ```json
/// {
///   "success": false,
///   "data": null,
///   "message": "You do not have permission to perform this action"
/// }
/// ```
///
/// - `404 Not Found`  
/// ```json
/// {
///   "success": false,
///   "data": null,
///   "message": "User with ID 3 does not exist"
/// }
/// ```
///
/// - `409 Conflict`  
/// ```json
/// {
///   "success": false,
///   "data": null,
///   "message": "Some users are already students for this module"
/// }
/// ```
pub async fn assign_students(
    axum::extract::Path(module_id): axum::extract::Path<i64>,
    Json(body): Json<ModifyUsersModuleRequest>,
) -> impl IntoResponse {
    use sea_orm::{ColumnTrait, Condition, EntityTrait, QueryFilter, Set, ActiveModelTrait};

    if body.user_ids.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error("Request must include a non-empty list of user_ids")),
        );
    }

    let db = connect().await;

    // Check if module exists
    let module = ModuleEntity::find_by_id(module_id).one(&db).await;
    if let Ok(None) | Err(_) = module {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Module not found")),
        );
    }

    let mut already_assigned = Vec::new();

    for &user_id in &body.user_ids {
        // Check if user exists
        match UserEntity::find_by_id(user_id).one(&db).await {
            Ok(Some(_)) => {}
            Ok(None) => {
                return (
                    StatusCode::NOT_FOUND,
                    Json(ApiResponse::<()>::error(&format!(
                        "User with ID {} does not exist",
                        user_id
                    ))),
                );
            }
            Err(_) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()>::error("Database error while checking user")),
                );
            }
        }

        // Check if already assigned as student
        let exists = RoleEntity::find()
            .filter(
                Condition::all()
                    .add(RoleCol::UserId.eq(user_id))
                    .add(RoleCol::ModuleId.eq(module_id))
                    .add(RoleCol::Role.eq(Role::Student)),
            )
            .one(&db)
            .await;

        match exists {
            Ok(Some(_)) => {
                already_assigned.push(user_id);
                continue;
            }
            Ok(None) => {
                // Insert assignment
                let new_role = RoleActiveModel {
                    user_id: Set(user_id),
                    module_id: Set(module_id),
                    role: Set(Role::Student),
                    ..Default::default()
                };

                if let Err(_) = new_role.insert(&db).await {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiResponse::<()>::error("Failed to assign student")),
                    );
                }
            }
            Err(_) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()>::error("Failed to query role assignment")),
                );
            }
        }
    }

    if already_assigned.is_empty() {
        (
            StatusCode::OK,
            Json(ApiResponse::<()>::success((), "Students assigned to module successfully")),
        )
    } else {
        (
            StatusCode::CONFLICT,
            Json(ApiResponse::<()>::error("Some users are already students for this module")),
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
/// - `user_ids`: must be a non-empty list of valid user IDs.
/// - All users must exist.
/// - Each user must not already be assigned as a tutor.
///
/// ### Responses
///
/// - `200 OK`  
/// ```json
/// {
///   "success": true,
///   "data": null,
///   "message": "Tutors assigned to module successfully"
/// }
/// ```
///
/// - `400 Bad Request`  
/// ```json
/// {
///   "success": false,
///   "data": null,
///   "message": "Request must include a non-empty list of user_ids"
/// }
/// ```
///
/// - `403 Forbidden`  
/// ```json
/// {
///   "success": false,
///   "data": null,
///   "message": "You do not have permission to perform this action"
/// }
/// ```
///
/// - `404 Not Found`  
/// ```json
/// {
///   "success": false,
///   "data": null,
///   "message": "User with ID 3 does not exist"
/// }
/// ```
///
/// - `409 Conflict`  
/// ```json
/// {
///   "success": false,
///   "data": null,
///   "message": "Some users are already tutors for this module"
/// }
/// ```
pub async fn assign_tutors(
    axum::extract::Path(module_id): axum::extract::Path<i64>,
    Json(body): Json<ModifyUsersModuleRequest>,
) -> impl axum::response::IntoResponse {
    use sea_orm::{ColumnTrait, Condition, EntityTrait, QueryFilter, Set, ActiveModelTrait};

    if body.user_ids.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error("Request must include a non-empty list of user_ids")),
        );
    }

    let db = connect().await;

    // Check if module exists
    let module = ModuleEntity::find_by_id(module_id).one(&db).await;
    if let Ok(None) | Err(_) = module {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Module not found")),
        );
    }

    let mut already_assigned = Vec::new();

    for &user_id in &body.user_ids {
        // Check if user exists
        match UserEntity::find_by_id(user_id).one(&db).await {
            Ok(Some(_)) => {}
            Ok(None) => {
                return (
                    StatusCode::NOT_FOUND,
                    Json(ApiResponse::<()>::error(&format!(
                        "User with ID {} does not exist",
                        user_id
                    ))),
                );
            }
            Err(_) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()>::error("Database error while checking user")),
                );
            }
        }

        // Check if already assigned as tutor
        let exists = RoleEntity::find()
            .filter(
                Condition::all()
                    .add(RoleCol::UserId.eq(user_id))
                    .add(RoleCol::ModuleId.eq(module_id))
                    .add(RoleCol::Role.eq(Role::Tutor)),
            )
            .one(&db)
            .await;

        match exists {
            Ok(Some(_)) => {
                already_assigned.push(user_id);
                continue;
            }
            Ok(None) => {
                // Insert new tutor role
                let new_role = RoleActiveModel {
                    user_id: Set(user_id),
                    module_id: Set(module_id),
                    role: Set(Role::Tutor),
                    ..Default::default()
                };

                if let Err(_) = new_role.insert(&db).await {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiResponse::<()>::error("Failed to assign tutor")),
                    );
                }
            }
            Err(_) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()>::error("Failed to query tutor assignment")),
                );
            }
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
            Json(ApiResponse::<()>::error("Some users are already tutors for this module")),
        )
    }
}
