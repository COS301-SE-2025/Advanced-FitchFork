use std::{env, path::PathBuf, fs};
use axum::{
    extract::Path,
    http::StatusCode,
    Json,
};
use tracing::{error, info};

use crate::response::ApiResponse;
use db::connect;
use code_runner::create_memo_outputs_for_all_tasks;

/// Starts asynchronous generation of memo outputs for all tasks in the specified assignment.
///
/// Validates the presence of the `memo` and `config` directories under:
/// `ASSIGNMENT_STORAGE_ROOT/module_{module_id}/assignment_{assignment_id}`
///
/// ### Returns:
/// - `202 Accepted` if validation passes and the background task is spawned
/// - `422 Unprocessable Entity` if either:
///     - `memo/` directory is missing or empty
///     - `config/` directory is missing or empty
///
/// The actual memo generation is performed in a background task and does not block the response.
///
/// ### Example `curl` request:
/// ```bash
/// curl -X POST http://localhost:3000/modules/1/assignments/2/memo/generate
/// ```
pub async fn generate_memo_output(
    Path((module_id, assignment_id)): Path<(i64, i64)>,
) -> (StatusCode, Json<ApiResponse<()>>) {
    let base_path = env::var("ASSIGNMENT_STORAGE_ROOT")
        .unwrap_or_else(|_| "data/assignment_files".into());

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
            Json(ApiResponse::<()>::error("Required memo directory is missing or empty")),
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

    let db = connect().await;

    match create_memo_outputs_for_all_tasks(&db, assignment_id).await {
        Ok(_) => {
            info!("Memo output generation complete for assignment {}", assignment_id);
            (
                StatusCode::OK,
                Json(ApiResponse::<()>::success((), "Memo output generation complete")),
            )
        }
        Err(e) => {
            error!("Memo output generation failed for assignment {}: {:?}", assignment_id, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("Failed to generate memo output")),
            )
        }
    }
}