
use axum::{
    extract::{Extension, Multipart, Path},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use chrono::{DateTime, Utc};

use serde::{Deserialize, Serialize};

use sea_orm::{ColumnTrait, DbErr, EntityTrait, QueryFilter};

use db::{
    connect,
    models::{
        assignment::{
            AssignmentType, Column as AssignmentColumn, Entity as AssignmentEntity,
            Model as AssignmentModel,
        },
        assignment_file::{FileType, Model as FileModel},
        assignment_submission::Model as AssignmentSubmissionModel,
    },
};

use crate::auth::AuthUser;
use crate::response::ApiResponse;

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


// POST /api/modules/:module_id/assignments/:assignment_id/submissions
// 
// Upload a submission file for a given assignment (Students only).
// 
// ### Multipart Body (form-data)
// - file (single file, must be .tgz, .gz, .tar, or .zip)
// 
// ### Example curl
// ```bash
// curl -X POST http://localhost:3000/api/modules/1/assignments/2/submissions \
//   -H "Authorization: Bearer <token>" \
//   -F "file=@solution.zip"
// ```
// 
// ### Responses
// - `201 Created` with submission metadata
// - `400 Bad Request` (no file, invalid format, or unsupported file type)
// - `403 Forbidden` (unauthorized)
// - `404 Not Found` (assignment/module not found)
// 
// pub async fn submit_assignment(
//     Path((module_id, assignment_id)): Path<(i64, i64)>,
//     Extension(AuthUser(claims)): Extension<AuthUser>,
//     mut multipart: Multipart,
// ) -> impl IntoResponse {
//     let db = connect().await;
// 
//     let assignment_exists = AssignmentEntity::find()
//         .filter(AssignmentColumn::Id.eq(assignment_id as i32))
//         .filter(AssignmentColumn::ModuleId.eq(module_id as i32))
//         .one(&db)
//         .await;
// 
//     match assignment_exists {
//         Ok(Some(_)) => {}
//         Ok(None) => {
//             return (
//                 StatusCode::NOT_FOUND,
//                 Json(ApiResponse::<UploadedFileMetadata>::error(
//                     "Assignment not found",
//                 )),
//             )
//                 .into_response();
//         }
//         Err(err) => {
//             eprintln!("DB error checking assignment: {:?}", err);
//             return (
//                 StatusCode::INTERNAL_SERVER_ERROR,
//                 Json(ApiResponse::<UploadedFileMetadata>::error("Database error")),
//             )
//                 .into_response();
//         }
//     }
// 
//     let field = match multipart.next_field().await {
//         Ok(Some(field)) => field,
//         Ok(None) => {
//             return (
//                 StatusCode::BAD_REQUEST,
//                 Json(ApiResponse::<UploadedFileMetadata>::error(
//                     "No file provided",
//                 )),
//             )
//                 .into_response();
//         }
//         Err(e) => {
//             eprintln!("Error reading multipart field: {:?}", e);
//             return (
//                 StatusCode::BAD_REQUEST,
//                 Json(ApiResponse::<UploadedFileMetadata>::error(
//                     "Invalid file upload",
//                 )),
//             )
//                 .into_response();
//         }
//     };
// 
//     let file_name = match field.file_name().map(|s| s.to_string()) {
//         Some(name) => name,
//         None => {
//             return (
//                 StatusCode::BAD_REQUEST,
//                 Json(ApiResponse::<UploadedFileMetadata>::error(
//                     "No filename provided",
//                 )),
//             )
//                 .into_response();
//         }
//     };
// 
//     let allowed_extensions = [".tgz", ".gz", ".tar", ".zip"];
//     let file_extension = std::path::Path::new(&file_name)
//         .extension()
//         .and_then(|ext| ext.to_str())
//         .map(|ext| format!(".{}", ext.to_lowercase()));
// 
//     if !file_extension
//         .as_ref()
//         .map_or(false, |ext| allowed_extensions.contains(&ext.as_str()))
//     {
//         return (
//             StatusCode::BAD_REQUEST,
//             Json(ApiResponse::<UploadedFileMetadata>::error(
//                 "Only .tgz, .gz, .tar, and .zip files are allowed",
//             )),
//         )
//             .into_response();
//     }
// 
//     let file_bytes = match field.bytes().await {
//         Ok(bytes) => bytes,
//         Err(e) => {
//             eprintln!("Error reading file bytes: {:?}", e);
//             return (
//                 StatusCode::BAD_REQUEST,
//                 Json(ApiResponse::<UploadedFileMetadata>::error(
//                     "Error reading file",
//                 )),
//             )
//                 .into_response();
//         }
//     };
// 
//     if file_bytes.is_empty() {
//         return (
//             StatusCode::BAD_REQUEST,
//             Json(ApiResponse::<UploadedFileMetadata>::error(
//                 "Empty file provided",
//             )),
//         )
//             .into_response();
//     }
// 
// let saved = match AssignmentSubmissionModel::save_file(
//     &db,
//     assignment_id,
//     claims.sub,
//     &file_name,
//     &file_bytes,
// )
// .await
// {
//     Ok(model) => model,
//     Err(e) => {
//         eprintln!("Error saving submission: {:?}", e);
//         return (
//             StatusCode::INTERNAL_SERVER_ERROR,
//             Json(ApiResponse::<AssignmentSubmissionMetadata>::error("Failed to save submission")),
//         )
//             .into_response();
//     }
// };
// 
// let response = AssignmentSubmissionMetadata {
//     id: saved.id,
//     assignment_id: saved.assignment_id,
//     user_id: saved.user_id,
//     filename: saved.filename,
//     path: saved.path,
//     created_at: saved.created_at.to_rfc3339(),
//     updated_at: saved.updated_at.to_rfc3339(),
// };
// 
// (
//     StatusCode::CREATED,
//     Json(ApiResponse::success(
//         response,
//         "Assignment submitted successfully",
//     )),
// )
//     .into_response()
// }


