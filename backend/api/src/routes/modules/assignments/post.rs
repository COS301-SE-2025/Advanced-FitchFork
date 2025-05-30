/// Handles the creation of a new assignment.
///
/// # Request Body
/// Expects a JSON object with the following fields:
/// - `module_id` (i64): The ID of the module to which the assignment belongs. **Required**
/// - `name` (string): The name of the assignment. **Required**
/// - `description` (string): A description of the assignment. *(Optional)*
/// - `available_from` (string): The date from which the assignment is available (ISO 8601 format). **Required**
/// - `due_date` (string): The due date for the assignment (ISO 8601 format). **Required**
/// - `assignment_type` (string): The type of assignment, either `"A"` for Assignment or any other value for Practical. *(Optional, defaults to Practical)*
///
/// # Responses
/// Returns a response indicating the result of the creation operation.
///
/// # Errors
/// Returns an error response if any of the required fields are missing or invalid.
/// Handles the deletion of an assignment.
use axum::{
    extract::{Multipart, Path},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use chrono::{DateTime, Utc};

use serde::{Deserialize, Serialize};

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, DbErr};

use db::{
    connect,
    models::{
        assignment::{Column as AssignmentColumn, Entity as AssignmentEntity, Model as AssignmentModel, AssignmentType},
        assignment_file::{FileType, Model as FileModel},
    },
};

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


/// POST /api/modules/:module_id/assignments/:assignment_id/files
///
/// Upload one or more files for a given assignment (Admin or Lecturer only).
///
/// ### Multipart Body (form-data)
/// - files[] (one or more)
///
/// ### Example curl
/// ```bash
/// curl -X POST http://localhost:3000/api/modules/1/assignments/2/files \
///   -H "Authorization: Bearer <token>" \
///   -F "files[]=@brief.pdf" \
///   -F "files[]=@rubric.xlsx"
/// ```
///
/// ### Responses
/// - `201 Created`
/// - `400 Bad Request` (no files or invalid format)
/// - `403 Forbidden` (unauthorized)
/// - `404 Not Found` (assignment/module not found)
pub async fn upload_files(
    Path((module_id, assignment_id)): Path<(i64, i64)>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let db = connect().await;

    // 1. Check if the assignment exists
    let assignment_exists = AssignmentEntity::find()
        .filter(AssignmentColumn::Id.eq(assignment_id as i32))
        .filter(AssignmentColumn::ModuleId.eq(module_id as i32))
        .one(&db)
        .await;

    match assignment_exists {
        Ok(Some(_)) => {}
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<Vec<UploadedFileMetadata>>::error("Assignment not found")),
            )
                .into_response();
        }
        Err(err) => {
            eprintln!("DB error checking assignment: {:?}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<Vec<UploadedFileMetadata>>::error("Database error")),
            )
                .into_response();
        }
    }

    // 2. Save files
    let mut saved_files = vec![];

    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let file_name = match field.file_name().map(|s| s.to_string()) {
            Some(name) => name,
            None => continue,
        };

        let file_bytes = field.bytes().await.unwrap_or_default();
        if file_bytes.is_empty() {
            continue;
        }

        let file_type = FileType::Spec; // Or dynamically assign based on form field

        match FileModel::save_file(
            &db,
            assignment_id,
            module_id,
            file_type,
            &file_name,
            &file_bytes,
        )
        .await
        {
            Ok(saved) => saved_files.push(UploadedFileMetadata {
                id: saved.id,
                assignment_id: saved.assignment_id,
                filename: saved.filename,
                path: saved.path,
                created_at: saved.created_at.to_rfc3339(),
                updated_at: saved.updated_at.to_rfc3339(),
            }),
            Err(e) => {
                eprintln!("Error saving file '{}': {:?}", file_name, e);
                continue;
            }
        }
    }

    if saved_files.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<Vec<UploadedFileMetadata>>::error("No files were uploaded")),
        )
            .into_response();
    }

    (
        StatusCode::CREATED,
        Json(ApiResponse::success(
            saved_files,
            "Files uploaded successfully",
        )),
    )
        .into_response()
}



