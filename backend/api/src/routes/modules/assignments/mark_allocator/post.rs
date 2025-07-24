use axum::{extract::Path, http::StatusCode, response::IntoResponse, Json};
use util::mark_allocator::mark_allocator::{generate_allocator, SaveError};

use crate::response::ApiResponse;

/// POST /api/modules/:module_id/assignments/:assignment_id/mark_allocator
///
/// Generate a new mark allocator configuration for a specific assignment. Accessible to users with
/// Lecturer roles assigned to the module.
///
/// This endpoint automatically generates a mark allocator configuration based on the memo output
/// files for the assignment. The generated configuration defines how marks are distributed across
/// different tasks and criteria, ensuring proper weight allocation for the grading system.
///
/// ### Path Parameters
/// - `module_id` (i64): The ID of the module containing the assignment
/// - `assignment_id` (i64): The ID of the assignment to generate mark allocator for
///
/// ### Example Request
/// ```bash
/// curl -X POST http://localhost:3000/api/modules/1/assignments/2/mark_allocator \
///   -H "Authorization: Bearer <token>"
/// ```
///
/// ### Success Response (200 OK)
/// ```json
/// {
///   "success": true,
///   "message": "Mark allocator successfully generated.",
///   "data": {
///     "tasks": [
///       {
///         "task_number": 1,
///         "weight": 0.4,
///         "criteria": [
///           {
///             "name": "Correctness",
///             "weight": 0.7
///           },
///           {
///             "name": "Code Quality",
///             "weight": 0.3
///           }
///         ]
///       }
///     ],
///     "total_weight": 1.0
///   }
/// }
/// ```
///
/// ### Error Responses
///
/// **404 Not Found** - Module or assignment folder does not exist
/// ```json
/// {
///   "success": false,
///   "message": "Module or assignment folder does not exist",
///   "data": null
/// }
/// ```
///
/// **500 Internal Server Error** - Failed to generate mark allocator
/// ```json
/// {
///   "success": false,
///   "message": "Failed to generate mark allocator",
///   "data": null
/// }
/// ```
///
/// ### Notes
/// - This endpoint requires memo output files to exist in the assignment directory
/// - The generated configuration is based on analysis of memo content and structure
/// - Task weights are automatically calculated to ensure fair distribution
/// - Generation is restricted to users with Lecturer permissions for the module
/// - The generated allocator can be further customized using the PUT endpoint
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
            Json(ApiResponse::<()>::error(
                "Module or assignment folder does not exist",
            )),
        )
            .into_response(),

        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(
                "Failed to generate mark allocator",
            )),
        )
            .into_response(),
    }
}