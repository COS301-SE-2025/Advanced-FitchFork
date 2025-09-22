use crate::response::ApiResponse;
use axum::{Json, extract::Path, http::StatusCode};
use code_runner::create_memo_outputs_for_all_tasks;
use std::fs;
use tracing::{error, info};
use util::{
    paths::{config_dir, memo_dir},
    state::AppState,
};

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
/// The endpoint validates the presence of required directories under and assignment:
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
    let db = app_state.db();

    // Use centralized helpers for directories
    let memo_dir = memo_dir(module_id, assignment_id);
    let memo_valid = memo_dir.is_dir()
        && fs::read_dir(&memo_dir)
            .map(|mut entries| entries.any(|e| e.ok().map(|f| f.path().is_file()).unwrap_or(false)))
            .unwrap_or(false);

    if !memo_valid {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ApiResponse::<()>::error(
                "Required memo directory is missing or empty",
            )),
        );
    }

    let cfg_dir = config_dir(module_id, assignment_id);
    let config_valid = cfg_dir.is_dir()
        && fs::read_dir(&cfg_dir)
            .map(|mut entries| entries.any(|e| e.ok().map(|f| f.path().is_file()).unwrap_or(false)))
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
            let err_str = e.to_string();
            error!(
                "Memo output generation failed for assignment {}: {}",
                assignment_id, err_str
            );

            // Map low-level error strings to user-friendly messages + appropriate status codes.
            let (status, message) = if err_str.contains("Config validation failed")
                || err_str.contains("Failed to load execution config")
            {
                (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "Your configuration is missing or invalid. Open the Config step and save settings before generating memo output.",
                )
            } else if err_str.contains("memo")
                && (err_str.contains("Required directory")
                    || err_str.contains("Missing directory")
                    || err_str.contains("No .zip"))
            {
                (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "Memo archive (.zip) not found. Upload your Memo files under Files & Resources.",
                )
            } else if err_str.contains("makefile")
                && (err_str.contains("Required directory")
                    || err_str.contains("Missing directory")
                    || err_str.contains("No .zip"))
            {
                (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "Makefile archive (.zip) not found. Upload a Makefile under Files & Resources.",
                )
            } else if err_str.contains("main")
                && (err_str.contains("Required directory")
                    || err_str.contains("Missing directory")
                    || err_str.contains("No .zip"))
            {
                (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "Main files (.zip) not found. In manual mode, upload Main Files; in GATLAM mode, ensure the Interpreter is configured.",
                )
            } else if err_str.contains("No tasks are defined") || err_str.contains("No tasks found")
            {
                (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "No tasks are defined yet. Add at least one task and try again.",
                )
            } else if err_str.contains("Failed to send request to code_manager") {
                (
                    StatusCode::SERVICE_UNAVAILABLE,
                    "The runner service is unavailable. Please try again shortly or contact support if it persists.",
                )
            } else if err_str.contains("code_manager responded with error")
                || err_str.contains("Failed to parse response JSON")
                || err_str.contains("Response missing 'output'")
            {
                (
                    StatusCode::BAD_GATEWAY,
                    "The runner failed to execute your tasks. Check your build/commands and execution limits, then retry.",
                )
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to generate memo output. Please retry; if it continues, contact support.",
                )
            };

            (status, Json(ApiResponse::<()>::error(message)))
        }
    }
}
