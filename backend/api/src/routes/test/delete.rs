use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use sea_orm::{ActiveModelTrait, EntityTrait};
use util::state::AppState;

use crate::response::ApiResponse;
use db::models::user::{ActiveModel as UserActiveModel, Entity as UserEntity};

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
    State(app): State<AppState>,
    Path(user_id): Path<i32>,
) -> impl IntoResponse {
    let db = app.db();

    match UserEntity::find_by_id(user_id).one(db).await {
        Ok(Some(user)) => {
            let am: UserActiveModel = user.into();
            if let Err(e) = am.delete(db).await {
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
