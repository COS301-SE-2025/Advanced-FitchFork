use crate::response::ApiResponse;
use axum::{
    Json,
    body::Bytes,
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::IntoResponse,
};

use db::models::assignment_overwrite_file::Model as OverwriteFileModel;
use util::state::AppState;

/// POST /api/modules/{module_id}/assignments/{assignment_id}/overwrite_files/task/{task_number}
///
/// Upload one or more overwrite files for a specific task in an assignment.
///
/// ### Path Parameters
/// - `module_id` (i64): Module ID
/// - `assignment_id` (i64): Assignment ID
/// - `task_number` (i64): Task number within the assignment
pub async fn post_task_overwrite_files(
    State(app_state): State<AppState>,
    Path((_module_id, assignment_id, task_number)): Path<(i64, i64, i64)>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let db = app_state.db();

    let mut saved_files = Vec::new();

    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let file_name = field
            .file_name()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "file".into());
        let bytes = field.bytes().await.unwrap_or_else(|_| Bytes::new());

        match OverwriteFileModel::save_file(db, assignment_id, task_number, &file_name, &bytes)
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
