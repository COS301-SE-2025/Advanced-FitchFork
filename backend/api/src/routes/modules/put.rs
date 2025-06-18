use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use validator::Validate;

use sea_orm::{
    ActiveModelTrait,
    ColumnTrait,
    Condition,
    DatabaseConnection,
    EntityTrait,
    QueryFilter,
    Set,
    TransactionTrait,
    IntoActiveModel,
};

use db::{
    connect,
    models::{
        module::{
            ActiveModel as ModuleActiveModel,
            Column as ModuleCol,
            Entity as ModuleEntity,
            Model as Module,
        },
        user::{Entity as UserEntity},
        user_module_role::{
            ActiveModel as RoleActiveModel,
            Column as RoleCol,
            Entity as RoleEntity,
            Role,
        },
    },
};

use crate::response::ApiResponse;

#[derive(Debug, Deserialize, Validate)]
pub struct EditModuleRequest {
    #[validate(regex(
        path = "MODULE_CODE_REGEX",
        message = "Module code must be in format ABC123"
    ))]
    pub code: String,

    pub year: i32,

    #[validate(length(max = 1000, message = "Description must be at most 1000 characters"))]
    pub description: String,

    #[validate(range(min = 1, message = "Credits must be a positive number"))]
    pub credits: i32,
}

#[derive(Debug, Serialize)]
struct ModuleResponse {
    id: i64,
    code: String,
    year: i32,
    description: String,
    credits: i32,
    created_at: String,
    updated_at: String,
}

impl From<Module> for ModuleResponse {
    fn from(module: Module) -> Self {
        Self {
            id: module.id,
            code: module.code,
            year: module.year,
            description: module.description.unwrap_or_default(),
            credits: module.credits,
            created_at: module.created_at.to_rfc3339(),
            updated_at: module.updated_at.to_rfc3339(),
        }
    }
}

lazy_static::lazy_static! {
    static ref MODULE_CODE_REGEX: regex::Regex = regex::Regex::new("^[A-Z]{3}\\d{3}$").unwrap();
}

/// Updates the details of a specific module by its ID.
///
/// # Arguments
///
/// Arguments are extracted from:
/// - `Path(module_id)`: The ID of the module to be updated (from the URL path).
/// - `Json(req)`: A JSON payload containing the updated fields:
///   - `code` (string, required): The new module code.
///   - `year` (integer, required): The academic year for the module.
///   - `description` (string, required): The updated module description.
///   - `credits` (integer, required): The number of credits for the module.
///
/// # Returns
///
/// Returns an HTTP response indicating the result:
/// - `200 OK`: If the module was successfully updated. Returns the updated module data.
/// - `400 BAD REQUEST`: If any required field (`code`, `year`, `description`, or `credits`) is missing or invalid.
/// - `404 NOT FOUND`: If no module exists with the given ID.
/// - `409 CONFLICT`: If the new module code already exists (violating a unique constraint).
/// - `500 INTERNAL SERVER ERROR`: If an unexpected database error occurs.
///
/// # Response Format
///
/// All responses use the `ApiResponse<ModuleResponse>` structure for consistency.
pub async fn edit_module(
    Path(module_id): Path<i64>,
    Json(req): Json<EditModuleRequest>,
) -> impl IntoResponse {
    if let Err(validation_errors) = req.validate() {
        let error_message = common::format_validation_errors(&validation_errors);
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<ModuleResponse>::error(error_message)),
        );
    }

    let db: DatabaseConnection = connect().await;

    // Check if the module exists
    let module = ModuleEntity::find_by_id(module_id).one(&db).await;
    if let Ok(None) | Err(_) = module {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<ModuleResponse>::error("Module not found")),
        );
    }

    // Check if the new code is already in use by another module
    let duplicate = ModuleEntity::find()
        .filter(
            Condition::all()
                .add(ModuleCol::Code.eq(req.code.clone()))
                .add(ModuleCol::Id.ne(module_id)),
        )
        .one(&db)
        .await;

    if let Ok(Some(_)) = duplicate {
        return (
            StatusCode::CONFLICT,
            Json(ApiResponse::<ModuleResponse>::error("Module code already exists")),
        );
    }

    let updated_module = ModuleActiveModel {
        id: Set(module_id),
        code: Set(req.code.clone()),
        year: Set(req.year),
        description: Set(Some(req.description.clone())),
        credits: Set(req.credits),
        updated_at: Set(Utc::now()),
        ..Default::default()
    };

    match updated_module.update(&db).await {
        Ok(module) => (
            StatusCode::OK,
            Json(ApiResponse::success(ModuleResponse::from(module), "Module updated successfully")),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<ModuleResponse>::error("Failed to update module")),
        ),
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct AssignLecturersRequest {
    #[validate(length(min = 1, message = "Request must include a non-empty list of user_ids"))]
    pub user_ids: Vec<i64>,
}

