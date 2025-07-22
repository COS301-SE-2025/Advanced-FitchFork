use axum::{
    extract::{State, Path},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use crate::auth::AuthUser;
use crate::response::ApiResponse;
use db::models::{
    user,
    user_module_role::{self, Column as RoleCol, Role},
};
use sea_orm::{EntityTrait, QueryFilter, Condition, ColumnTrait, DatabaseConnection, ActiveModelTrait};
use crate::routes::modules::common::ModifyUsersModuleRequest;

/// DELETE /api/modules/{module_id}/tutors
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
    State(db): State<DatabaseConnection>,
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
                if user_module_role::ActiveModel::from(model).delete(&db).await.is_err() {
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