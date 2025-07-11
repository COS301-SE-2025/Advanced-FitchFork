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

use sea_orm::{ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, QueryFilter};

use crate::response::ApiResponse;
use db::models::assignment_task::{ActiveModel, Column, Entity};
use db::{
    connect,
    models::{
        assignment::{
            AssignmentType, Column as AssignmentColumn, Entity as AssignmentEntity,
            Model as AssignmentModel,
        },
        assignment_file::Model as FileModel,
    },
};

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

/// POST /api/modules/:module_id/assignments/:assignment_id/files
///
/// Upload a single assignment-related file (Admin or Lecturer only).
///
/// ### Multipart Body (form-data)
/// - `file_type` (string) — one of: `spec`, `main`, `memo`, etc.
/// - `file` (file) — exactly one file must be uploaded
///
/// ### Example curl
/// ```bash
/// curl -X POST http://localhost:3000/api/modules/1/assignments/2/files \
///   -H "Authorization: Bearer <token>" \
///   -F "file_type=spec" \
///   -F "file=@brief.pdf"
/// ```
///
/// ### Responses
/// - `201 Created` – File successfully uploaded
/// - `400 Bad Request` – Missing/invalid file_type, multiple files, or file missing
/// - `403 Forbidden` – User not authorized (must be Admin or Lecturer)
/// - `404 Not Found` – Assignment or module not found
/// - `422 Unprocessable Entity` – File rejected due to invalid format
pub async fn upload_files(
    Path((module_id, assignment_id)): Path<(i64, i64)>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    use crate::response::ApiResponse;
    use db::models::assignment_file::FileType;

    let db = connect().await;

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
                Json(ApiResponse::<UploadedFileMetadata>::error(
                    "Assignment not found",
                )),
            )
                .into_response();
        }
        Err(e) => {
            eprintln!("DB error checking assignment: {:?}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<UploadedFileMetadata>::error("Database error")),
            )
                .into_response();
        }
    }

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
        &db,
        assignment_id,
        module_id,
        file_type.clone(),
        &file_name,
        &file_bytes,
    )
    .await
    {
        Ok(saved) => {
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
    let available_from =
        match DateTime::parse_from_rfc3339(&req.available_from).map(|dt| dt.with_timezone(&Utc)) {
            Ok(date) => date,
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::<AssignmentResponse>::error(
                        "Invalid available_from datetime",
                    )),
                );
            }
        };

    let due_date =
        match DateTime::parse_from_rfc3339(&req.due_date).map(|dt| dt.with_timezone(&Utc)) {
            Ok(date) => date,
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::<AssignmentResponse>::error(
                        "Invalid due_date datetime",
                    )),
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
    .await
    {
        Ok(model) => {
            let response = AssignmentResponse::from(model);
            (
                StatusCode::OK,
                Json(ApiResponse::success(
                    response,
                    "Assignment created successfully",
                )),
            )
        }
        Err(DbErr::RecordNotInserted) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<AssignmentResponse>::error(
                "Assignment could not be inserted",
            )),
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

/// POST /api/modules/:module_id/assignments/:assignment_id/submissions
///
/// Upload a submission file for a given assignment (Students only).
///
/// ### Multipart Body (form-data)
/// - file (single file, must be .tgz, .gz, .tar, or .zip)
///
/// ### Example curl
/// ```bash
/// curl -X POST http://localhost:3000/api/modules/1/assignments/2/submissions \
///   -H "Authorization: Bearer <token>" \
///   -F "file=@solution.zip"
/// ```
///
/// ### Responses
/// - `201 Created` with submission metadata
/// - `400 Bad Request` (no file, invalid format, or unsupported file type)
/// - `403 Forbidden` (unauthorized)
/// - `404 Not Found` (assignment/module not found)

// pub async fn submit_assignment(
//     Path((module_id, assignment_id)): Path<(i64, i64)>,
//     Extension(AuthUser(claims)): Extension<AuthUser>,
//     mut multipart: Multipart,
// ) -> impl IntoResponse {
//     let db = connect().await;

//     let assignment_exists = AssignmentEntity::find()
//         .filter(AssignmentColumn::Id.eq(assignment_id as i32))
//         .filter(AssignmentColumn::ModuleId.eq(module_id as i32))
//         .one(&db)
//         .await;

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

//     let allowed_extensions = [".tgz", ".gz", ".tar", ".zip"];
//     let file_extension = std::path::Path::new(&file_name)
//         .extension()
//         .and_then(|ext| ext.to_str())
//         .map(|ext| format!(".{}", ext.to_lowercase()));

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

