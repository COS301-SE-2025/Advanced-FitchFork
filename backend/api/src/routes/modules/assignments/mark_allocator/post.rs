use axum::{extract::Path, http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use util::mark_allocator::mark_allocator::{generate_allocator, SaveError};

use crate::response::ApiResponse;

pub async fn generate(Path((module_id, assignment_id)): Path<(i64, i64)>) -> impl IntoResponse {
    match generate_allocator(module_id, assignment_id).await {
        Ok(json) => (
            StatusCode::OK,
            Json(ApiResponse::success(
                json,
                "Mark allocator successfully generated.",
            )),
        )
            .into_response(),

        Err(SaveError::DirectoryNotFound) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Module or assignment folder does not exist" })),
        )
            .into_response(),

        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to generate mark allocator" })),
        )
            .into_response(),
    }
}
