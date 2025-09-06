use crate::response::ApiResponse;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use db::models::assignment_overwrite_file::{
    ActiveModel as OverwriteFileActiveModel, Column as OverwriteFileColumn,
    Entity as OverwriteFileEntity, Model as OverwriteFileModel,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};
use util::state::AppState;

/// DELETE /api/modules/{module_id}/assignments/{assignment_id}/overwrite_files/task/{task_number}
///
/// Delete all overwrite files associated with a specific task in an assignment.
///
/// ### Path Parameters
/// - `module_id` (i64): Module ID
/// - `assignment_id` (i64): Assignment ID
/// - `task_number` (i64): Task number within the assignment
pub async fn delete_task_overwrite_files(
    State(app_state): State<AppState>,
    Path((_module_id, assignment_id, task_number)): Path<(i64, i64, i64)>,
) -> impl IntoResponse {
    let db = app_state.db();

    let files: Vec<OverwriteFileModel> = match OverwriteFileEntity::find()
        .filter(OverwriteFileColumn::AssignmentId.eq(assignment_id))
        .filter(OverwriteFileColumn::TaskId.eq(task_number))
        .all(db)
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
        if let Err(e) = file.delete_file_only() {
            eprintln!("Failed to delete file from disk: {:?}", e);
        }

        let active: OverwriteFileActiveModel = file.into();
        if let Err(e) = active.delete(db).await {
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
