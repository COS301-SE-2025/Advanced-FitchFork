use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use crate::response::ApiResponse;
use services::service::Service;
use services::user::UserService;

/// DELETE `/api/test/users/{user_id}`
///
/// Deletes a user by their **numeric ID**.  
/// Intended for **test environment teardown** in non-production environments.
///
/// # Path parameters
/// - `user_id` — The primary key of the user to delete.
///
/// # Example request
/// ```bash
/// curl -X DELETE http://localhost:3000/api/test/users/1
/// ```
///
/// ## 200 OK
/// ```json
/// { "success": true, "data": null, "message": "User deleted" }
/// ```
///
/// ## 404 Not Found
/// ```json
/// { "success": false, "data": null, "message": "User not found" }
/// ```
pub async fn delete_user(
    Path(user_id): Path<i64>,
) -> impl IntoResponse {
    match UserService::find_by_id(user_id).await {
        Ok(Some(_)) => {
            if let Err(e) = UserService::delete_by_id(user_id).await {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()>::error(format!("Database error: {e}"))),
                )
                    .into_response();
            }
            (
                StatusCode::OK,
                Json(ApiResponse::<()>::success((), "User deleted")),
            )
                .into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("User not found")),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Database error: {e}"))),
        )
            .into_response(),
    }
}