#[derive(Debug, Serialize)]
struct AssignLecturersResponse {
    message: String,
}

/// PUT /api/modules/:module_id/lecturers
///
/// Assign one or more users as lecturers to a module. This endpoint will overwrite
/// existing role assignments for the specified users in this module, setting their
/// role exclusively to Lecturer. Admin only.
///
/// ### Request Body
/// ```json
/// {
///   "user_ids": [1, 2, 3]
/// }
/// ```
///
/// ### Validation Rules
/// - `module_id` must reference an existing module
/// - All `user_ids` must reference existing users
/// - `user_ids` array must be non-empty
/// - Users not previously assigned to the module will be added as Lecturers
/// - Users with existing roles (Student/Tutor) will be converted to Lecturers
///
/// ### Responses
///
/// - `200 OK`  
/// ```json
/// {
///   "success": true,
///   "data": {
///     "message": "Users set as lecturers successfully"
///   },
///   "message": "Users set as lecturers successfully"
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
///   "message": "You do not have permission to modify roles"
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
/// - `500 Internal Server Error`  
/// ```json
/// {
///   "success": false,
///   "data": null,
///   "message": "Failed to update role assignments"
/// }
/// ```
pub async fn edit_lecturers(
    Path(module_id): Path<i64>,
    Json(req): Json<AssignLecturersRequest>,
) -> impl IntoResponse {
    if let Err(validation_errors) = req.validate() {
        let error_message = common::format_validation_errors(&validation_errors);
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<AssignLecturersResponse>::error(error_message)),
        );
    }

    let db: DatabaseConnection = connect().await;

    let module = ModuleEntity::find_by_id(module_id).one(&db).await;
    if let Ok(None) | Err(_) = module {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<AssignLecturersResponse>::error("Module not found")),
        );
    }

    for &user_id in &req.user_ids {
        let user = UserEntity::find_by_id(user_id).one(&db).await;
        if let Ok(None) | Err(_) = user {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<AssignLecturersResponse>::error(&format!(
                    "User with ID {} does not exist",
                    user_id
                ))),
            );
        }
    }

    let transaction = db.begin().await;
    if let Err(_) = transaction {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<AssignLecturersResponse>::error("Failed to start transaction")),
        );
    }
    let transaction = transaction.unwrap();

    for &user_id in &req.user_ids {
        let existing_role = RoleEntity::find()
            .filter(
                Condition::all()
                    .add(RoleCol::UserId.eq(user_id))
                    .add(RoleCol::ModuleId.eq(module_id)),
            )
            .one(&transaction)
            .await;

        match existing_role {
            Ok(Some(existing)) => {
                let mut active_model = existing.into_active_model();
                active_model.role = Set(Role::Lecturer);
                
                if let Err(_) = active_model.update(&transaction).await {
                    if let Err(_) = transaction.rollback().await {

                    }
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiResponse::<AssignLecturersResponse>::error("Failed to update role assignments")),
                    );
                }
            }
            Ok(None) => {
                let new_role = RoleActiveModel {
                    user_id: Set(user_id),
                    module_id: Set(module_id),
                    role: Set(Role::Lecturer),
                    ..Default::default()
                };

                if let Err(_) = new_role.insert(&transaction).await {
                    if let Err(_) = transaction.rollback().await {

                    }
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiResponse::<AssignLecturersResponse>::error("Failed to create role assignments")),
                    );
                }
            }
            Err(_) => {
                if let Err(_) = transaction.rollback().await {

                }
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<AssignLecturersResponse>::error("Failed to query existing roles")),
                );
            }
        }
    }

    if let Err(_) = transaction.commit().await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<AssignLecturersResponse>::error("Failed to commit role assignments")),
        );
    }

    let response = AssignLecturersResponse {
        message: "Users set as lecturers successfully".to_string(),
    };

    (
        StatusCode::OK,
        Json(ApiResponse::success(response, "Users set as lecturers successfully")),
    )
}

