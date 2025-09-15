use axum::{
    extract::{Path, Extension},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use crate::{
    auth::AuthUser,
    response::ApiResponse,
};
use util::filters::FilterParam;
use services::service::Service;
use services::user::UserService;
use services::user_module_role::UserModuleRoleService;

#[derive(Debug, Deserialize)]
pub struct RemovePersonnelRequest {
    pub user_ids: Vec<i64>,
    pub role: String,
}

/// DELETE /api/modules/{module_id}/personnel
///
/// Remove one or more users from a module for a specific role.
///
/// Permissions:
/// - Admins can remove users from any role.
/// - Lecturers can remove only `student`, `tutor`, or `assistant_lecturer` roles from modules they are assigned to as lecturers.
/// - Lecturers cannot remove other lecturers.
/// - All other roles are denied access.
///
/// ---
///
/// Request Body (JSON):
/// ```json
/// {
///   "user_ids": [1, 2, 3],
///   "role": "student"
/// }
/// ```
/// - `user_ids`: List of user IDs to unassign. Must not be empty.
/// - `role`: Role to remove. One of: `"student"`, `"tutor"`, `"assistant_lecturer"`, `"lecturer"`
///
/// ---
///
/// Success Response: 200 OK
/// ```json
/// {
///   "success": true,
///   "message": "Users removed from role successfully",
///   "data": null
/// }
/// ```
///
/// ---
///
/// Error Responses:
///
/// 400 Bad Request:
/// ```json
/// {
///   "success": false,
///   "message": "user_ids list cannot be empty"
/// }
/// ```
///
/// 403 Forbidden:
/// ```json
/// {
///   "success": false,
///   "message": "Only admins can remove lecturers"
/// }
/// ```
/// or:
/// ```json
/// {
///   "success": false,
///   "message": "Lecturer access required for this module"
/// }
/// ```
///
/// 404 Not Found:
/// ```json
/// {
///   "success": false,
///   "message": "User with ID 42 does not exist"
/// }
/// ```
///
/// 409 Conflict:
/// ```json
/// {
///   "success": false,
///   "message": "Some users are not assigned with this role"
/// }
/// ```
///
/// 500 Internal Server Error:
/// ```json
/// {
///   "success": false,
///   "message": "Failed to remove role assignment"
/// }
/// ```
pub async fn remove_personnel(
    Path(module_id): Path<i64>,
    Extension(AuthUser(claims)): Extension<AuthUser>,
    Json(body): Json<RemovePersonnelRequest>,
) -> impl IntoResponse {
    let requester_id = claims.sub;

    if body.user_ids.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error("user_ids list cannot be empty")),
        );
    }

    let role_to_remove = &body.role;

    // === Permission Check ===
    if !claims.admin {
        // Disallow lecturers from removing other lecturers
        if *role_to_remove == "lecturer" {
            return (
                StatusCode::FORBIDDEN,
                Json(ApiResponse::<()>::error("Only admins can remove lecturers")),
            );
        }

        match UserModuleRoleService::find_one(
            &vec![
                FilterParam::eq("user_id", requester_id),
                FilterParam::eq("module_id", module_id),
                FilterParam::eq("role", "lecturer".to_string()),
            ],
            &vec![],
            None,
        ).await {
            Ok(Some(_)) => {},
            Ok(None) | Err(_) => {
                return (
                    StatusCode::FORBIDDEN,
                    Json(ApiResponse::<()>::error("Lecturer access required for this module")),
                );
            }
        }
    }

    // === Deletion ===
    let mut not_assigned = Vec::new();

    for &target_user_id in &body.user_ids {
        match UserService::find_by_id(target_user_id).await {
            Ok(Some(_)) => {},
            Ok(None) => {
                return (
                    StatusCode::NOT_FOUND,
                    Json(ApiResponse::<()>::error(&format!("User with ID {} does not exist", target_user_id))),
                );
            },
            Err(_) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()>::error("Failed to query user existence")),
                );
            }
        }

        match UserModuleRoleService::delete(
            &vec![
                FilterParam::eq("user_id", target_user_id),
                FilterParam::eq("module_id", module_id),
                FilterParam::eq("role", role_to_remove.clone()),
            ],
            &vec![],
        ).await {
            Ok(res) if res == 0 => not_assigned.push(target_user_id),
            Ok(_) => {}
            Err(_) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()>::error("Failed to remove role assignment")),
                );
            }
        }
    }

    if not_assigned.is_empty() {
        (
            StatusCode::OK,
            Json(ApiResponse::<()>::success((), "Users removed from role successfully")),
        )
    } else {
        (
            StatusCode::CONFLICT,
            Json(ApiResponse::<()>::error("Some users are not assigned with this role")),
        )
    }
}
