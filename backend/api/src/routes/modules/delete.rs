use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use sea_orm::{ColumnTrait, Condition, EntityTrait, ModelTrait, QueryFilter};

use crate::{
    auth::AuthUser,
    response::ApiResponse,
    routes::modules::post::ModifyUsersModuleRequest,
};

use db::{
    connect,
    models::{
        module,
        user,
        user_module_role::{self, Column as RoleCol, Role},
    },
};

/// DELETE /api/modules/:module_id
///
/// Permanently deletes a module by ID, including all its assignments and assignment files.  
/// Only accessible by admin users.
///
/// ### Responses
///
/// - `200 OK`  
/// ```json
/// {
///   "success": true,
///   "data": null,
///   "message": "Module deleted successfully"
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
///   "message": "Module not found"
/// }
/// ```
pub async fn delete_module(Path(module_id): Path<i32>) -> impl IntoResponse {
    let db = connect().await;

    match module::Entity::find()
        .filter(module::Column::Id.eq(module_id))
        .one(&db)
        .await
    {
        Ok(Some(m)) => {
            match m.delete(&db).await {
                Ok(_) => (
                    StatusCode::OK,
                    Json(ApiResponse::<()>::success((), "Module deleted successfully")),
                ),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()>::error(format!("Failed to delete module: {}", e))),
                ),
            }
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Module not found")),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Database error: {}", e))),
        ),
    }
}

/// DELETE /api/modules/:module_id/lecturers
///
/// Remove one or more users from the list of lecturers assigned to a module.  
/// Only accessible by admin users.
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
/// - All users must currently be assigned as lecturers to the module.
///
/// ### Responses
///
/// - `200 OK`  
/// ```json
/// {
///   "success": true,
///   "data": null,
///   "message": "Lecturers removed from module successfully"
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
///   "message": "User with ID 2 does not exist"
/// }
/// ```
///
/// - `409 Conflict`  
/// ```json
/// {
///   "success": false,
///   "data": null,
///   "message": "Some users are not lecturers for this module"
/// }
/// ```
pub async fn remove_lecturers(
    Path(module_id): Path<i64>,
    AuthUser(claims): AuthUser,
    Json(body): Json<ModifyUsersModuleRequest>,
) -> impl IntoResponse {
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

    let db = connect().await;

    // Verify module exists
    let module_exists = module::Entity::find_by_id(module_id)
        .one(&db)
        .await
        .map(|opt| opt.is_some())
        .unwrap_or(false);

    if !module_exists {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Module not found")),
        );
    }

    let mut not_assigned = Vec::new();

    for &user_id in &body.user_ids {
        // Verify user exists
        let user_exists = user::Entity::find_by_id(user_id)
            .one(&db)
            .await
            .map(|opt| opt.is_some())
            .unwrap_or(false);

        if !user_exists {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<()>::error(&format!("User with ID {} does not exist", user_id))),
            );
        }

        // Attempt to delete the lecturer role for the user from the module
        let deletion = user_module_role::Model::remove_user_from_module(&db, user_id, module_id)
            .await;

        match deletion {
            Ok(res) if res.rows_affected == 0 => {
                not_assigned.push(user_id);
            }
            Ok(_) => {}
            Err(_) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()>::error("Error while removing lecturer role")),
                );
            }
        }
    }

    if not_assigned.is_empty() {
        (
            StatusCode::OK,
            Json(ApiResponse::<()>::success((), "Lecturers removed from module successfully")),
        )
    } else {
        (
            StatusCode::CONFLICT,
            Json(ApiResponse::<()>::error("Some users are not lecturers for this module")),
        )
    }
}

