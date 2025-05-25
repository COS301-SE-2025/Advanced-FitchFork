use axum::extract::Path;
use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use db::models::user::User;
use db::pool;
use crate::auth::AuthUser;
use crate::response::ApiResponse;
use crate::routes::modules::post::PersonnelResponse;


/// GET /api/modules/:module_id/lecturers
///
/// Retrieve a list of users assigned as lecturers to the specified module.
///
/// ### Access Control
/// This endpoint is accessible to:
/// - Admin users (claims.admin == true)
///
/// ### Path Parameter
/// - `module_id` (integer): The ID of the module to retrieve lecturers for.
///
/// ### Authentication
/// Requires a valid JWT with the appropriate permissions. Returns 403 if the user
/// is not assigned to the module or is not an admin.
///
/// ### Responses
///
/// - `200 OK`  
/// ```json
/// {
///   "success": true,
///   "data": [
///     {
///       "id": 1,
///       "student_number": "u12345678",
///       "email": "lecturer@example.com",
///       "admin": false,
///       "created_at": "2025-05-23T18:00:00Z",
///       "updated_at": "2025-05-23T18:00:00Z"
///     }
///   ],
///   "message": "Lecturers retrieved successfully"
/// }
/// ```
///
/// - `403 Forbidden`  
/// ```json
/// {
///   "success": false,
///   "message": "You do not have permission to view this module's users"
/// }
/// ```
///
/// - `404 Not Found`  
/// ```json
/// {
///   "success": false,
///   "message": "Module not found"
/// }
/// ```
pub async fn get_lecturers(Path(module_id): Path<i64>, AuthUser(claims): AuthUser, ) -> Response {
    let pool = pool::get();

    let module_exists = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM modules WHERE id = ?)").bind(module_id).fetch_one(pool).await.unwrap_or(false);

    if !module_exists {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Module not found")),
        ).into_response();
    }

    let is_admin = claims.admin;
    let is_assigned = sqlx::query_scalar(
        "SELECT EXISTS(
            SELECT 1 FROM module_lecturers WHERE module_id = ? AND user_id = ?
            UNION
            SELECT 1 FROM module_tutors WHERE module_id = ? AND user_id = ?
            UNION
            SELECT 1 FROM module_students WHERE module_id = ? AND user_id = ?
        )",
    ).bind(module_id).bind(claims.sub).bind(module_id).bind(claims.sub).bind(module_id).bind(claims.sub).fetch_one(pool).await.unwrap_or(false);

    if !is_admin && !is_assigned {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<()>::error("You do not have permission to view this module's users")),
        ).into_response();
    }

    let users = sqlx::query_as::<_, User>(
        "SELECT u.* FROM users u
         INNER JOIN module_lecturers ml ON u.id = ml.user_id
         WHERE ml.module_id = ?",
    ).bind(module_id).fetch_all(pool).await.unwrap_or_else(|_| vec![]);

    let result: Vec<PersonnelResponse> = users.into_iter().map(Into::into).collect();

    (
        StatusCode::OK,
        Json(ApiResponse::success(
            result,
            "Lecturers retrieved successfully",
        )),
    ).into_response()
}


/// GET /api/modules/:module_id/tutors
///
/// Retrieve a list of users assigned as tutors to the specified module.
///
/// ### Access Control
/// This endpoint is accessible to:
/// - Admin users
/// 
///
/// ### Path Parameter
/// - `module_id` (integer): The ID of the module to retrieve tutors for.
///
/// ### Authentication
/// Requires a valid JWT. Users must be either an admin or assigned to the module.
///
/// ### Responses
///
/// - `200 OK`  
/// ```json
/// {
///   "success": true,
///   "data": [
///     {
///       "id": 7,
///       "student_number": "u22222222",
///       "email": "tutor@example.com",
///       "admin": false,
///       "created_at": "2025-05-23T18:00:00Z",
///       "updated_at": "2025-05-23T18:00:00Z"
///     }
///   ],
///   "message": "Tutors retrieved successfully"
/// }
/// ```
///
/// - `403 Forbidden`  
/// ```json
/// {
///   "success": false,
///   "message": "You do not have permission to view this module's users"
/// }
/// ```
///
/// - `404 Not Found`  
/// ```json
/// {
///   "success": false,
///   "message": "Module not found"
/// }
/// ```

