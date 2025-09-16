use crate::response::ApiResponse;
use axum::{
    Json,
    extract::Path,
    http::StatusCode,
};
use code_runner::create_memo_outputs_for_all_tasks;
use std::{env, fs, path::PathBuf};
use tracing::{error, info};

/// POST /api/modules/{module_id}/assignments/{assignment_id}/memo_output/generate
///
/// Start asynchronous generation of memo outputs for all tasks in the specified assignment. Accessible
/// to users with Lecturer or Admin roles assigned to the module.
///
/// This endpoint validates the presence of required directories and starts a background task to
/// generate memo outputs for all tasks in the assignment. The memo outputs are used by the grading
/// system to evaluate student submissions against expected results.
///
/// ### Path Parameters
/// - `module_id` (i64): The ID of the module containing the assignment
/// - `assignment_id` (i64): The ID of the assignment to generate memo outputs for
///
/// ### Example Request
/// ```bash
/// curl -X POST http://localhost:3000/api/modules/1/assignments/2/memo_output/generate \
///   -H "Authorization: Bearer <token>"
/// ```
///
/// ### Success Response (202 Accepted)
/// ```json
/// {
///   "success": true,
///   "message": "Task started",
///   "data": null
/// }
/// ```
///
/// ### Error Responses
///
/// **422 Unprocessable Entity** - Required directories missing or empty
/// ```json
/// {
///   "success": false,
///   "message": "Required memo directory is missing or empty"
/// }
/// ```
/// or
/// ```json
/// {
///   "success": false,
///   "message": "Config file not valid"
/// }
/// ```
///
/// **403 Forbidden** - Insufficient permissions
/// ```json
/// {
///   "success": false,
///   "message": "Access denied"
/// }
/// ```
///
/// **404 Not Found** - Assignment or module not found
/// ```json
/// {
///   "success": false,
///   "message": "Assignment or module not found"
/// }
/// ```
///
/// ### Directory Requirements
/// The endpoint validates the presence of required directories under:
/// `ASSIGNMENT_STORAGE_ROOT/module_{module_id}/assignment_{assignment_id}/`
///
/// - `memo/` directory: Must exist and contain at least one file
///   - Contains memo files that define expected outputs for each task
///   - Used as reference for evaluating student submissions
/// - `config/` directory: Must exist and contain at least one file
///   - Contains configuration files for task execution
///   - Defines test parameters and evaluation criteria
///
/// ### Background Processing
/// - Memo generation is performed asynchronously in a background task
/// - The response is returned immediately after validation
/// - Processing continues even if the client disconnects
/// - Task completion is logged with success/failure status
/// - Generated memo outputs are stored in the assignment directory
///
/// ### Notes
/// - This endpoint only starts the generation process; it does not wait for completion
/// - Memo outputs are essential for the grading system to function properly
/// - The background task processes all tasks defined in the assignment
/// - Generation is restricted to users with appropriate module permissions
/// - Check server logs for detailed progress and error information
pub async fn generate_memo_output(
    Path((module_id, assignment_id)): Path<(i64, i64)>,
) -> (StatusCode, Json<ApiResponse<()>>) {
    let base_path = env::var("ASSIGNMENT_STORAGE_ROOT").unwrap_or_else(|_| "data/assignment_files".into());

    let assignment_root = PathBuf::from(&base_path)
        .join(format!("module_{}", module_id))
        .join(format!("assignment_{}", assignment_id));

    let memo_dir = assignment_root.join("memo");
    let memo_valid = memo_dir.is_dir()
        && fs::read_dir(&memo_dir)
            .map(|entries| {
                entries
                    .filter_map(Result::ok)
                    .any(|entry| entry.path().is_file())
            })
            .unwrap_or(false);

    if !memo_valid {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ApiResponse::<()>::error(
                "Required memo directory is missing or empty",
            )),
        );
    }

    let config_dir = assignment_root.join("config");
    let config_valid = config_dir.is_dir()
        && fs::read_dir(&config_dir)
            .map(|entries| {
                entries
                    .filter_map(Result::ok)
                    .any(|entry| entry.path().is_file())
            })
            .unwrap_or(false);

    if !config_valid {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ApiResponse::<()>::error("Config file not valid")),
        );
    }

    match create_memo_outputs_for_all_tasks(assignment_id).await {
        Ok(_) => {
            info!(
                "Memo output generation complete for assignment {}",
                assignment_id
            );
            (
                StatusCode::OK,
                Json(ApiResponse::<()>::success(
                    (),
                    "Memo output generation complete",
                )),
            )
        }
        Err(e) => {
            println!("{}", e);
            error!(
                "Memo output generation failed for assignment {}: {:?}",
                assignment_id, e
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("Failed to generate memo output")),
            )
        }
    }
}