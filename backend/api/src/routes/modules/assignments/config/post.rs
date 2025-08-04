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
/// - Configuration is saved to disk under `ASSIGNMENT_STORAGE_ROOT/module_{id}/assignment_{id}/config/config.json`.
/// - Only valid `ExecutionConfig` objects are accepted.
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

    // Check assignment existence
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
                Json(ApiResponse::<()>::error("Assignment or module not found")),
            );
        }
        Err(e) => {
            eprintln!("DB error: {:?}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("Database error")),
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
