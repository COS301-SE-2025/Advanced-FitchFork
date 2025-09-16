use axum::{
    extract::{Json, Path},
    http::StatusCode,
    response::IntoResponse,
};
use serde_json::Value;
use crate::response::ApiResponse;
use util::execution_config::ExecutionConfig;
use services::service::Service;
use services::assignment_file::{AssignmentFileService, CreateAssignmentFile};

/// POST /api/modules/{module_id}/assignments/{assignment_id}/config
///
/// Save or replace the JSON execution configuration object for a specific assignment.
///
/// Accessible to users with Lecturer or Admin roles assigned to the module. The config is persisted
/// to disk as a JSON file under the module's assignment directory. This currently uses the
/// [`ExecutionConfig`] structure, which will be expanded in the future.
///
/// ### Path Parameters
/// - `module_id` (i64): The ID of the module
/// - `assignment_id` (i64): The ID of the assignment
///
/// ### Request Body
/// A JSON object matching the shape of `ExecutionConfig`:
/// ```json
/// {
///   "execution": {
///     "timeout_secs": 10,
///     "max_cpus": 2,
///     "max_processes": 256
///   },
///   "marking": {
///     "marking_scheme": "exact",
///     "feedback_scheme": "auto",
///     "deliminator": "&-=-&"
///   }
/// }
/// ```
///
/// ### Success Response (200 OK)
/// ```json
/// {
///   "success": true,
///   "message": "Assignment configuration saved",
///   "data": null
/// }
/// ```
///
/// ### Error Responses
/// - **400** – Invalid JSON structure
/// - **404** – Assignment not found
/// - **500** – Internal error saving the file
///
/// ### Notes
/// - Configuration is saved to disk under `ASSIGNMENT_STORAGE_ROOT/module_{id}/assignment_{id}/config/config.json`.
/// - Only valid `ExecutionConfig` objects are accepted.
pub async fn set_assignment_config(
    Path((module_id, assignment_id)): Path<(i64, i64)>,
    Json(config_json): Json<Value>,
) -> impl IntoResponse {
    if !config_json.is_object() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error("Configuration must be a JSON object")),
        );
    }

    let config: ExecutionConfig = match serde_json::from_value(config_json) {
        Ok(cfg) => cfg,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<()>::error(format!("Invalid config format: {}", e))),
            );
        }
    };

    // Save as assignment file using `save_file`
    let bytes = match serde_json::to_vec_pretty(&config) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Serialization error: {:?}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("Failed to serialize config")),
            );
        }
    };

    match AssignmentFileService::create(
        CreateAssignmentFile {
            assignment_id: assignment_id,
            module_id,
            file_type: "config".to_string(),
            filename: "config.json".to_string(),
            bytes,
        }
    ).await {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse::success((), "Assignment configuration saved")),
        ),
        Err(e) => {
            eprintln!("File save error: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("Failed to save config as assignment file")),
            )
        }
    }
}


/// POST /api/modules/{module_id}/assignments/{assignment_id}/config/reset
///
/// Overwrite the assignment's config on disk with the system defaults (`ExecutionConfig::default_config()`).
/// Returns the default config that was saved.
///
/// ### Success Response (200 OK)
/// ```json
/// {
///   "success": true,
///   "message": "Assignment configuration reset to defaults",
///   "data": { ... full ExecutionConfig ... }
/// }
/// ```
pub async fn reset_assignment_config(
    Path((module_id, assignment_id)): Path<(i64, i64)>,
) -> impl IntoResponse {
    // Build defaults
    let default_cfg = ExecutionConfig::default_config();

    // Persist file
    let bytes = match serde_json::to_vec_pretty(&default_cfg) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Serialization error: {:?}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<ExecutionConfig>::error("Failed to serialize default config")),
            );
        }
    };

    match AssignmentFileService::create(
        CreateAssignmentFile {
            assignment_id: assignment_id,
            module_id,
            file_type: "config".to_string(),
            filename: "config.json".to_string(),
            bytes,
        }
    ).await {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse::<ExecutionConfig>::success(
                default_cfg,
                "Assignment configuration reset to defaults",
            )),
        ),
        Err(e) => {
            eprintln!("File save error: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<ExecutionConfig>::error(
                    "Failed to save default config as assignment file",
                )),
            )
        }
    }
}