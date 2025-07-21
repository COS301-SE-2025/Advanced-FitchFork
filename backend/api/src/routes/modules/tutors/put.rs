use validator::{Validate};
use axum::{
    extract::{State, Path},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use crate::response::ApiResponse;
use db::{
    models::{
        user::{Entity as UserEntity},
        module::{Entity as ModuleEntity},
        user_module_role::{Entity as RoleEntity, Column as RoleCol, Role},
    },
};
use sea_orm::{EntityTrait, QueryFilter, Condition, ColumnTrait, Set, ActiveModelTrait, DatabaseConnection, TransactionTrait, IntoActiveModel};
use crate::routes::modules::common::EditRoleRequest;

/// PUT /api/modules/{module_id}/tutors
///
/// Update the role of users already assigned to a module to Tutor. This endpoint will overwrite
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
/// - All users must already be assigned to the module (any role)
/// - Users with existing roles (Lecturer/Student) will be converted to Tutors
///
/// ### Responses
///
/// - `200 OK`  
/// ```json
/// {
///   "success": true,
///   "data": null,
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
/// - `400 Bad Request` (user not assigned to module)  
/// ```json
/// {
///   "success": false,
///   "data": null,
///   "message": "User with ID 3 is not assigned to this module"
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
    State(db): State<DatabaseConnection>,
    Path(module_id): Path<i64>,
    Json(req): Json<EditRoleRequest>,
) -> impl IntoResponse {
    if let Err(validation_errors) = req.validate() {
        let error_message = common::format_validation_errors(&validation_errors);
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(error_message)),
        );
    }

    let module = ModuleEntity::find_by_id(module_id).one(&db).await;
    if let Ok(None) | Err(_) = module {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Module not found")),
        );
    }

    for &user_id in &req.user_ids {
        let user = UserEntity::find_by_id(user_id).one(&db).await;
        if let Ok(None) | Err(_) = user {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<()>::error(&format!(
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
            Json(ApiResponse::<()>::error("Failed to start transaction")),
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
                        Json(ApiResponse::<()>::error("Failed to update role assignments")),
                    );
                }
            }
            Ok(None) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::<()>::error(&format!(
                        "User with ID {} is not assigned to this module",
                        user_id
                    ))),
                );
            }
            Err(_) => {
                if let Err(_) = transaction.rollback().await {

                }
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()>::error("Failed to query existing roles")),
                );
            }
        }
    }

    if let Err(_) = transaction.commit().await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error("Failed to commit role assignments")),
        );
    }

    (
        StatusCode::OK,
        Json(ApiResponse::success((), "Users set as tutors successfully")),
    )
}