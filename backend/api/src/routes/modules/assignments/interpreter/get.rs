use crate::response::ApiResponse;
use axum::{
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Json, Response},
};
use db::models::assignment_interpreter::{
    Column as InterpreterColumn, Entity as InterpreterEntity,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use std::{env, path::PathBuf};
use tokio::{fs::File as FsFile, io::AsyncReadExt};
use util::state::AppState;

/// GET /api/modules/{module_id}/assignments/{assignment_id}/interpreter
///
/// Download the interpreter file for an assignment. Only one interpreter may exist per assignment.
///
/// ### Path Parameters
/// - `module_id` (i64): The ID of the module containing the assignment
/// - `assignment_id` (i64): The ID of the assignment containing the interpreter
///
/// ### Responses
/// - `200 OK`: Returns the interpreter as a binary attachment
/// - `404 Not Found`: If no interpreter exists for the assignment, or if the file is missing on disk
/// - `500 Internal Server Error`: If DB or file read fails
///
pub async fn download_interpreter(
    State(app_state): State<AppState>,
    Path((_module_id, assignment_id)): Path<(i64, i64)>,
) -> Response {
    let db = app_state.db();

    // There should be at most one interpreter per assignment
    let interpreter = match InterpreterEntity::find()
        .filter(InterpreterColumn::AssignmentId.eq(assignment_id))
        .one(db)
        .await
    {
        Ok(Some(interpreter)) => interpreter,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<()>::error("Interpreter not found")),
            )
                .into_response();
        }
        Err(err) => {
            eprintln!("DB error fetching interpreter: {:?}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("Database error")),
            )
                .into_response();
        }
    };

    let storage_root =
        env::var("ASSIGNMENT_STORAGE_ROOT").unwrap_or_else(|_| "data/interpreters".to_string());
    let fs_path = PathBuf::from(storage_root).join(&interpreter.path);

    if tokio::fs::metadata(&fs_path).await.is_err() {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Interpreter file missing on disk")),
        )
            .into_response();
    }

    let mut file_handle = match FsFile::open(&fs_path).await {
        Ok(f) => f,
        Err(err) => {
            eprintln!("File open error: {:?}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("Could not open interpreter file")),
            )
                .into_response();
        }
    };

    let mut buffer = Vec::new();
    if let Err(err) = file_handle.read_to_end(&mut buffer).await {
        eprintln!("File read error: {:?}", err);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error("Failed to read interpreter file")),
        )
            .into_response();
    }

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!(
            "attachment; filename=\"{}\"",
            interpreter.filename
        ))
        .unwrap_or_else(|_| HeaderValue::from_static("attachment")),
    );
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/octet-stream"),
    );

    (StatusCode::OK, headers, buffer).into_response()
}
