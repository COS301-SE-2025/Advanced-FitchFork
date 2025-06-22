use axum::{extract::Path, http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use util::mark_allocator::mark_allocator::{load_allocator, SaveError};

use crate::response::ApiResponse;

/// Load the mark allocator JSON for a specific module and assignment.
///
/// # Endpoint
/// `GET /allocator`
///
/// # Path Parameters
/// - `module_id`: The ID of the module.
/// - `assignment_id`: The ID of the assignment.
///
/// # Returns
/// - `200 OK`: If the allocator was successfully loaded. Returns a JSON response with the allocator data.
/// - `404 NOT FOUND`: If the corresponding module or assignment directory does not exist.
/// - `500 INTERNAL SERVER ERROR`: If there was an unexpected error during loading (e.g., I/O or JSON issues).
///
/// # Example Successful Response
/// ```json
/// {
///   "success": true,
///   "message": "Mark allocator successfully loaded.",
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
/// # Example Error Response (Other Failure)
/// ```json
/// {
///   "error": "Failed to load allocator"
/// }
/// ```
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
            Json(json!({ "error": "Module or assignment folder does not exist" })),
        )
            .into_response(),

        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to load allocator" })),
        )
            .into_response(),
    }
}
