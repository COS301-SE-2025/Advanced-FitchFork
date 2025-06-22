use axum::{extract::Path, http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use util::mark_allocator::mark_allocator::{load_allocator, SaveError};

use crate::response::ApiResponse;

pub async fn load(Path((module_id, assignment_id)): Path<(i64, i64)>) -> impl IntoResponse {
    match load_allocator(module_id, assignment_id).await {
        Ok(json) => (
            StatusCode::OK,
            Json(ApiResponse::success(
                json,
                "Mark allocator successfully loaded.",
            )),
        )
            .into_response(),

        Err(SaveError::DirectoryNotFound) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Allocator not found" })),
        )
            .into_response(),

        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to load allocator" })),
        )
            .into_response(),
    }
}
