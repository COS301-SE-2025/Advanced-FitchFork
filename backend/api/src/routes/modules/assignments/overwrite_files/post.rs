use crate::response::ApiResponse;
use axum::{
    Json,
    body::Bytes,
    extract::{Multipart, Path},
    http::StatusCode,
    response::IntoResponse,
};
use services::assignment_overwrite_file::{
    AssignmentOverwriteFileService, CreateAssignmentOverwriteFile,
};
use services::service::Service;

/// POST /api/modules/{module_id}/assignments/{assignment_id}/overwrite_files/task/{task_id}
///
/// Upload one or more overwrite files for a specific task in an assignment.
///
/// ### Path Parameters
/// - `module_id` (i64): Module ID
/// - `assignment_id` (i64): Assignment ID
/// - `task_id` (i64): Task id within the assignment
pub async fn post_task_overwrite_files(
    Path((_module_id, assignment_id, task_id)): Path<(i64, i64, i64)>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut saved_files = Vec::new();

    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let file_name = field
            .file_name()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "file".into());
        let bytes = field.bytes().await.unwrap_or_else(|_| Bytes::new());

        match AssignmentOverwriteFileService::create(CreateAssignmentOverwriteFile {
            assignment_id,
            task_id,
            filename: file_name,
            bytes: bytes.to_vec(),
        })
        .await
        {
            Ok(file_model) => saved_files.push(file_model.filename),
            Err(e) => {
                eprintln!("Failed to save overwrite file: {:?}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()>::error("Failed to save overwrite file")),
                )
                    .into_response();
            }
        }
    }

    (
        StatusCode::OK,
        Json(ApiResponse::success(
            saved_files,
            "Overwrite files uploaded successfully",
        )),
    )
        .into_response()
}
