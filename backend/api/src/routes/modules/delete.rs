use axum::http::StatusCode;
use axum::Json;
use axum::response::IntoResponse;
use db::pool;
use crate::auth::AuthUser;
use crate::response::ApiResponse;
use crate::routes::modules::post::ModifyUsersModuleRequest;



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
    axum::extract::Path(module_id): axum::extract::Path<i64>,
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

    let pool = pool::get();

    let module_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM modules WHERE id = ?)"
    )
    .bind(module_id)
    .fetch_one(pool)
    .await
    .unwrap_or(false);

    if !module_exists {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Module not found")),
        );
    }

    let mut not_assigned = Vec::new();

    for &user_id in &body.user_ids {
        let user_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM users WHERE id = ?)"
        )
        .bind(user_id)
        .fetch_one(pool)
        .await
        .unwrap_or(false);

        if !user_exists {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<()>::error(&format!("User with ID {} does not exist", user_id))),
            );
        }

        let is_assigned: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM module_lecturers WHERE module_id = ? AND user_id = ?)"
        )
        .bind(module_id)
        .bind(user_id)
        .fetch_one(pool)
        .await
        .unwrap_or(false);

        if is_assigned {
            let _ = sqlx::query(
                "DELETE FROM module_lecturers WHERE module_id = ? AND user_id = ?"
            )
            .bind(module_id)
            .bind(user_id)
            .execute(pool)
            .await;
        } else {
            not_assigned.push(user_id);
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
    axum::extract::Path(module_id): axum::extract::Path<i64>,
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

    let pool = pool::get();

    let module_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM modules WHERE id = ?)"
    )
    .bind(module_id)
    .fetch_one(pool)
    .await
    .unwrap_or(false);

    if !module_exists {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Module not found")),
        );
    }

    let mut not_assigned = Vec::new();

    for &user_id in &body.user_ids {
        let assigned: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM module_students WHERE module_id = ? AND user_id = ?)"
        )
        .bind(module_id)
        .bind(user_id)
        .fetch_one(pool)
        .await
        .unwrap_or(false);

        if assigned {
            let _ = sqlx::query(
                "DELETE FROM module_students WHERE module_id = ? AND user_id = ?"
            )
            .bind(module_id)
            .bind(user_id)
            .execute(pool)
            .await;
        } else {
            not_assigned.push(user_id);
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
    axum::extract::Path(module_id): axum::extract::Path<i64>,
    crate::auth::claims::AuthUser(claims): crate::auth::claims::AuthUser,
    axum::Json(body): axum::Json<crate::routes::modules::post::ModifyUsersModuleRequest>,
) -> impl axum::response::IntoResponse {
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

    let pool = db::pool::get();

    let module_exists = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM modules WHERE id = ?)"
    )
    .bind(module_id)
    .fetch_one(pool)
    .await
    .unwrap_or(false);

    if !module_exists {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Module not found")),
        );
    }

    let mut not_assigned = Vec::new();

    for &user_id in &body.user_ids {
        let assigned = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM module_tutors WHERE module_id = ? AND user_id = ?)"
        )
        .bind(module_id)
        .bind(user_id)
        .fetch_one(pool)
        .await
        .unwrap_or(false);

        if assigned {
            let _ = sqlx::query(
                "DELETE FROM module_tutors WHERE module_id = ? AND user_id = ?"
            )
            .bind(module_id)
            .bind(user_id)
            .execute(pool)
            .await;
        } else {
            not_assigned.push(user_id);
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