pub async fn get_tutors(Path(module_id): Path<i64>, AuthUser(claims): AuthUser, ) -> axum::response::Response {
    let pool = db::pool::get();

    let module_exists = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM modules WHERE id = ?)")
        .bind(module_id)
        .fetch_one(pool)
        .await
        .unwrap_or(false);

    if !module_exists {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Module not found")),
        ).into_response();
    }

    let is_admin = claims.admin;
    let is_assigned = sqlx::query_scalar(
        "SELECT EXISTS(
            SELECT 1 FROM module_lecturers WHERE module_id = ? AND user_id = ?
            UNION
            SELECT 1 FROM module_tutors WHERE module_id = ? AND user_id = ?
            UNION
            SELECT 1 FROM module_students WHERE module_id = ? AND user_id = ?
        )",
    ).bind(module_id).bind(claims.sub).bind(module_id).bind(claims.sub).bind(module_id).bind(claims.sub).fetch_one(pool).await.unwrap_or(false);

    if !is_admin && !is_assigned {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<()>::error("You do not have permission to view this module's users")),
        ).into_response();
    }

    let users = sqlx::query_as::<_, db::models::user::User>(
        "SELECT u.* FROM users u
         INNER JOIN module_tutors mt ON u.id = mt.user_id
         WHERE mt.module_id = ?",
    ).bind(module_id).fetch_all(pool).await.unwrap_or_else(|_| vec![]);
    
    let result: Vec<PersonnelResponse> = users.into_iter().map(Into::into).collect();

    (
        StatusCode::OK,
        Json(ApiResponse::success(result, "Tutors retrieved successfully")),
    ).into_response()
}

/// GET /api/modules/:module_id/students
///
/// Retrieve a list of users enrolled as students in the specified module.
///
/// ### Access Control
/// This endpoint is accessible to:
/// - Admin users
/// 
///
/// ### Path Parameter
/// - `module_id` (integer): The ID of the module to retrieve students for.
///
/// ### Authentication
/// Requires a valid JWT. Access is denied to unauthorized users.
///
/// ### Responses
///
/// - `200 OK`  
/// ```json
/// {
///   "success": true,
///   "data": [
///     {
///       "id": 11,
///       "student_number": "u33333333",
///       "email": "student@example.com",
///       "admin": false,
///       "created_at": "2025-05-23T18:00:00Z",
///       "updated_at": "2025-05-23T18:00:00Z"
///     }
///   ],
///   "message": "Students retrieved successfully"
/// }
/// ```
///
/// - `403 Forbidden`  
/// ```json
/// {
///   "success": false,
///   "message": "You do not have permission to view this module's users"
/// }
/// ```
///
/// - `404 Not Found`  
/// ```json
/// {
///   "success": false,
///   "message": "Module not found"
/// }
/// ```
pub async fn get_students(Path(module_id): Path<i64>, AuthUser(claims): AuthUser, ) -> axum::response::Response {
    let pool = db::pool::get();

    let module_exists = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM modules WHERE id = ?)").bind(module_id).fetch_one(pool).await.unwrap_or(false);

    if !module_exists {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Module not found")),
        ).into_response();
    }

    let is_admin = claims.admin;
    let is_assigned = sqlx::query_scalar(
        "SELECT EXISTS(
            SELECT 1 FROM module_lecturers WHERE module_id = ? AND user_id = ?
            UNION
            SELECT 1 FROM module_tutors WHERE module_id = ? AND user_id = ?
            UNION
            SELECT 1 FROM module_students WHERE module_id = ? AND user_id = ?
        )",
    ).bind(module_id).bind(claims.sub).bind(module_id).bind(claims.sub).bind(module_id).bind(claims.sub).fetch_one(pool).await.unwrap_or(false);
    
    if !is_admin && !is_assigned {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<()>::error("You do not have permission to view this module's users")),
        ).into_response();
    }

    let users = sqlx::query_as::<_, db::models::user::User>(
        "SELECT u.* FROM users u
         INNER JOIN module_students ms ON u.id = ms.user_id
         WHERE ms.module_id = ?",
    ).bind(module_id).fetch_all(pool).await.unwrap_or_else(|_| vec![]);

    let result: Vec<PersonnelResponse> = users.into_iter().map(Into::into).collect();

    (
        StatusCode::OK,
        Json(ApiResponse::success(result, "Students retrieved successfully")),
    ).into_response()
}
