use axum::{
    extract::{Path, Extension},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use crate::{
    auth::AuthUser,
    response::{ApiResponse},
};
use util::filters::FilterParam;
use services::{service::Service, user_module_role::{CreateUserModuleRole, UpdateUserModuleRole}};
use services::user::UserService;
use services::user_module_role::UserModuleRoleService;

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

        match UserModuleRoleService::find_one(
            &vec![
                FilterParam::eq("user_id", user_id),
                FilterParam::eq("module_id", module_id),
                FilterParam::eq("role", "lecturer".to_string()),
            ],
            &vec![],
            None,
        ).await {
            Ok(Some(_)) => {}
            Ok(None) | Err(_) => {
                return (
                    StatusCode::FORBIDDEN,
                    Json(ApiResponse::<()>::error("Lecturer access required for this module")),
                );
            }
        }
    }

    // === PROCESS ASSIGNMENTS ===
    for &target_user_id in &body.user_ids {
        match UserService::find_by_id(target_user_id).await {
            Ok(Some(_)) => {}
            Ok(None) => {
                return (
                    StatusCode::NOT_FOUND,
                    Json(ApiResponse::<()>::error(&format!("User with ID {} does not exist", target_user_id))),
                );
            }
            Err(_) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()>::error("Database error while checking user existence")),
                );
            }
        }

        match UserModuleRoleService::find_one(
            &vec![
                FilterParam::eq("user_id", target_user_id),
                FilterParam::eq("module_id", module_id),
            ],
            &vec![],
            None,
        ).await {
            Ok(Some(existing)) => {
                if existing.role.to_string() != *assigning_role {
                    match UserModuleRoleService::update(
                        UpdateUserModuleRole {
                            user_id: target_user_id,
                            module_id: module_id,
                            role: Some(assigning_role.clone()),
                        }
                    ).await {
                        Ok(_) => {},
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
                match UserModuleRoleService::create(
                    CreateUserModuleRole {
                        user_id: target_user_id,
                        module_id: module_id,
                        role: assigning_role.clone(),
                    }
                ).await {
                    Ok(_) => {},
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
