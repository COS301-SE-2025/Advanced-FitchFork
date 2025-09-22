use crate::response::ApiResponse;
use axum::{Json, extract::Path, http::StatusCode, response::IntoResponse};
use db::models::assignment_memo_output;
use db::models::assignment_task;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use util::mark_allocator::TaskInfo;
use util::mark_allocator::{SaveError, generate_allocator};
use util::paths::{memo_output_dir, storage_root};
use util::state::AppState;

/// POST /api/modules/{module_id}/assignments/{assignment_id}/mark_allocator
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
    let tasks = match AssignmentTaskService::find_all(
        &vec![FilterParam::eq("assignment_id", assignment_id)],
        &vec![],
        None,
    )
    .await
    {
        Ok(t) => t,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("Failed to fetch assignment tasks")),
            )
                .into_response();
        }
    };

    let memo_dir = memo_output_dir(module_id, assignment_id);
    let mut task_file_pairs = vec![];

    for task in &tasks {
        let task_info = TaskInfo {
            id: task.id,
            task_number: task.task_number,
            code_coverage: task.code_coverage,
            name: if task.name.trim().is_empty() {
                format!("Task {}", task.task_number)
            } else {
                task.name.clone()
            },
        };

        let memo_output_res = assignment_memo_output::Entity::find()
            .filter(assignment_memo_output::Column::AssignmentId.eq(assignment_id))
            .filter(assignment_memo_output::Column::TaskId.eq(task.id))
            .one(db)
            .await;

        let memo_path = match memo_output_res {
            Ok(Some(memo_output)) => storage_root().join(&memo_output.path),
            Ok(None) => memo_dir.join(format!("no_memo_for_task_{}", task.id)),
            Err(_) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()>::error("Failed to fetch memo outputs")),
                )
                    .into_response();
            }
        };

        task_file_pairs.push((task_info, memo_path));
    }

    match generate_allocator(module_id, assignment_id, &task_file_pairs).await {
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
