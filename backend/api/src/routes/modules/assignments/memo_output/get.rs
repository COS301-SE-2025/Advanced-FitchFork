use crate::response::ApiResponse;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use db::models::{assignment_memo_output, assignment_task};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::Serialize;
use tokio::fs as tokio_fs;
use util::{execution_config::ExecutionConfig, paths::storage_root, state::AppState};

#[derive(Serialize)]
struct MemoSubsection {
    label: String,
    output: String,
}

#[derive(Serialize)]
struct MemoTaskOutput {
    task_number: i64,
    name: String,
    subsections: Vec<MemoSubsection>,
    raw: String,
}

/// GET /api/modules/{module_id}/assignments/{assignment_id}/memo_output
///
/// Retrieve all memo output files for a given assignment, parsed into structured format.
///
/// Scans the `memo_output` directory for the specified assignment and parses each file into a
/// `MemoTaskOutput` object, which contains labeled subsections and the raw file content.
///
/// **Path Parameters**
/// - `module_id` (i64): The ID of the module
/// - `assignment_id` (i64): The ID of the assignment
///
/// **Success Response (200 OK)**
/// ```json
/// [
///   {
///     "task_number": 1,
///     "name": "Task 1",
///     "subsections": [
///       { "label": "Section A", "output": "..." }
///     ],
///     "raw": "..."
///   }
/// ]
/// ```
///
/// **Error Responses**
/// - `404 Not Found` if the memo output directory does not exist or contains no valid files
/// - `500 Internal Server Error` if reading the directory fails
///
/// **Example Request**
/// ```bash
/// curl http://localhost:3000/api/modules/1/assignments/2/memo_output
/// ```
pub async fn get_all_memo_outputs(
    State(app_state): State<AppState>,
    Path((module_id, assignment_id)): Path<(i64, i64)>,
) -> impl IntoResponse {
    let db = app_state.db();

    // Fetch all memo output models for the given assignment
    let memo_outputs = match assignment_memo_output::Entity::find()
        .filter(assignment_memo_output::Column::AssignmentId.eq(assignment_id))
        .all(db)
        .await
    {
        Ok(models) if !models.is_empty() => models,
        Ok(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<Vec<MemoTaskOutput>>::error(
                    "No memo output records found",
                )),
            );
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<Vec<MemoTaskOutput>>::error(
                    "Failed to query memo outputs",
                )),
            );
        }
    };

    // Load separator from execution config (once)
    let separator = match ExecutionConfig::get_execution_config(module_id, assignment_id) {
        Ok(config) => config.marking.deliminator,
        Err(_) => "###".to_string(),
    };

    let mut results = Vec::new();

    for memo in memo_outputs {
        let full_path = storage_root().join(&memo.path);
        if !full_path.is_file() {
            continue;
        }

        let raw_content = match tokio_fs::read_to_string(&full_path).await {
            Ok(c) => c,
            Err(_) => continue,
        };

        // Parse output
        let sections = raw_content
            .split(&separator)
            .filter(|s| !s.trim().is_empty());

        let subsections = sections
            .map(|s| {
                let mut lines = s.lines();
                let label = lines.next().unwrap_or("").trim().to_string();
                let output = lines.collect::<Vec<_>>().join("\n");
                MemoSubsection { label, output }
            })
            .collect::<Vec<_>>();

        // Lookup the task number and name
        let Some(task) = assignment_task::Entity::find_by_id(memo.task_id)
            .filter(assignment_task::Column::AssignmentId.eq(assignment_id))
            .one(db)
            .await
            .ok()
            .flatten()
        else {
            continue;
        };

        results.push(MemoTaskOutput {
            task_number: task.task_number,
            name: task.name,
            subsections,
            raw: raw_content,
        });
    }

    if results.is_empty() {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<Vec<MemoTaskOutput>>::error(
                "No valid memo output files found",
            )),
        );
    }

    (
        StatusCode::OK,
        Json(ApiResponse::success(
            results,
            "Fetched memo output successfully",
        )),
    )
}
