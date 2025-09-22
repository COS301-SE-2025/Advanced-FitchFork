use crate::response::ApiResponse;
use axum::{Json, extract::Path, http::StatusCode, response::IntoResponse};
use services::assignment_overwrite_file::AssignmentOverwriteFileService;
use services::service::Service;
use util::filters::FilterParam;

/// DELETE /api/modules/{module_id}/assignments/{assignment_id}/overwrite_files/task/{task_id}
///
/// Delete all overwrite files associated with a specific task in an assignment.
///
/// ### Path Parameters
/// - `module_id` (i64): Module ID
/// - `assignment_id` (i64): Assignment ID
/// - `task_id` (i64): Task id within the assignment
pub async fn delete_task_overwrite_files(
    Path((_, assignment_id, task_id)): Path<(i64, i64, i64)>,
) -> impl IntoResponse {
    let files = match AssignmentOverwriteFileService::find_all(
        &vec![
            FilterParam::eq("assignment_id", assignment_id),
            FilterParam::eq("task_id", task_id),
        ],
        &vec![],
        None,
    )
    .await
    {
        Ok(f) => f,
        Err(e) => {
            eprintln!("DB error fetching overwrite files: {:?}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(
                    "Database error fetching overwrite files",
                )),
            )
                .into_response();
        }
    };

    if files.is_empty() {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error(
                "No overwrite files found for this task",
            )),
        )
            .into_response();
    }

    for file in files {
        if let Err(e) = AssignmentOverwriteFileService::delete_file_only(file.id).await {
            eprintln!("Failed to delete file from disk: {:?}", e);
        }
        if let Err(e) = AssignmentOverwriteFileService::delete_by_id(file.id).await {
            eprintln!("DB error deleting overwrite file: {:?}", e);
        }
    }

    (
        StatusCode::OK,
        Json(ApiResponse::<()>::success(
            (),
            "All overwrite files for the task deleted successfully",
        )),
    )
        .into_response()
}