#[derive(Debug, Deserialize, Validate)]
pub struct AssignStudentsRequest {
    #[validate(length(min = 1, message = "Request must include a non-empty list of user_ids"))]
    pub user_ids: Vec<i64>,
}

#[derive(Debug, Serialize)]
struct AssignStudentsResponse {
    message: String,
}

/// PUT /api/modules/:module_id/students
///
/// Assign one or more users as students to a module. This endpoint will overwrite
/// existing role assignments for the specified users in this module, setting their
/// role exclusively to Student. Admin only.
///
/// ### Request Body
/// ```json
/// {
///   "user_ids": [1, 2, 3]
/// }
/// ```
///
/// ### Validation Rules
/// - `module_id` must reference an existing module
/// - All `user_ids` must reference existing users
/// - `user_ids` array must be non-empty
/// - Users not previously assigned to the module will be added as Students
/// - Users with existing roles (Lecturer/Tutor) will be converted to Students
///
/// ### Responses
///
/// - `200 OK`  
/// ```json
/// {
///   "success": true,
///   "data": {
///     "message": "Users set as students successfully"
///   },
///   "message": "Users set as students successfully"
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
///   "message": "You do not have permission to modify roles"
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
/// - `500 Internal Server Error`  
/// ```json
/// {
///   "success": false,
///   "data": null,
///   "message": "Failed to update role assignments"
/// }
/// ```
pub async fn edit_students(
    Path(module_id): Path<i64>,
    Json(req): Json<AssignStudentsRequest>,
) -> impl IntoResponse {
    if let Err(validation_errors) = req.validate() {
        let error_message = common::format_validation_errors(&validation_errors);
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<AssignStudentsResponse>::error(error_message)),
        );
    }

    let db: DatabaseConnection = connect().await;

    let module = ModuleEntity::find_by_id(module_id).one(&db).await;
    if let Ok(None) | Err(_) = module {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<AssignStudentsResponse>::error("Module not found")),
        );
    }

    for &user_id in &req.user_ids {
        let user = UserEntity::find_by_id(user_id).one(&db).await;
        if let Ok(None) | Err(_) = user {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<AssignStudentsResponse>::error(&format!(
                    "User with ID {} does not exist",
                    user_id
                ))),
            );
        }
    }

    let transaction = db.begin().await;
    if let Err(_) = transaction {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<AssignStudentsResponse>::error("Failed to start transaction")),
        );
    }
    let transaction = transaction.unwrap();

    for &user_id in &req.user_ids {
        let existing_role = RoleEntity::find()
            .filter(
                Condition::all()
                    .add(RoleCol::UserId.eq(user_id))
                    .add(RoleCol::ModuleId.eq(module_id)),
            )
            .one(&transaction)
            .await;

        match existing_role {
            Ok(Some(existing)) => {
                let mut active_model = existing.into_active_model();
                active_model.role = Set(Role::Student);
                
                if let Err(_) = active_model.update(&transaction).await {
                    if let Err(_) = transaction.rollback().await {

                    }
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiResponse::<AssignStudentsResponse>::error("Failed to update role assignments")),
                    );
                }
            }
            Ok(None) => {
                let new_role = RoleActiveModel {
                    user_id: Set(user_id),
                    module_id: Set(module_id),
                    role: Set(Role::Student),
                    ..Default::default()
                };

                if let Err(_) = new_role.insert(&transaction).await {
                    if let Err(_) = transaction.rollback().await {

                    }
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiResponse::<AssignStudentsResponse>::error("Failed to create role assignments")),
                    );
                }
            }
            Err(_) => {
                if let Err(_) = transaction.rollback().await {

                }
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<AssignStudentsResponse>::error("Failed to query existing roles")),
                );
            }
        }
    }

    if let Err(_) = transaction.commit().await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<AssignStudentsResponse>::error("Failed to commit role assignments")),
        );
    }

    let response = AssignStudentsResponse {
        message: "Users set as students successfully".to_string(),
    };

    (
        StatusCode::OK,
        Json(ApiResponse::success(response, "Users set as students successfully")),
    )
}

