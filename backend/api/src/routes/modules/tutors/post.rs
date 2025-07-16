use axum::{
    http::StatusCode,
    Json,
};
use crate::response::ApiResponse;
use db::{
    connect,
    models::{
        user::{Entity as UserEntity},
        module::{Entity as ModuleEntity},
        user_module_role::{Entity as RoleEntity, ActiveModel as RoleActiveModel, Column as RoleCol, Role},
    },
};
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait, ActiveModelTrait, Condition, Set};
use crate::routes::modules::common::ModifyUsersModuleRequest;

/// POST /api/modules/{module_id}/tutors
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
    if body.user_ids.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error("Request must include a non-empty list of user_ids")),
        );
    }

    let db = connect().await;

    let module = ModuleEntity::find_by_id(module_id).one(&db).await;
    if let Ok(None) | Err(_) = module {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Module not found")),
        );
    }

    let mut already_assigned = Vec::new();

    for &user_id in &body.user_ids {
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