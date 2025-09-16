use crate::response::ApiResponse;
use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::to_value;
use util::execution_config::ExecutionConfig;
use util::filters::FilterParam;
use services::service::Service;
use services::assignment::AssignmentService;
use services::assignment_file::AssignmentFileService;

/// GET /api/modules/{module_id}/assignments/{assignment_id}/config
///
/// Retrieve the JSON configuration object associated with a specific assignment. Accessible to users
/// assigned to the module with appropriate permissions.
///
/// The configuration object is loaded from disk using the [`ExecutionConfig`] schema. If no configuration
/// file is present on disk, an empty config is returned instead.
///
/// ### Path Parameters
/// - `module_id` (i64): The ID of the module containing the assignment
/// - `assignment_id` (i64): The ID of the assignment to retrieve configuration for
///
/// ### Example Request
/// ```bash
/// curl -X GET http://localhost:3000/api/modules/1/assignments/2/config \
///   -H "Authorization: Bearer <token>"
/// ```
///
/// ### Success Response (200 OK) - With Configuration
/// ```json
/// {
///   "success": true,
///   "message": "Assignment configuration retrieved successfully",
///   "data": {
///     "execution": {
///       "timeout_secs": 10,
///       "max_memory": 8589934592,
///       "max_cpus": 2,
///       "max_uncompressed_size": 100000000,
///       "max_processes": 256
///     },
///     "marking": {
///       "marking_scheme": "exact",
///       "feedback_scheme": "auto",
///       "deliminator": "&-=-&"
///     }
///   }
/// }
/// ```
///
/// ### Success Response (200 OK) - No Configuration File
/// ```json
/// {
///   "success": true,
///   "message": "No configuration set for this assignment",
///   "data": {}
/// }
/// ```
///
/// ### Error Responses
/// - **404** – Assignment not found
/// - **500** – Failed to load configuration from disk
///
/// ### Notes
/// - Configurations are stored on disk under `ASSIGNMENT_STORAGE_ROOT/module_{id}/assignment_{id}/config/config.json`
/// - Config format uses [`ExecutionConfig`] as the schema
/// - This is an example schema and will evolve over time
pub async fn get_assignment_config(
    Path((module_id, assignment_id)): Path<(i64, i64)>,
) -> impl IntoResponse {
    // Verify the assignment exists
    match AssignmentService::find_by_id(assignment_id).await {
        Ok(Some(_)) => {}
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<()>::error("Assignment or module not found")),
            )
                .into_response();
        }
        Err(e) => {
            eprintln!("DB error: {:?}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("Database error")),
            )
                .into_response();
        }
    }

    // Look up the latest config assignment file
    let config_file = match AssignmentFileService::find_one(
        &vec![
            FilterParam::eq("assignment_id", assignment_id),
            FilterParam::eq("file_type", "config".to_string()),
        ],
        &vec![],
        Some("-updated_at".to_string()),
    ).await {
        Ok(opt) => opt,
        Err(e) => {
            eprintln!("DB error while fetching config file: {:?}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("Database error")),
            )
                .into_response();
        }
    };

    // Load the config from the file model
    match config_file {
        Some(_) => match AssignmentFileService::load_execution_config(module_id, assignment_id).await {
            Ok(cfg) => {
                let json = to_value(cfg).unwrap_or_else(|_| serde_json::json!({}));
                (
                    StatusCode::OK,
                    Json(ApiResponse::success(json, "Assignment configuration retrieved successfully")),
                )
                    .into_response()
            }
            Err(err) => {
                eprintln!("Failed to load config from disk: {}", err);
                (
                    StatusCode::OK,
                    Json(ApiResponse::success(
                        serde_json::json!({}),
                        "No configuration set for this assignment",
                    )),
                )
                    .into_response()
            }
        },
        None => (
            StatusCode::OK,
            Json(ApiResponse::success(
                serde_json::json!({}),
                "No configuration set for this assignment",
            )),
        )
            .into_response(),
    }
}

/// GET /api/modules/{module_id}/assignments/{assignment_id}/config/default
///
/// Returns the default execution configuration used when no custom config file is present.
/// This helps clients pre-fill configuration forms or understand system defaults.
///
/// ### Success Response (200 OK)
/// ```json
/// {
///   "success": true,
///   "message": "Default execution config retrieved successfully",
///   "data":
/// {
//   "execution": {
//     "timeout_secs": 10,
//     "max_memory": 1000000,
//     "max_cpus": 2,
//     "max_uncompressed_size": 1000000,
//     "max_processes": 256
//   },
//   "marking": {
//     "marking_scheme": "exact",
//     "feedback_scheme": "auto",
//     "deliminator": "&-=-&"
//   }
// }

/// }
/// ```
pub async fn get_default_assignment_config(
    Path((_module_id, _assignment_id)): Path<(i64, i64)>,
) -> impl IntoResponse {
    let default_config = ExecutionConfig::default_config();
    (
        StatusCode::OK,
        Json(ApiResponse::success(
            default_config,
            "Default execution config retrieved successfully",
        )),
    )
}
