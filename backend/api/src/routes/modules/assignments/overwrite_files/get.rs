use axum::{
    extract::{Path, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use db::models::assignment_overwrite_file::{
    Entity as OverwriteFileEntity, Model as OverwriteFileModel,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use util::state::AppState;

/// GET /api/modules/{module_id}/assignments/{assignment_id}/overwrite_files/task/{task_number}
///
/// Returns the first overwrite file for the task as a downloadable file
pub async fn get_task_overwrite_files(
    State(app_state): State<AppState>,
    Path((_module_id, assignment_id, task_number)): Path<(i64, i64, i64)>,
) -> impl IntoResponse {
    let db = app_state.db();

    let file: Option<OverwriteFileModel> = match OverwriteFileEntity::find()
        .filter(db::models::assignment_overwrite_file::Column::AssignmentId.eq(assignment_id))
        .filter(db::models::assignment_overwrite_file::Column::TaskId.eq(task_number))
        .order_by_desc(db::models::assignment_overwrite_file::Column::CreatedAt)
        .one(db)
        .await
    {
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

    let contents = match file.load_file() {
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