/// DELETE /api/modules/:module_id/students
///
/// Remove one or more users from a module's student list.  
/// Only accessible by admin users.
///
/// ### Request Body
/// ```json
/// {
///   "user_ids": [3, 4]
/// }
/// ```
///
/// ### Validation Rules
/// - `user_ids`: must be a non-empty list of valid user IDs.
/// - Each user must currently be enrolled as a student in the specified module.
///
/// ### Responses
///
/// - `200 OK`  
/// ```json
/// {
///   "success": true,
///   "data": null,
///   "message": "Students removed from module successfully"
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
///   "message": "Module not found"
/// }
/// ```
///
/// - `409 Conflict`  
/// ```json
/// {
///   "success": false,
///   "data": null,
///   "message": "Some users are not students for this module"
/// }
/// ```
pub async fn remove_students(
    Path(module_id): Path<i32>,
    AuthUser(claims): AuthUser,
    Json(body): Json<ModifyUsersModuleRequest>,
) -> impl IntoResponse {
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

    let db = connect().await;

    // Check if module exists
    let module_exists = module::Entity::find_by_id(module_id)
        .one(&db)
        .await;

    if let Ok(None) | Err(_) = module_exists {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Module not found")),
        );
    }

    let mut not_assigned = Vec::new();

    for &user_id in &body.user_ids {
        // Ensure user exists
        let user_exists = user::Entity::find_by_id(user_id)
            .one(&db)
            .await;

        if let Ok(None) | Err(_) = user_exists {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<()>::error(&format!("User with ID {} does not exist", user_id))),
            );
        }

        // Check if user is a student for the module
        let role_entry = user_module_role::Entity::find()
            .filter(
                Condition::all()
                    .add(RoleCol::UserId.eq(user_id))
                    .add(RoleCol::ModuleId.eq(module_id))
                    .add(RoleCol::Role.eq(Role::Student)),
            )
            .one(&db)
            .await;

        match role_entry {
            Ok(Some(role)) => {
                let _ = role.delete(&db).await;
            }
            Ok(None) => not_assigned.push(user_id),
            Err(_) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()>::error("Database error during role check")),
                );
            }
        }
    }

    if not_assigned.is_empty() {
        (
            StatusCode::OK,
            Json(ApiResponse::<()>::success((), "Students removed from module successfully")),
        )
    } else {
        (
            StatusCode::CONFLICT,
            Json(ApiResponse::<()>::error("Some users are not students for this module")),
        )
    }
}

/// DELETE /api/modules/:module_id/tutors
///
/// Remove one or more users from the tutor list of a module.  
/// Only accessible by admin users.
///
/// ### Request Body
/// ```json
/// {
///   "user_ids": [5, 6]
/// }
/// ```
///
/// ### Validation Rules
/// - `user_ids`: must be a non-empty list of valid user IDs.
/// - Users must already be assigned as tutors for the specified module.
///
/// ### Responses
///
/// - `200 OK`  
/// ```json
/// {
///   "success": true,
///   "data": null,
///   "message": "Tutors removed from module successfully"
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
///   "message": "Module not found"
/// }
/// ```
///
/// - `409 Conflict`  
/// ```json
/// {
///   "success": false,
///   "data": null,
///   "message": "Some users are not tutors for this module"
/// }
/// ```
pub async fn remove_tutors(
    Path(module_id): Path<i32>,
    AuthUser(claims): AuthUser,
    Json(body): Json<ModifyUsersModuleRequest>,
) -> impl IntoResponse {
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

    let db = connect().await;

    let module_exists = module::Entity::find_by_id(module_id)
        .one(&db)
        .await;

    if let Ok(None) | Err(_) = module_exists {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Module not found")),
        );
    }

    let mut not_assigned = Vec::new();

    for &user_id in &body.user_ids {
        let user_exists = user::Entity::find_by_id(user_id)
            .one(&db)
            .await;

        if let Ok(None) | Err(_) = user_exists {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<()>::error(&format!("User with ID {} does not exist", user_id))),
            );
        }

        let role_entry = user_module_role::Entity::find()
            .filter(
                Condition::all()
                    .add(RoleCol::UserId.eq(user_id))
                    .add(RoleCol::ModuleId.eq(module_id))
                    .add(RoleCol::Role.eq(Role::Tutor)),
            )
            .one(&db)
            .await;

        match role_entry {
            Ok(Some(model)) => {
                if model.delete(&db).await.is_err() {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiResponse::<()>::error("Failed to remove tutor role")),
                    );
                }
            }
            Ok(None) => not_assigned.push(user_id),
            Err(_) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()>::error("Database error during role check")),
                );
            }
        }
    }

    if not_assigned.is_empty() {
        (
            StatusCode::OK,
            Json(ApiResponse::<()>::success((), "Tutors removed from module successfully")),
        )
    } else {
        (
            StatusCode::CONFLICT,
            Json(ApiResponse::<()>::error("Some users are not tutors for this module")),
        )
    }
}