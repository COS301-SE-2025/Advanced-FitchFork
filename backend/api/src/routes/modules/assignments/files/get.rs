use std::{env, path::PathBuf};

use axum::{
    extract::Path,
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Json, Response},
};

use tokio::{fs::File as FsFile, io::AsyncReadExt};

use sea_orm::{
    ColumnTrait,
    EntityTrait,
    QueryFilter,
};

use crate::response::ApiResponse;
use db::models::{
    assignment::{
        Column as AssignmentColumn,
        Entity as AssignmentEntity,
    },
    assignment_file::{
        Column as FileColumn,
        Entity as FileEntity,
    },
};
use crate::routes::modules::assignments::common::File;

/// GET /api/modules/:module_id/assignments/:assignment_id/files/:file_id
///
/// Download a specific file from an assignment. Accessible to users assigned to the module.
///
/// ### Path Parameters
/// - `module_id` (i64): The ID of the module containing the assignment
/// - `assignment_id` (i64): The ID of the assignment containing the file
/// - `file_id` (i64): The ID of the file to download
///
/// ### Responses
///
/// - `200 OK`: Returns the file as a binary attachment with appropriate headers
/// - `404 Not Found`
/// ```json
/// {
///   "success": false,
///   "message": "File not found" // or "File missing on disk"
/// }
/// ```
///
/// - `500 Internal Server Error`
/// ```json
/// {
///   "success": false,
///   "message": "Database error" // or "Could not open file" or "Failed to read file"
/// }
/// ```
///
pub async fn download_file(
    Path((_module_id, assignment_id, file_id)): Path<(i64, i64, i64)>,
) -> Response {
    let db = db::connect().await;

    let file = match FileEntity::find()
        .filter(FileColumn::Id.eq(file_id as i32))
        .filter(FileColumn::AssignmentId.eq(assignment_id as i32))
        .one(&db)
        .await
    {
        Ok(Some(file)) => file,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<()>::error("File not found")),
            )
                .into_response();
        }
        Err(err) => {
            eprintln!("DB error fetching file: {:?}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("Database error")),
            )
                .into_response();
        }
    };

    let storage_root =
        env::var("ASSIGNMENT_STORAGE_ROOT").unwrap_or_else(|_| "data/assignment_files".to_string());
    let fs_path = PathBuf::from(storage_root).join(&file.path);

    eprintln!("Resolved file path: {:?}", fs_path);

    if tokio::fs::metadata(&fs_path).await.is_err() {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("File missing on disk")),
        )
            .into_response();
    }

    let mut file_handle = match FsFile::open(&fs_path).await {
        Ok(f) => f,
        Err(err) => {
            eprintln!("File open error: {:?}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("Could not open file")),
            )
                .into_response();
        }
    };

    let mut buffer = Vec::new();
    if let Err(err) = file_handle.read_to_end(&mut buffer).await {
        eprintln!("File read error: {:?}", err);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error("Failed to read file")),
        )
            .into_response();
    }

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!("attachment; filename=\"{}\"", file.filename))
            .unwrap_or_else(|_| HeaderValue::from_static("attachment")),
    );
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/octet-stream"),
    );

    (StatusCode::OK, headers, buffer).into_response()
}

/// GET /api/modules/:module_id/assignments/:assignment_id/files
///
/// List all files associated with an assignment. Accessible to users assigned to the module.
///
/// ### Path Parameters
/// - `module_id` (i64): The ID of the module containing the assignment
/// - `assignment_id` (i64): The ID of the assignment to list files for
///
/// ### Responses
///
/// - `200 OK`
/// ```json
/// {
///   "success": true,
///   "message": "Assignment files retrieved successfully",
///   "data": [
///     {
///       "id": "123",
///       "filename": "assignment.pdf",
///       "path": "module_456/assignment_789/assignment.pdf",
///       "created_at": "2024-01-01T00:00:00Z",
///       "updated_at": "2024-01-01T00:00:00Z"
///     }
///   ]
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
///   "message": "Database error" // or "Failed to retrieve files"
/// }
/// ```
///
pub async fn list_files(Path((module_id, assignment_id)): Path<(i64, i64)>) -> Response {
    let db = db::connect().await;

    match AssignmentEntity::find()
        .filter(AssignmentColumn::Id.eq(assignment_id as i32))
        .filter(AssignmentColumn::ModuleId.eq(module_id as i32))
        .one(&db)
        .await
    {
        Ok(Some(_)) => {} // assignment exists
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<Vec<File>>::error("Assignment not found")),
            )
                .into_response();
        }
        Err(err) => {
            eprintln!("DB error checking assignment: {:?}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<Vec<File>>::error("Database error")),
            )
                .into_response();
        }
    }

    match FileEntity::find()
        .filter(FileColumn::AssignmentId.eq(assignment_id as i32))
        .all(&db)
        .await
    {
        Ok(files) => {
            let file_list: Vec<File> = files
                .into_iter()
                .map(|f| File {
                    id: f.id.to_string(),
                    filename: f.filename,
                    path: f.path,
                    created_at: f.created_at.to_rfc3339(),
                    updated_at: f.updated_at.to_rfc3339(),
                })
                .collect();

            (
                StatusCode::OK,
                Json(ApiResponse::success(
                    file_list,
                    "Assignment files retrieved successfully",
                )),
            )
                .into_response()
        }
        Err(err) => {
            eprintln!("DB error fetching files: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<Vec<File>>::error("Failed to retrieve files")),
            )
                .into_response()
        }
    }
}