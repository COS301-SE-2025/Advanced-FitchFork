use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait, DatabaseConnection, ActiveModelTrait, Condition, Set};
use crate::response::ApiResponse;
use db::models::{
    user::{Entity as UserEntity},
    user_module_role::{Entity as RoleEntity, Column as RoleCol, Role, ActiveModel as RoleActiveModel},
};
use crate::routes::modules::common::ModifyUsersModuleRequest;

/// POST /api/modules/{module_id}/students
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
    State(db): State<DatabaseConnection>,
    axum::extract::Path(module_id): axum::extract::Path<i64>,
    Json(body): Json<ModifyUsersModuleRequest>,
) -> impl IntoResponse {
    if body.user_ids.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error("Request must include a non-empty list of user_ids")),
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