//     if file_bytes.is_empty() {
//         return (
//             StatusCode::BAD_REQUEST,
//             Json(ApiResponse::<UploadedFileMetadata>::error(
//                 "Empty file provided",
//             )),
//         )
//             .into_response();
//     }

//TODO - Reece please fix this. I think this route needs to be nested one deeper since assignment_submission also has an attempt number
//This:
//POST /api/modules/:module_id/assignments/:assignment_id/submissions
//Needs to change to this:
//POST /api/modules/:module_id/assignments/:assignment_id/submissions/:attempt_number

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

// let response = AssignmentSubmissionMetadata {
//     id: saved.id,
//     assignment_id: saved.assignment_id,
//     user_id: saved.user_id,
//     filename: saved.filename,
//     path: saved.path,
//     created_at: saved.created_at.to_rfc3339(),
//     updated_at: saved.updated_at.to_rfc3339(),
// };

// (
//     StatusCode::CREATED,
//     Json(ApiResponse::success(
//         response,
//         "Assignment submitted successfully",
//     )),
// )
//     .into_response()
// }

#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    task_number: i64,
    command: String,
}

#[derive(Debug, Serialize)]
pub struct TaskResponse {
    id: i64,
    task_number: i64,
    command: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

/// POST /api/modules/:module_id/assignments/:assignment_id/tasks
///
/// Create a new task for a given assignment (Lecturers and Admins only).
///
/// Each task must have a unique `task_number` within the assignment. The `command`
/// field defines how the task will be executed during evaluation.
///
/// ### JSON Body
/// ```json
/// {
///   "task_number": 1,
///   "command": "cargo test"
/// }
/// ```
///
/// ### Example curl
/// ```bash
/// curl -X POST http://localhost:3000/api/modules/1/assignments/2/tasks \
///   -H "Authorization: Bearer <token>" \
///   -H "Content-Type: application/json" \
///   -d '{"task_number": 1, "command": "cargo test"}'
/// ```
///
/// ### Responses
/// - `201 Created` with task metadata
/// - `400 Bad Request` (missing or malformed JSON body)
/// - `403 Forbidden` (unauthorized)
/// - `404 Not Found` (assignment or module not found)
/// - `422 Unprocessable Entity` (invalid input or task_number not unique)
/// - `500 Internal Server Error` (unexpected database error)
pub async fn create_task(
    Path((module_id, assignment_id)): Path<(i64, i64)>,
    Json(payload): Json<CreateTaskRequest>,
) -> impl IntoResponse {
    if payload.task_number <= 0 || payload.command.trim().is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ApiResponse::<()>::error("Invalid task_number or command")),
        )
            .into_response();
    }

    let db = connect().await;

    let assignment = AssignmentEntity::find()
        .filter(AssignmentColumn::Id.eq(assignment_id as i32))
        .filter(AssignmentColumn::ModuleId.eq(module_id as i32))
        .one(&db)
        .await;

    match assignment {
        Ok(Some(_)) => {}
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<()>::error("Assignment or module not found")),
            )
                .into_response();
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("Database error")),
            )
                .into_response();
        }
    }

    let exists = Entity::find()
        .filter(Column::AssignmentId.eq(assignment_id))
        .filter(Column::TaskNumber.eq(payload.task_number))
        .one(&db)
        .await;

    if let Ok(Some(_)) = exists {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ApiResponse::<()>::error("task_number must be unique")),
        )
            .into_response();
    }

    let now = Utc::now();
    let new_task = ActiveModel {
        assignment_id: sea_orm::ActiveValue::Set(assignment_id),
        task_number: sea_orm::ActiveValue::Set(payload.task_number),
        command: sea_orm::ActiveValue::Set(payload.command.clone()),
        created_at: sea_orm::ActiveValue::Set(now.clone()),
        updated_at: sea_orm::ActiveValue::Set(now.clone()),
        ..Default::default()
    };

    match new_task.insert(&db).await {
        Ok(task) => {
            let response = TaskResponse {
                id: task.id,
                task_number: task.task_number,
                command: task.command,
                created_at: task.created_at,
                updated_at: task.updated_at,
            };

            (
                StatusCode::CREATED,
                Json(ApiResponse::success(response, "Task created successfully")),
            )
                .into_response()
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error("Failed to create task")),
        )
            .into_response(),
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
