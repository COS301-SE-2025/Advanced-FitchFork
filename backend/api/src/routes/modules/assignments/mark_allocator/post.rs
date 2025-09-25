use crate::response::ApiResponse;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use db::models::{assignment_memo_output, assignment_task};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use util::paths::{assignment_dir, memo_output_dir, storage_root};
use util::state::AppState;

use util::mark_allocator::{
    generate_allocator,
    save_allocator,
    // optional: validate_markallocator
};

/// POST /api/modules/{module_id}/assignments/{assignment_id}/mark_allocator
///
/// Generates a **mark allocator** for the assignment by parsing memo output files,
/// then persists it to disk and returns the normalized struct model in the response.
///
/// ### Behavior
/// - Parses each task’s memo output and groups lines by the memo section delimiter
///   from `ExecutionConfig.marking.deliminator` (default: `###`).
/// - Counts non-empty lines per subsection to produce `value`.
/// - **Regex prepopulation:** If `ExecutionConfig.marking.marking_scheme == "regex"`,
///   each subsection’s `regex` field is `Some(Vec<String>)` with one **empty string** per
///   counted output line (e.g., `["", "", ...]`). Otherwise, `regex` is omitted (`None`).
/// - `feedback` is optional and omitted by default (`None`).
/// - Code-coverage tasks are included but do not contribute counts.
///
/// ### Persistence
/// - Saves to `{STORAGE_ROOT}/module_{m}/assignment_{a}/mark_allocator/allocator.json`.
/// - On disk, uses the **legacy** array-of-`{ "taskN": { ... } }` JSON shape to maintain
///   backward compatibility.
/// - The HTTP response returns the **normalized** shape:
///   `{ generated_at, total_value, tasks: [{ task_number, name, value, code_coverage?, subsections: [...] }] }`.
///
/// ### Responses
/// - **200 OK**: `{ "success": true, "message": "Mark allocator successfully generated.", "data": <normalized allocator> }`
/// - **404 Not Found**: assignment folder missing
/// - **500 Internal Server Error**: DB query failures, generation errors, or write failures
///
/// ### Notes
/// - Authorization (e.g., Lecturer role) should be enforced by upstream middleware.
/// - To validate before saving, enable `validate_markallocator` in the commented section below.
pub async fn generate(
    State(app_state): State<AppState>,
    Path((module_id, assignment_id)): Path<(i64, i64)>,
) -> impl IntoResponse {
    let db = app_state.db();

    // 1) Basic existence check → 404 if assignment tree missing
    let assign_path = assignment_dir(module_id, assignment_id);
    if !assign_path.exists() {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error(
                "Module or assignment folder does not exist",
            )),
        )
            .into_response();
    }

    // 2) Load tasks from DB
    let tasks = match assignment_task::Entity::find()
        .filter(assignment_task::Column::AssignmentId.eq(assignment_id))
        .all(db)
        .await
    {
        Ok(ts) => ts,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("Failed to fetch assignment tasks")),
            )
                .into_response();
        }
    };

    // 3) Build (TaskInfo, PathBuf) pairs
    let memo_dir = memo_output_dir(module_id, assignment_id);
    let mut task_file_pairs = Vec::with_capacity(tasks.len());

    for t in &tasks {
        let task_info = util::mark_allocator::TaskInfo {
            id: t.id,
            task_number: t.task_number,
            code_coverage: t.code_coverage,
            valgrind: t.valgrind,
            name: if t.name.trim().is_empty() {
                format!("Task {}", t.task_number)
            } else {
                t.name.clone()
            },
        };

        let memo_path = match assignment_memo_output::Entity::find()
            .filter(assignment_memo_output::Column::AssignmentId.eq(assignment_id))
            .filter(assignment_memo_output::Column::TaskId.eq(t.id))
            .one(db)
            .await
        {
            Ok(Some(m)) => storage_root().join(&m.path),
            Ok(None) => memo_dir.join(format!("no_memo_for_task_{}", t.id)),
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

    // 4) Generate normalized allocator
    let alloc = match generate_allocator(module_id, assignment_id, &task_file_pairs).await {
        Ok(a) => a,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(&format!(
                    "Failed to generate mark allocator: {e}"
                ))),
            )
                .into_response();
        }
    };

    // 5) (Optional) validate
    // if let Err(e) = validate_markallocator(&alloc) {
    //     return (
    //         StatusCode::BAD_REQUEST,
    //         Json(ApiResponse::<()>::error(&format!("Invalid allocator: {e}"))),
    //     ).into_response();
    // }

    // 6) Save in legacy on-disk JSON shape
    if let Err(e) = save_allocator(module_id, assignment_id, &alloc) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(&format!(
                "Failed to persist allocator: {e}"
            ))),
        )
            .into_response();
    }

    // 7) Respond with normalized JSON (clean shape)
    (
        StatusCode::OK,
        Json(ApiResponse::success(
            alloc, // serde will emit the normalized shape to clients
            "Mark allocator successfully generated.",
        )),
    )
        .into_response()
}
