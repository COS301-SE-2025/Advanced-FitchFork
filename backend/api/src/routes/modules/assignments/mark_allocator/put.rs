use axum::{extract::Path, http::StatusCode, response::IntoResponse, Json};
use serde_json::Value;
use util::mark_allocator::mark_allocator::{save_allocator, SaveError};

use crate::response::ApiResponse;

/// PUT /api/modules/:module_id/assignments/:assignment_id/mark_allocator
///
/// Save the mark allocator JSON configuration for a specific assignment. Accessible to users with
/// Lecturer roles assigned to the module.
///
/// This endpoint saves a mark allocator configuration to the assignment directory. The configuration
/// defines how marks are distributed across different tasks and criteria within the assignment.
/// The saved configuration is used by the grading system to calculate final marks.
///
/// ### Path Parameters
/// - `module_id` (i64): The ID of the module containing the assignment
/// - `assignment_id` (i64): The ID of the assignment to save mark allocator for
///
/// ### Request Body
/// A valid JSON object representing the mark allocator structure:
/// ```json
/// {
///   "tasks": [
///     {
///       "task_number": 1,
///       "weight": 0.4,
///       "criteria": [
///         {
///           "name": "Correctness",
///           "weight": 0.7
///         },
///         {
///           "name": "Code Quality",
///           "weight": 0.3
///         }
///       ]
///     },
///     {
///       "task_number": 2,
///       "weight": 0.6,
///       "criteria": [
///         {
///           "name": "Functionality",
///           "weight": 0.8
///         },
///         {
///           "name": "Documentation",
///           "weight": 0.2
///         }
///       ]
///     }
///   ],
///   "total_weight": 1.0
/// }
/// ```
///
/// ### Request Body Structure
/// The mark allocator configuration should contain:
/// - `tasks`: Array of task configurations with mark allocations
///   - `task_number`: Sequential number of the task
///   - `weight`: Relative weight of this task in the overall assignment (0.0 to 1.0)
///   - `criteria`: Array of grading criteria for this task
///     - `name`: Name of the grading criterion
///     - `weight`: Weight of this criterion within the task (0.0 to 1.0)
/// - `total_weight`: Sum of all task weights (should equal 1.0)
///
/// ### Example Request
/// ```bash
/// curl -X PUT http://localhost:3000/api/modules/1/assignments/2/mark_allocator \
///   -H "Authorization: Bearer <token>" \
///   -H "Content-Type: application/json" \
///   -d '{
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
///   }'
/// ```
///
/// ### Success Response (200 OK)
/// ```json
/// {
///   "success": true,
///   "message": "Mark allocator successfully saved.",
///   "data": "{}"
/// }
/// ```
///
/// ### Error Responses
///
/// **404 Not Found** - Module or assignment does not exist
/// ```json
/// {
///   "success": false,
///   "message": "Module or assignment does not exist"
/// }
/// ```
///
/// **500 Internal Server Error** - Could not save file
/// ```json
/// {
///   "success": false,
///   "message": "Could not save file"
/// }
/// ```
///
/// ### Validation Rules
/// - Request body must be a valid JSON object
/// - Task weights should sum to 1.0 across all tasks
/// - Criteria weights within each task should sum to 1.0
/// - Assignment must exist and belong to the specified module
/// - User must have Lecturer permissions for the module
///
/// ### Notes
/// - The mark allocator configuration is saved as a JSON file in the assignment directory
/// - This configuration is used by the grading system to calculate final marks
/// - The saved configuration replaces any existing mark allocator for the assignment
/// - Task weights should be carefully balanced to ensure fair mark distribution
/// - Saving is restricted to users with Lecturer permissions for the module
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