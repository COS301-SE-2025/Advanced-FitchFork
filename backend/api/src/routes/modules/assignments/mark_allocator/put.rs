use axum::{extract::Path, http::StatusCode, response::IntoResponse, Json};
use serde_json::Value;
use util::mark_allocator::mark_allocator::{save_allocator, SaveError};

use crate::response::ApiResponse;

/// Save the mark allocator JSON for a specific module and assignment.
///
/// # Endpoint
/// `PUT /allocator`
///
/// # Path Parameters
/// - `module_id`: The ID of the module.
/// - `assignment_id`: The ID of the assignment.
///
/// # Request Body
/// A valid JSON `Value` representing the allocator structure (e.g., tasks, subsections).
///
/// # Returns
/// - `200 OK`: If the allocator was successfully saved.
/// - `404 NOT FOUND`: If the corresponding module or assignment folder does not exist.
/// - `500 INTERNAL SERVER ERROR`: If an error occurs during file creation or writing.
///
/// # Example Request Body
/// ```json
/// {
///   "tasks": [
///     {
///       "task1": {
///         "name": "Task 1",
///         "value": 10,
///         "subsections": [
///           { "name": "Subsection A", "value": 5 },
///           { "name": "Subsection B", "value": 5 }
///         ]
///       }
///     }
///   ]
/// }
/// ```
///
/// # Example Successful Response
/// ```json
/// {
///   "success": true,
///   "message": "Mark allocator successfully saved.",
///   "data": "{}"
/// }
/// ```
///
/// # Example Error Response (Directory Not Found)
/// ```json
/// {
///   "success": false,
///   "message": "Module or assignment does not exist"
/// }
/// ```
///
/// # Example Error Response (Internal Error)
/// ```json
/// {
///   "success": false,
///   "message": "Could not save file"
/// }
/// ```
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
            StatusCode::NOT_FOUND,
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
