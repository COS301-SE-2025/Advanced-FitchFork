use axum::{extract::Path, http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use util::mark_allocator::mark_allocator::{generate_allocator, SaveError};

use crate::response::ApiResponse;

/// Generate a new mark allocator for a given module and assignment.
///
/// # Endpoint
/// `POST /allocator`
///
/// # Path Parameters
/// - `module_id`: The ID of the module.
/// - `assignment_id`: The ID of the assignment.
///
/// # Returns
/// - `200 OK`: If the allocator was successfully generated. Returns a JSON response with the generated allocator structure.
/// - `404 NOT FOUND`: If the `memo_output` directory for the module and assignment doesn't exist.
/// - `500 INTERNAL SERVER ERROR`: If an unexpected error occurs during reading, processing, or writing.
///
/// # Example Successful Response
/// ```json
/// {
///   "success": true,
///   "message": "Mark allocator successfully generated.",
///   "data": {
///     "tasks": [ ... ]
///   }
/// }
/// ```
///
/// # Example Error Response (Directory Not Found)
/// ```json
/// {
///   "error": "Module or assignment folder does not exist"
/// }
/// ```
///
/// # Example Error Response (Internal Error)
/// ```json
/// {
///   "error": "Failed to generate mark allocator"
/// }
/// ```
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
            StatusCode::NOT_FOUND,
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