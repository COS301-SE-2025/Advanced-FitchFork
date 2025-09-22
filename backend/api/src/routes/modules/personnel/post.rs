use crate::{auth::AuthUser, response::ApiResponse};
use axum::{
    Json,
    extract::{Extension, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, IntoActiveModel, QueryFilter, Set,
};
use serde::Deserialize;

use crate::{auth::AuthUser, response::ApiResponse};
use db::models::{
    user::Entity as UserEntity,
    user_module_role::{
        ActiveModel as RoleActiveModel, Column as RoleCol, Entity as RoleEntity, Role,
    },
};
use util::state::AppState;

/// Request body for assigning or updating users in a module with a role
#[derive(Debug, Deserialize)]
pub struct AssignPersonnelRequest {
    pub user_ids: Vec<i64>,
    pub role: String,
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
    Path(module_id): Path<i64>,
    Extension(AuthUser(claims)): Extension<AuthUser>,
    Json(body): Json<AssignPersonnelRequest>,
) -> impl IntoResponse {
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
        if *assigning_role == "lecturer".to_string() {
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
                Json(ApiResponse::<()>::error(
                    "Lecturer access required for this module",
                )),
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
                Json(ApiResponse::<()>::error(&format!(
                    "User with ID {} does not exist",
                    target_user_id
                ))),
            );
        }

        match UserModuleRoleService::find_one(
            &vec![
                FilterParam::eq("user_id", target_user_id),
                FilterParam::eq("module_id", module_id),
            ],
            &vec![],
            None,
        )
        .await
        {
            Ok(Some(existing)) => {
                if existing.role.to_string() != *assigning_role {
                    match UserModuleRoleService::update(UpdateUserModuleRole {
                        user_id: target_user_id,
                        module_id: module_id,
                        role: Some(assigning_role.clone()),
                    })
                    .await
                    {
                        Ok(_) => {}
                        Err(_) => {
                            return (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                Json(ApiResponse::<()>::error("Failed to update role")),
                            );
                        }
                    }
                }
            }
            Ok(None) => {
                match UserModuleRoleService::create(CreateUserModuleRole {
                    user_id: target_user_id,
                    module_id: module_id,
                    role: assigning_role.clone(),
                })
                .await
                {
                    Ok(_) => {}
                    Err(_) => {
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(ApiResponse::<()>::error("Failed to assign role")),
                        );
                    }
                }
            }
            Err(_) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()>::error(
                        "Database error while checking existing assignment",
                    )),
                );
            }
        }
    }

    (
        StatusCode::OK,
        Json(ApiResponse::success(
            (),
            "Users assigned/updated successfully",
        )),
    )
}
