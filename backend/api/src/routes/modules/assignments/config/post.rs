use db::models::assignment_file::{FileType, Model as AssignmentFile};
use axum::{
    extract::{State, Json, Path},
    http::StatusCode,
    response::IntoResponse,
};
use sea_orm::{EntityTrait, ColumnTrait, QueryFilter};
use serde_json::Value;
use crate::response::ApiResponse;
use db::models::assignment::{Column as AssignmentColumn, Entity as AssignmentEntity};
use util::{execution_config::ExecutionConfig, state::AppState};


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
/// - Configuration is saved to disk.
/// - Only valid `ExecutionConfig` objects are accepted.
// api route
pub async fn set_assignment_config(
    State(app_state): State<AppState>,
    Path((module_id, assignment_id)): Path<(i64, i64)>,
    Json(config_json): Json<Value>,
) -> impl IntoResponse {
    let db = app_state.db();

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

    // Ensure assignment exists
    if let Err(resp) = AssignmentEntity::find()
        .filter(AssignmentColumn::Id.eq(assignment_id as i32))
        .filter(AssignmentColumn::ModuleId.eq(module_id as i32))
        .one(db)
        .await
        .map_err(|e| {
            eprintln!("DB error: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("Database error")),
            )
        })
        .and_then(|opt| {
            opt.ok_or((
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<()>::error("Assignment or module not found")),
            ))
        })
    {
        return resp;
    }

    // Serialize and overwrite-in-place (handled inside save_file)
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

    match AssignmentFile::save_file(
        &db,
        assignment_id,
        module_id,
        FileType::Config,
        "config.json",
        &bytes,
    )
    .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse::success((), "Assignment configuration saved")),
        ),
        Err(e) => {
            eprintln!("File save error: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("Failed to save config")),
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
    State(app_state): State<AppState>,
    Path((module_id, assignment_id)): Path<(i64, i64)>,
) -> impl IntoResponse {
    let db = app_state.db();

    // Ensure the assignment exists and belongs to the module
    let assignment = match AssignmentEntity::find()
        .filter(AssignmentColumn::Id.eq(assignment_id as i32))
        .filter(AssignmentColumn::ModuleId.eq(module_id as i32))
        .one(db)
        .await
    {
        Ok(Some(a)) => a,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<ExecutionConfig>::error("Assignment or module not found")),
            );
        }
        Err(e) => {
            eprintln!("DB error: {:?}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<ExecutionConfig>::error("Database error")),
            );
        }
    };

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

    match AssignmentFile::save_file(
        &db,
        assignment.id.into(),
        module_id,
        FileType::Config,
        "config.json",
        &bytes,
    )
    .await
    {
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