use axum::{
    extract::{State, Extension, Path},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sea_orm::{EntityTrait, DatabaseConnection};
use crate::{
    auth::claims::AuthUser,
    response::ApiResponse,
};
use db::models::user::{Entity as UserEntity};

/// DELETE /users/{user_id}
///
/// Delete a user by their ID. Only admins can access this endpoint.
/// Users cannot delete their own account.
///
/// ### Path Parameters
/// - `id` - The ID of the user to delete
///
/// ### Responses
///
/// - `200 OK`  
/// ```json
/// {
///   "success": true,
///   "message": "User deleted successfully"
/// }
/// ```
///
/// - `404 Not Found`  
/// ```json
/// {
///   "success": false,
///   "message": "User not found"
/// }
/// ```
///
/// - `400 Bad Request` (invalid ID format)  
/// ```json
/// {
///   "success": false,
///   "message": "Invalid user ID format"
/// }
/// ```
///
/// - `403 Forbidden`  
/// ```json
/// {
///   "success": false,
///   "message": "You cannot delete your own account"
/// }
/// ```
///
/// - `500 Internal Server Error`  
/// ```json
/// {
///   "success": false,
///   "message": "Database error: detailed error here"
/// }
/// ```
pub async fn delete_user(
    State(db): State<DatabaseConnection>,
    Path(user_id): Path<i64>,
    Extension(AuthUser(claims)): Extension<AuthUser>,
) -> impl IntoResponse {
    if user_id == claims.sub {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<()>::error("You cannot delete your own account")),
        );
    }

    match UserEntity::delete_by_id(user_id).exec(&db).await {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse::success_without_data("User deleted successfully")),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Database error: {}", e))),
        ),
    }
}