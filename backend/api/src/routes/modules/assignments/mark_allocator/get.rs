use crate::response::ApiResponse;
use axum::{extract::Path, http::StatusCode, response::IntoResponse, Json};
use util::mark_allocator::load_allocator;
use util::paths::{assignment_dir, mark_allocator_path};

/// GET /api/modules/{module_id}/assignments/{assignment_id}/mark_allocator
///
/// Load the **normalized** mark allocator for an assignment.
/// Returns:
/// ```json
/// {
///   "success": true,
///   "message": "Mark allocator successfully loaded.",
///   "data": {
///     "generated_at": "2025-09-22T15:08:51.377Z",
///     "total_value": 37,
///     "tasks": [
///       {
///         "task_number": 1,
///         "name": "Core list operations",
///         "value": 12,
///         "code_coverage": false,
///         "subsections": [
///           { "name": "empty-list", "value": 1, "regex": [""], "feedback": "" },
///           { "name": "push_front_back", "value": 1, "regex": ["", "" ] }
///         ]
///       }
///     ]
///   }
/// }
/// ```
///
/// Notes:
/// - This returns the **normalized** structure: `{ generated_at, total_value, tasks[] }`.
/// - Each task contains `subsections[]` with `{ name, value, feedback?, regex? }`.
/// - If the assignment’s marking scheme is **Regex**, the generator will have
///   created `regex` arrays where each element corresponds to a line within that subsection.
/// - File location: `{STORAGE_ROOT}/module_{m}/assignment_{a}/mark_allocator/allocator.json`.
///
/// Errors:
/// - **404 Not Found**: assignment folder or allocator file is missing
/// - **500 Internal Server Error**: allocator exists but failed to parse/read
pub async fn load(Path((module_id, assignment_id)): Path<(i64, i64)>) -> impl IntoResponse {
    // Quick existence checks for clearer 404s
    let assign_path = assignment_dir(module_id, assignment_id);
    if !assign_path.exists() {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Module or assignment folder does not exist")),
        )
            .into_response();
    }
    let alloc_path = mark_allocator_path(module_id, assignment_id);
    if !alloc_path.exists() {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Mark allocator file not found for this assignment")),
        )
            .into_response();
    }

    match load_allocator(module_id, assignment_id) {
        Ok(alloc) => (
            StatusCode::OK,
            Json(ApiResponse::success(
                alloc,
                "Mark allocator successfully loaded.",
            )),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(&format!("Failed to load allocator: {e}"))),
        )
            .into_response(),
    }
}
