use axum::{extract::Path, http::StatusCode, response::IntoResponse, Json};
use serde_json::Value;
use util::mark_allocator::mark_allocator::{save_allocator, SaveError};

use crate::response::ApiResponse;

pub async fn save(
    Path((module_id, assignment_id)): Path<(i64, i64)>,
    Json(req): Json<Value>,
) -> impl IntoResponse {
    let res = save_allocator(module_id, assignment_id, req).await;

    match res {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse::success(
                "{}",
                "Mark allocator successfully saved.",
            )),
        )
            .into_response(),

        Err(SaveError::DirectoryNotFound) => (
            StatusCode::BAD_REQUEST,
            Json::<ApiResponse<()>>(ApiResponse::error("Module or assignment does not exist")),
        )
            .into_response(),

        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json::<ApiResponse<()>>(ApiResponse::error("Could not save file")),
        )
            .into_response(),
    }
}
