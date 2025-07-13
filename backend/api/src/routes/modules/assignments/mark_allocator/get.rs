use axum::{extract::Path, http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use util::mark_allocator::mark_allocator::{load_allocator, SaveError};

use crate::response::ApiResponse;

/// GET /api/modules/:module_id/assignments/:assignment_id/mark_allocator
///
/// Load the mark allocator JSON configuration for a specific assignment. Accessible to users with
/// appropriate permissions assigned to the module.
///
/// The mark allocator configuration defines how marks are distributed across different tasks and
/// criteria within an assignment. This configuration is used by the grading system to determine
/// the weight and allocation of marks for various assessment components.
///
/// ### Path Parameters
/// - `module_id` (i64): The ID of the module containing the assignment
/// - `assignment_id` (i64): The ID of the assignment to load mark allocator for
///
/// ### Example Request
/// ```bash
/// curl -X GET http://localhost:3000/api/modules/1/assignments/2/mark_allocator \
///   -H "Authorization: Bearer <token>"
/// ```
///
/// ### Success Response (200 OK)
/// ```json
/// {
///   "success": true,
///   "message": "Mark allocator successfully loaded.",
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
///       },
///       {
///         "task_number": 2,
///         "weight": 0.6,
///         "criteria": [
///           {
///             "name": "Functionality",
///             "weight": 0.8
///           },
///           {
///             "name": "Documentation",
///             "weight": 0.2
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
///   "error": "Module or assignment folder does not exist"
/// }
/// ```
///
/// **500 Internal Server Error** - Failed to load allocator
/// ```json
/// {
///   "error": "Failed to load allocator"
/// }
/// ```
///
/// ### Mark Allocator Structure
/// The mark allocator configuration typically contains:
/// - `tasks`: Array of task configurations with mark allocations
///   - `task_number`: Sequential number of the task
///   - `weight`: Relative weight of this task in the overall assignment (0.0 to 1.0)
///   - `criteria`: Array of grading criteria for this task
///     - `name`: Name of the grading criterion
///     - `weight`: Weight of this criterion within the task (0.0 to 1.0)
/// - `total_weight`: Sum of all task weights (should equal 1.0)
///
/// ### Notes
/// - The mark allocator configuration is stored as a JSON file in the module/assignment directory
/// - This configuration is used by the grading system to calculate final marks
/// - Task weights should sum to 1.0 across all tasks in the assignment
/// - Criteria weights within each task should also sum to 1.0
/// - Mark allocator loading is restricted to users with appropriate module permissions
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