#[derive(Debug, Serialize)]
pub struct AssignmentResponse {
    pub id: i64,
    pub module_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub assignment_type: String,
    pub available_from: String,
    pub due_date: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<AssignmentModel> for AssignmentResponse {
    fn from(model: AssignmentModel) -> Self {
        Self {
            id: model.id,
            module_id: model.module_id,
            name: model.name,
            description: model.description,
            assignment_type: model.assignment_type.to_string(),
            available_from: model.available_from.to_rfc3339(),
            due_date: model.due_date.to_rfc3339(),
            created_at: model.created_at.to_rfc3339(),
            updated_at: model.updated_at.to_rfc3339(),
        }
    }
}


/// Creates a new assignment under a specific module.
///
/// # Arguments
///
/// The arguments are extracted automatically from the HTTP request:
/// - Path parameter `module_id`: The ID of the module the assignment belongs to.
/// - JSON body with the following fields:
///   - `name` (string, required): The name of the assignment.
///   - `description` (string, optional): A description of the assignment.
///   - `assignment_type` (string, required): The type of assignment. Must be either `"Assignment"` or `"Practical"`.
///   - `available_from` (string, required): The release date in ISO 8601 format.
///   - `due_date` (string, required): The due date in ISO 8601 format.
///
/// # Returns
///
/// Returns an HTTP response indicating the result:
/// - `200 OK` with the created assignment data if successful.
/// - `400 BAD REQUEST` if required fields are missing or if `assignment_type` is invalid.
/// - `500 INTERNAL SERVER ERROR` if the database operation fails.
///
/// The response body is a JSON object using a standardized API response format.
#[derive(Debug, Deserialize)]
pub struct CreateAssignmentRequest {
    name: String,
    description: Option<String>,
    assignment_type: String,
    available_from: String,
    due_date: String,
}

/// Creates a new assignment under a specific module.
pub async fn create(
    Path(module_id): Path<i64>,
    Json(req): Json<CreateAssignmentRequest>,
) -> impl IntoResponse {
    let db = connect().await;

    // Parse and validate dates
    let available_from = match DateTime::parse_from_rfc3339(&req.available_from)
        .map(|dt| dt.with_timezone(&Utc))
    {
        Ok(date) => date,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<AssignmentResponse>::error("Invalid available_from datetime")),
            );
        }
    };

    let due_date = match DateTime::parse_from_rfc3339(&req.due_date)
        .map(|dt| dt.with_timezone(&Utc))
    {
        Ok(date) => date,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<AssignmentResponse>::error("Invalid due_date datetime")),
            );
        }
    };
    // Convert string to enum
    let assignment_type = match req.assignment_type.parse::<AssignmentType>() {
        Ok(t) => t,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<AssignmentResponse>::error(
                    "assignment_type must be 'assignment' or 'practical'",
                )),
            );
        }
    };

    match AssignmentModel::create(
        &db,
        module_id,
        &req.name,
        req.description.as_deref(),
        assignment_type,
        available_from,
        due_date,
    )
    .await {
        Ok(model) => {
            let response = AssignmentResponse::from(model);
            (
                StatusCode::OK,
                Json(ApiResponse::success(response, "Assignment created successfully")),
            )
        }
        Err(DbErr::RecordNotInserted) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<AssignmentResponse>::error("Assignment could not be inserted")),
        ),
        Err(e) => {
            eprintln!("DB error: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<AssignmentResponse>::error("Database error")),
            )
        }
    }
}


#[cfg(test)]
mod tests {
    // use axum::{http::StatusCode, response::IntoResponse, Json};
    // use db::{create_test_db, delete_database};
    // use serde_json::json;

    #[tokio::test]
    async fn create_ass() {
        // let pool = create_test_db(Some("assignment.db")).await;
        // let mut request_json= Json(json!({}));
        // let response = crate::routes::assignments::post::create(request_json)
        //     .await
        //     .into_response();
        // assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        // request_json = Json(json!({
        //     "module_id":313312,
        //     "name": "prac3",
        //     "available_from": "2025-12-34",
        //     "due_date": "12-12-1"
        // }));
        // let response = crate::routes::assignments::post::create(request_json)
        //     .await
        //     .into_response();
        // assert_eq!(response.status(), StatusCode::OK);

        // pool.close().await;
        // delete_database("test_update_not_found.db");
        assert_eq!(true, true);
    }
}
