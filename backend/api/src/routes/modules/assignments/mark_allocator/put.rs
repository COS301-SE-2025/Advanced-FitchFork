use crate::response::ApiResponse;
use axum::{Json, extract::Path, http::StatusCode, response::IntoResponse};
use util::execution_config::{ExecutionConfig, MarkingScheme};
use util::paths::assignment_dir;

use util::mark_allocator::{MarkAllocator, save_allocator, load_allocator};

/// PUT /api/modules/{module_id}/assignments/{assignment_id}/mark_allocator
///
/// Save a **normalized** mark allocator for an assignment.
/// - Body must be a `MarkAllocator` JSON (normalized shape).
/// - Validates:
///   - `tasks` non-empty
///   - each task: `task_number > 0`, `name != ""`, `value >= 0`
///   - each subsection: `name != ""`, `value >= 0`
///   - sum(subsection.value) == task.value for every task
///   - sum(task.value) == total_value
/// - If `ExecutionConfig.marking.marking_scheme == "regex"`:
///   - For every subsection, ensures `regex.len() == subsection.value`.
///   - If `regex` is missing → **pad** with `""` to length = `value`.
///   - If `regex` exists but shorter → **pad** with `""`.
///   - If `regex` longer than `value` → **400 Bad Request**.
/// - Persists **normalized JSON** at:
///   `{STORAGE_ROOT}/module_{m}/assignment_{a}/mark_allocator/allocator.json`
/// - Responds with the normalized allocator.
pub async fn save(
    Path((module_id, assignment_id)): Path<(i64, i64)>,
    Json(mut alloc): Json<MarkAllocator>,
) -> impl IntoResponse {
    // 0) Basic existence check
    let assign_path = assignment_dir(module_id, assignment_id);
    if !assign_path.exists() {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error(
                "Module or assignment directory does not exist",
            )),
        )
            .into_response();
    }

    // 1) Structural validation + per-task/per-subsection checks
    if alloc.tasks.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error("`tasks` must not be empty")),
        )
            .into_response();
    }

    let mut sum_task_values: f64 = 0.0;
    for (tidx, t) in alloc.tasks.iter().enumerate() {
        if t.task_number <= 0 {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<()>::error(&format!(
                    "tasks[{}].task_number must be > 0",
                    tidx
                ))),
            )
                .into_response();
        }
        if t.name.trim().is_empty() {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<()>::error(&format!(
                    "tasks[{}].name must be a non-empty string",
                    tidx
                ))),
            )
                .into_response();
        }
        if t.value < 0.0 {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<()>::error(&format!(
                    "tasks[{}].value must be >= 0",
                    tidx
                ))),
            )
                .into_response();
        }

        let mut sum_sub_values: f64 = 0.0;
        for (sidx, s) in t.subsections.iter().enumerate() {
            if s.name.trim().is_empty() {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::<()>::error(&format!(
                        "tasks[{}].subsections[{}].name must be a non-empty string",
                        tidx, sidx
                    ))),
                )
                    .into_response();
            }
            if s.value < 0.0 {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::<()>::error(&format!(
                        "tasks[{}].subsections[{}].value must be >= 0",
                        tidx, sidx
                    ))),
                )
                    .into_response();
            }
            sum_sub_values += s.value;
        }
        
        if !t.code_coverage.unwrap_or(false) && sum_sub_values != t.value {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<()>::error(&format!(
                    "tasks[{}]: sum of subsection values ({}) must equal task value ({})",
                    tidx, sum_sub_values, t.value
                ))),
            )
                .into_response();
        }

        sum_task_values += t.value;
    }

    if sum_task_values != alloc.total_value {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(&format!(
                "sum of task values ({}) must equal total_value ({})",
                sum_task_values, alloc.total_value
            ))),
        )
            .into_response();
    }

    // 2) Load existing allocator to preserve code coverage task values
    let existing_alloc = load_allocator(module_id, assignment_id);
    
    // 3) Preserve code coverage task values from existing allocator
    if let Ok(existing) = existing_alloc {
        let mut coverage_values: std::collections::HashMap<i64, f64> = std::collections::HashMap::new();
        for task in &existing.tasks {
            if task.code_coverage.unwrap_or(false) {
                coverage_values.insert(task.task_number, task.value);
            }
        }
        
        for task in &mut alloc.tasks {
            if task.code_coverage.unwrap_or(false) {
                if let Some(&existing_value) = coverage_values.get(&task.task_number) {
                    task.value = existing_value;
                }
            }
        }

        alloc.total_value = alloc.tasks.iter().map(|t| t.value).sum();
    }

    // 4) Read marking scheme to decide regex behavior
    let want_regex = match ExecutionConfig::get_execution_config(module_id, assignment_id) {
        Ok(cfg) => matches!(cfg.marking.marking_scheme, MarkingScheme::Regex),
        Err(_) => false,
    };

    // 5) If Regex scheme, accept regex arrays as-is (independent of value)
    if want_regex {
        for (_tidx, t) in alloc.tasks.iter_mut().enumerate() {
            for (_sidx, s) in t.subsections.iter_mut().enumerate() {
                match s.regex.as_mut() {
                    Some(v) => {
                        // normalize: ensure strings (empty "" is allowed)
                        for r in v.iter_mut() {
                            // nothing to do; keep user-provided string
                            // (you could trim whitespace if you want)
                            let _ = r;
                        }
                    }
                    None => {
                        // if omitted, store empty vec (not tied to marks)
                        s.regex = Some(Vec::new());
                    }
                }
            }
        }
    }

    // 6) Save normalized JSON
    if let Err(e) = save_allocator(module_id, assignment_id, &alloc) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(&format!(
                "Failed to persist allocator: {e}"
            ))),
        )
            .into_response();
    }

    // 7) Respond with normalized object
    (
        StatusCode::OK,
        Json(ApiResponse::success(
            alloc,
            "Mark allocator successfully saved.",
        )),
    )
        .into_response()
}