#[derive(Debug, Deserialize, Validate)]
pub struct AssignTutorsRequest {
    #[validate(length(min = 1, message = "Request must include a non-empty list of user_ids"))]
    pub user_ids: Vec<i64>,
}

#[derive(Debug, Serialize)]
struct AssignTutorsResponse {
    message: String,
}

/// PUT /api/modules/:module_id/tutors
///
/// Assign one or more users as tutors to a module. This endpoint will overwrite
/// existing role assignments for the specified users in this module, setting their
/// role exclusively to Tutor. Admin only.
///
/// ### Request Body
/// ```json
/// {
///   "user_ids": [1, 2, 3]
/// }
/// ```
///
/// ### Validation Rules
/// - `module_id` must reference an existing module
/// - All `user_ids` must reference existing users
/// - `user_ids` array must be non-empty
/// - Users not previously assigned to the module will be added as Tutors
/// - Users with existing roles (Lecturer/Student) will be converted to Tutors
///
/// ### Responses
///
/// - `200 OK`  
/// ```json
/// {
///   "success": true,
///   "data": {
///     "message": "Users set as tutors successfully"
///   },
///   "message": "Users set as tutors successfully"
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
///   "message": "You do not have permission to modify roles"
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
/// - `500 Internal Server Error`  
/// ```json
/// {
///   "success": false,
///   "data": null,
///   "message": "Failed to update role assignments"
/// }
/// ```
pub async fn edit_tutors(
    Path(module_id): Path<i64>,
    Json(req): Json<AssignTutorsRequest>,
) -> impl IntoResponse {
    if let Err(validation_errors) = req.validate() {
        let error_message = common::format_validation_errors(&validation_errors);
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<AssignTutorsResponse>::error(error_message)),
        );
    }

    let db: DatabaseConnection = connect().await;

    let module = ModuleEntity::find_by_id(module_id).one(&db).await;
    if let Ok(None) | Err(_) = module {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<AssignTutorsResponse>::error("Module not found")),
        );
    }

    for &user_id in &req.user_ids {
        let user = UserEntity::find_by_id(user_id).one(&db).await;
        if let Ok(None) | Err(_) = user {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<AssignTutorsResponse>::error(&format!(
                    "User with ID {} does not exist",
                    user_id
                ))),
            );
        }
    }

    let transaction = db.begin().await;
    if let Err(_) = transaction {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<AssignTutorsResponse>::error("Failed to start transaction")),
        );
    }
    let transaction = transaction.unwrap();

    for &user_id in &req.user_ids {
        let existing_role = RoleEntity::find()
            .filter(
                Condition::all()
                    .add(RoleCol::UserId.eq(user_id))
                    .add(RoleCol::ModuleId.eq(module_id)),
            )
            .one(&transaction)
            .await;

        match existing_role {
            Ok(Some(existing)) => {
                let mut active_model = existing.into_active_model();
                active_model.role = Set(Role::Tutor);
                
                if let Err(_) = active_model.update(&transaction).await {
                    if let Err(_) = transaction.rollback().await {

                    }
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiResponse::<AssignTutorsResponse>::error("Failed to update role assignments")),
                    );
                }
            }
            Ok(None) => {
                let new_role = RoleActiveModel {
                    user_id: Set(user_id),
                    module_id: Set(module_id),
                    role: Set(Role::Tutor),
                    ..Default::default()
                };

                if let Err(_) = new_role.insert(&transaction).await {
                    if let Err(_) = transaction.rollback().await {

                    }
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiResponse::<AssignTutorsResponse>::error("Failed to create role assignments")),
                    );
                }
            }
            Err(_) => {
                if let Err(_) = transaction.rollback().await {

                }
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<AssignTutorsResponse>::error("Failed to query existing roles")),
                );
            }
        }
    }

    if let Err(_) = transaction.commit().await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<AssignTutorsResponse>::error("Failed to commit role assignments")),
        );
    }

    let response = AssignTutorsResponse {
        message: "Users set as tutors successfully".to_string(),
    };

    (
        StatusCode::OK,
        Json(ApiResponse::success(response, "Users set as tutors successfully")),
    )
}