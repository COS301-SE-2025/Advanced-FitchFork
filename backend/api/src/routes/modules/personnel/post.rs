use axum::{
    extract::{Path, State, Extension},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use sea_orm::{
    ColumnTrait, Condition, EntityTrait, QueryFilter, Set, ActiveModelTrait,
    IntoActiveModel,
};

use db::models::{
    user::Entity as UserEntity,
    user_module_role::{Entity as RoleEntity, ActiveModel as RoleActiveModel, Column as RoleCol, Role},
};
use util::state::AppState;
use crate::{
    auth::AuthUser,
    response::{ApiResponse},
};

/// Request body for assigning or updating users in a module with a role
#[derive(Debug, Deserialize)]
pub struct AssignPersonnelRequest {
    pub user_ids: Vec<i64>,
    pub role: Role,
}

/// POST /api/modules/{module_id}/personnel
///
/// Assign or update users to a module with a specific role.
///
/// ### Permissions:
/// - Admins can assign **any** role.
/// - Lecturers can assign **student**, **tutor**, and **assistant_lecturer** roles **within modules they teach**.
/// - Lecturers **cannot** assign **lecturer** roles.
/// - Tutors, students, and assistant lecturers are not allowed to use this endpoint.
///
/// ---
///
/// ### Request Body (JSON)
/// ```json
/// {
///   "user_ids": [1, 2, 3],
///   "role": "tutor"
/// }
/// ```
/// - `user_ids`: list of user IDs to assign. Must not be empty.
/// - `role`: one of `"student"`, `"tutor"`, `"assistant_lecturer"`, `"lecturer"`
///
/// ---
///
/// ### Success Response (200 OK)
/// ```json
/// {
///   "success": true,
///   "message": "Users assigned/updated successfully",
///   "data": null
/// }
/// ```
///
/// ---
///
/// ### Error Responses
///
/// **400 Bad Request**
/// ```json
/// {
///   "success": false,
///   "message": "user_ids list cannot be empty"
/// }
/// ```
///
/// **403 Forbidden**
/// - Admin-only role violation:
/// ```json
/// {
///   "success": false,
///   "message": "Only admins can assign lecturers"
/// }
/// ```
/// - Lecturer access required:
/// ```json
/// {
///   "success": false,
///   "message": "Lecturer access required for this module"
/// }
/// ```
///
/// **404 Not Found**
/// ```json
/// {
///   "success": false,
///   "message": "User with ID 42 does not exist"
/// }
/// ```
///
/// **500 Internal Server Error**
/// ```json
/// {
///   "success": false,
///   "message": "Failed to assign role"
/// }
/// ```
pub async fn assign_personnel(
    State(app_state): State<AppState>,
    Path(module_id): Path<i64>,
    Extension(AuthUser(claims)): Extension<AuthUser>,
    Json(body): Json<AssignPersonnelRequest>,
) -> impl IntoResponse {
    let db = app_state.db();
    let user_id = claims.sub;

    if body.user_ids.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error("user_ids list cannot be empty")),
        );
    }

    // === PERMISSION ENFORCEMENT ===
    let assigning_role = &body.role;

    if !claims.admin {
        // Only admins can assign lecturers
        if *assigning_role == Role::Lecturer {
            return (
                StatusCode::FORBIDDEN,
                Json(ApiResponse::<()>::error("Only admins can assign lecturers")),
            );
        }

        // Check if current user is a lecturer for the module
        let is_module_lecturer = RoleEntity::find()
            .filter(
                Condition::all()
                    .add(RoleCol::UserId.eq(user_id))
                    .add(RoleCol::ModuleId.eq(module_id))
                    .add(RoleCol::Role.eq(Role::Lecturer)),
            )
            .one(db)
            .await
            .map(|res| res.is_some())
            .unwrap_or(false);

        if !is_module_lecturer {
            return (
                StatusCode::FORBIDDEN,
                Json(ApiResponse::<()>::error("Lecturer access required for this module")),
            );
        }
    }

    // === PROCESS ASSIGNMENTS ===
    for &target_user_id in &body.user_ids {
        let user_exists = UserEntity::find_by_id(target_user_id)
            .one(db)
            .await
            .map(|opt| opt.is_some())
            .unwrap_or(false);

        if !user_exists {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<()>::error(&format!("User with ID {} does not exist", target_user_id))),
            );
        }

        let existing_role = RoleEntity::find()
            .filter(
                Condition::all()
                    .add(RoleCol::UserId.eq(target_user_id))
                    .add(RoleCol::ModuleId.eq(module_id)),
            )
            .one(db)
            .await;

        match existing_role {
            Ok(Some(existing)) => {
                if existing.role != *assigning_role {
                    let mut active = existing.into_active_model();
                    active.role = Set(assigning_role.clone());

                    if let Err(_) = active.update(db).await {
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(ApiResponse::<()>::error("Failed to update role")),
                        );
                    }
                }
            }
            Ok(None) => {
                let new_role = RoleActiveModel {
                    user_id: Set(target_user_id),
                    module_id: Set(module_id),
                    role: Set(assigning_role.clone()),
                };

                if let Err(_) = new_role.insert(db).await {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiResponse::<()>::error("Failed to assign role")),
                    );
                }
            }
            Err(_) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()>::error("Database error while checking existing assignment")),
                );
            }
        }
    }

    (
        StatusCode::OK,
        Json(ApiResponse::success((), "Users assigned/updated successfully")),
    )
}
