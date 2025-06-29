use axum::{extract::Path, http::StatusCode, response::IntoResponse, Json};
use crate::{
    auth::AuthUser,
    response::ApiResponse,
};
use sea_orm::EntityTrait;
use db::{
    connect,
    models::{
        module,
        user,
        user_module_role,
    },
};
use crate::routes::modules::common::ModifyUsersModuleRequest;

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