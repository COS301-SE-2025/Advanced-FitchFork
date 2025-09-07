use crate::response::ApiResponse;
use axum::{
    Json,
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use db::models::assignment_interpreter::Model as InterpreterModel;
use serde::Serialize;
use util::state::AppState;

#[derive(Debug, Serialize)]
pub struct UploadedInterpreterMetadata {
    pub id: i64,
    pub assignment_id: i64,
    pub filename: String,
    pub path: String,
    pub command: String,
    pub created_at: String,
    pub updated_at: String,
}

/// POST /api/modules/{module_id}/assignments/{assignment_id}/interpreter
///
/// Upload an interpreter file for an assignment. Only one interpreter may exist per assignment.
/// Existing interpreters with the same filename will be overwritten.
///
/// ### Path Parameters
/// - `module_id` (i64): The ID of the module containing the assignment
/// - `assignment_id` (i64): The ID of the assignment to upload the interpreter for
///
/// ### Request Body (Multipart Form Data)
/// - `command` (string, required): The command to execute the interpreter (e.g., "python3 main.py")
/// - `file` (file, required): The interpreter file to upload
///
/// ### Responses
/// - `201 Created` → success with metadata
/// - `400 Bad Request` → missing command/file, empty file, multiple files, etc.
/// - `500 Internal Server Error` → database or file write errors
///
pub async fn upload_interpreter(
    State(app_state): State<AppState>,
    Path((module_id, assignment_id)): Path<(i64, i64)>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let db = app_state.db();

    let mut command: Option<String> = None;
    let mut file_name: Option<String> = None;
    let mut file_bytes: Option<Vec<u8>> = None;
    let mut file_count = 0;

    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let name = field.name().unwrap_or("");

        match name {
            "command" => {
                if let Ok(cmd) = field.text().await {
                    command = Some(cmd);
                }
            }
            "file" => {
                if file_count > 0 {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(ApiResponse::<UploadedInterpreterMetadata>::error(
                            "Only one file may be uploaded per request",
                        )),
                    )
                        .into_response();
                }
                file_name = field.file_name().map(|s| s.to_string());
                file_bytes = Some(field.bytes().await.unwrap_or_default().to_vec());
                file_count += 1;
            }
            _ => continue,
        }
    }

    let command = match command {
        Some(c) if !c.trim().is_empty() => c,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<UploadedInterpreterMetadata>::error(
                    "Missing required field: command",
                )),
            )
                .into_response();
        }
    };

    let file_name = match file_name {
        Some(name) => name,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<UploadedInterpreterMetadata>::error(
                    "Missing file upload",
                )),
            )
                .into_response();
        }
    };

    let file_bytes = match file_bytes {
        Some(bytes) if !bytes.is_empty() => bytes,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<UploadedInterpreterMetadata>::error(
                    "Empty file provided",
                )),
            )
                .into_response();
        }
    };

    match InterpreterModel::save_file(
        db,
        assignment_id,
        module_id,
        &file_name,
        &command,
        &file_bytes,
    )
    .await
    {
        Ok(saved) => {
            let metadata = UploadedInterpreterMetadata {
                id: saved.id,
                assignment_id: saved.assignment_id,
                filename: saved.filename,
                path: saved.path,
                command: saved.command,
                created_at: saved.created_at.to_rfc3339(),
                updated_at: saved.updated_at.to_rfc3339(),
            };
            (
                StatusCode::CREATED,
                Json(ApiResponse::success(
                    metadata,
                    "Interpreter uploaded successfully",
                )),
            )
                .into_response()
        }
        Err(e) => {
            eprintln!("Interpreter save error: {:?}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<UploadedInterpreterMetadata>::error(
                    "Failed to save interpreter",
                )),
            )
                .into_response();
        }
    }
}
