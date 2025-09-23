use crate::response::ApiResponse;
use axum::{
    Json,
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use db::models::{
    assignment_file::{FileType, Model as FileModel},
    user::Model as UserModel,
};
use serde::Serialize;
use util::state::AppState;

#[derive(Debug, Serialize)]
pub struct UploadedFileMetadata {
    pub id: i64,
    pub assignment_id: i64,
    pub filename: String,
    pub path: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct AssignmentSubmissionMetadata {
    pub id: i64,
    pub assignment_id: i64,
    pub user_id: i64,
    pub filename: String,
    pub path: String,
    pub created_at: String,
    pub updated_at: String,
}

/// POST /api/modules/{module_id}/assignments/{assignment_id}/files
///
/// Upload a single file to an assignment. Only accessible by lecturers assigned to the module.
///
/// ### Path Parameters
/// - `module_id` (i64): The ID of the module containing the assignment
/// - `assignment_id` (i64): The ID of the assignment to upload the file to
///
/// ### Request Body (Multipart Form Data)
/// - `file_type` (string, required): The type of file. Must be one of: `spec`, `main`, `memo`, etc.
/// - `file` (file, required): The file to upload. Only one file per request is allowed.
///
/// ### Responses
///
/// - `201 Created`
/// ```json
/// {
///   "success": true,
///   "message": "File uploaded successfully",
///   "data": {
///     "id": 123,
///     "assignment_id": 456,
///     "filename": "assignment.pdf",
///     "path": "module_456/assignment_789/assignment.pdf",
///     "created_at": "2024-01-01T00:00:00Z",
///     "updated_at": "2024-01-01T00:00:00Z"
///   }
/// }
/// ```
///
/// - `400 Bad Request`
/// ```json
/// {
///   "success": false,
///   "message": "Invalid file_type" // or "Missing required field: file_type" or "Missing file upload" or "Empty file provided" or "Only one file may be uploaded per request"
/// }
/// ```
///
/// - `404 Not Found`
/// ```json
/// {
///   "success": false,
///   "message": "Assignment not found"
/// }
/// ```
///
/// - `500 Internal Server Error`
/// ```json
/// {
///   "success": false,
///   "message": "Database error" // or "Failed to save file"
/// }
/// ```
///
pub async fn upload_files(
    State(app_state): State<AppState>,
    Path((module_id, assignment_id)): Path<(i64, i64)>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let db = app_state.db();

    let mut file_type: Option<FileType> = None;
    let mut file_name: Option<String> = None;
    let mut file_bytes: Option<Vec<u8>> = None;
    let mut file_count = 0;

    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let name = field.name().unwrap_or("");

        match name {
            "file_type" => {
                if let Ok(ftype_str) = field.text().await {
                    match ftype_str.parse::<FileType>() {
                        Ok(ftype) => file_type = Some(ftype),
                        Err(_) => {
                            return (
                                StatusCode::BAD_REQUEST,
                                Json(ApiResponse::<UploadedFileMetadata>::error(
                                    "Invalid file_type",
                                )),
                            )
                                .into_response();
                        }
                    }
                }
            }
            "file" => {
                if file_count > 0 {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(ApiResponse::<UploadedFileMetadata>::error(
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

    let file_type = match file_type {
        Some(ft) => ft,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<UploadedFileMetadata>::error(
                    "Missing required field: file_type",
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
                Json(ApiResponse::<UploadedFileMetadata>::error(
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
                Json(ApiResponse::<UploadedFileMetadata>::error(
                    "Empty file provided",
                )),
            )
                .into_response();
        }
    };

    match FileModel::save_file(
        db,
        assignment_id,
        module_id,
        file_type.clone(),
        &file_name,
        &file_bytes,
    )
    .await
    {
        Ok(saved) => {
            if file_type == FileType::Spec {
                // fetch emails of all users assigned to module_id
                let email_list = UserModel::get_emails_by_module_id(db, module_id).await;

                // send email notification to all users
                crate::services::email::EmailService::send_email_when_spec_changes(
                        email_list,
                        file_name.clone(),
                        format!("The assignment specification has been updated. Please check the latest version: {}", file_name),
                    )
                    .await;
            }

            let metadata = UploadedFileMetadata {
                id: saved.id,
                assignment_id: saved.assignment_id,
                filename: saved.filename,
                path: saved.path,
                created_at: saved.created_at.to_rfc3339(),
                updated_at: saved.updated_at.to_rfc3339(),
            };
            (
                StatusCode::CREATED,
                Json(ApiResponse::success(metadata, "File uploaded successfully")),
            )
                .into_response()
        }
        Err(e) => {
            eprintln!("File save error: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<UploadedFileMetadata>::error(
                    "Failed to save file",
                )),
            )
                .into_response()
        }
    }
}