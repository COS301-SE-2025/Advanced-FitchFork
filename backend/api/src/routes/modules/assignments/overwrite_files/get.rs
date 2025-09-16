use axum::{
    extract::Path,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use util::filters::FilterParam;
use services::service::Service;
use services::assignment_overwrite_file::AssignmentOverwriteFileService;

/// GET /api/modules/{module_id}/assignments/{assignment_id}/overwrite_files/task/{task_id}
///
/// Returns the first overwrite file for the task as a downloadable file
pub async fn get_task_overwrite_files(
    Path((_, assignment_id, task_id)): Path<(i64, i64, i64)>,
) -> impl IntoResponse {
    let file = match AssignmentOverwriteFileService::find_one(
        &vec![
            FilterParam::eq("assignment_id", assignment_id),
            FilterParam::eq("task_id", task_id),
        ],
        &vec![],
        Some("-created_at".to_string()),
    ).await {
        Ok(f) => f,
        Err(e) => {
            eprintln!("DB error fetching overwrite files: {:?}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error fetching overwrite files",
            )
                .into_response();
        }
    };

    let file = match file {
        Some(f) => f,
        None => return (StatusCode::NOT_FOUND, "Overwrite file not found").into_response(),
    };

    let contents = match AssignmentOverwriteFileService::load_file(file.id).await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to read file from disk: {:?}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to read overwrite file",
            )
                .into_response();
        }
    };

    Response::builder()
        .status(StatusCode::OK)
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", file.filename),
        )
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .body(contents.into())
        .unwrap()
}
