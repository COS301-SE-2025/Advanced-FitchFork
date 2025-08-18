use axum::{
    extract::{State, Path},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use db::models::{
    assignment_submission::{self, Entity as SubmissionEntity},
    assignment_file,
    plagiarism_case,
    user::{Entity as UserEntity},
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use crate::{response::ApiResponse, services::moss::MossService};
use util::state::AppState;
use std::env;
use std::fs;
use chrono::Utc;

#[derive(Serialize, Deserialize)]
pub struct CreatePlagiarismCasePayload {
    pub submission_id_1: i64,
    pub submission_id_2: i64,
    pub description: String,
}

#[derive(Serialize)]
pub struct PlagiarismCaseResponse {
    pub id: i64,
    pub assignment_id: i64,
    pub submission_id_1: i64,
    pub submission_id_2: i64,
    pub description: String,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// POST /api/modules/{module_id}/assignments/{assignment_id}/plagiarism
///
/// Creates a new plagiarism case between two submissions in a specific assignment.
/// Accessible only to lecturers and assistant lecturers assigned to the module.
///
/// # Path Parameters
///
/// - `module_id`: The ID of the parent module
/// - `assignment_id`: The ID of the assignment containing the submissions
///
/// # Request Body
///
/// Requires a JSON payload with the following fields:
/// - `submission_id_1`: First submission ID (must differ from second)
/// - `submission_id_2`: Second submission ID (must differ from first)
/// - `description`: Explanation of the plagiarism case
///
/// # Returns
///
/// Returns an HTTP response indicating the result:
/// - `201 Created` with the newly created plagiarism case on success
/// - `400 BAD REQUEST` for invalid payload or submission validation errors
/// - `403 FORBIDDEN` if user lacks required permissions
/// - `500 INTERNAL SERVER ERROR` for database errors or creation failures
///
/// The response body follows a standardized JSON format containing the created plagiarism case.
///
/// # Example Request
///
/// ```json
/// {
///   "submission_id_1": 42,
///   "submission_id_2": 51,
///   "description": "Similarity in logic and structure between both files."
/// }
/// ```
///
/// # Example Response (201 Created)
///
/// ```json
/// {
///   "success": true,
///   "message": "Plagiarism case created successfully",
///   "data": {
///     "id": 17,
///     "assignment_id": 3,
///     "submission_id_1": 42,
///     "submission_id_2": 51,
///     "description": "Similarity in logic and structure between both files.",
///     "status": "review",
///     "created_at": "2024-05-20T14:30:00Z",
///     "updated_at": "2024-05-20T14:30:00Z"
///   }
/// }
/// ```
///
/// # Example Responses
///
/// - `400 Bad Request` (same submission IDs)  
/// ```json
/// {
///   "success": false,
///   "message": "Submissions cannot be the same"
/// }
/// ```
///
/// - `400 Bad Request` (submission not found)  
/// ```json
/// {
///   "success": false,
///   "message": "One or both submissions do not exist or belong to a different assignment"
/// }
/// ```
///
/// - `403 Forbidden` (unauthorized role)  
/// ```json
/// {
///   "success": false,
///   "message": "Forbidden: Insufficient permissions"
/// }
/// ```
///
/// - `500 Internal Server Error`  
/// ```json
/// {
///   "success": false,
///   "message": "Failed to create plagiarism case"
/// }
/// ```
pub async fn create_plagiarism_case(
    State(app_state): State<AppState>,
    Path((_, assignment_id)): Path<(i64, i64)>,
    Json(payload): Json<CreatePlagiarismCasePayload>,
) -> impl IntoResponse {
    if payload.submission_id_1 == payload.submission_id_2 {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error("Submissions cannot be the same".to_string())),
        )
            .into_response();
    }

    let submission1 = SubmissionEntity::find_by_id(payload.submission_id_1)
        .filter(assignment_submission::Column::AssignmentId.eq(assignment_id))
        .one(app_state.db())
        .await
        .unwrap_or(None);

    let submission2 = SubmissionEntity::find_by_id(payload.submission_id_2)
        .filter(assignment_submission::Column::AssignmentId.eq(assignment_id))
        .one(app_state.db())
        .await
        .unwrap_or(None);

    if submission1.is_none() || submission2.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error("One or both submissions do not exist or belong to a different assignment".to_string())),
        )
            .into_response();
    }

    let new_case = plagiarism_case::Model::create_case(
        app_state.db(),
        assignment_id,
        payload.submission_id_1,
        payload.submission_id_2,
        &payload.description,
    )
    .await;

    match new_case {
        Ok(case) => (
            StatusCode::CREATED,
            Json(ApiResponse::success(
                PlagiarismCaseResponse {
                    id: case.id,
                    assignment_id,
                    submission_id_1: case.submission_id_1,
                    submission_id_2: case.submission_id_2,
                    description: case.description,
                    status: case.status.to_string(),
                    created_at: case.created_at,
                    updated_at: case.updated_at,
                },
                "Plagiarism case created successfully",
            )),
        )
            .into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error("Failed to create plagiarism case".to_string())),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
pub struct MossRequest {
    pub language: String,
}

/// POST /api/modules/{module_id}/assignments/{assignment_id}/plagiarism/moss
///
/// Runs a MOSS check on the **latest submission for every student** on the assignment.
/// Accessible only to lecturers and assistant lecturers assigned to the module.
///
/// # Path Parameters
///
/// - `module_id`: The ID of the parent module
/// - `assignment_id`: The ID of the assignment containing the submissions
///
/// # Request Body
///
/// JSON payload:
/// - `language` (string): The programming language of the submissions (e.g., "c", "cpp", "java",
///   "python", "javascript").
///
/// # Returns
///
/// Returns an HTTP response indicating the result:
/// - `200 OK` with the MOSS report URL on success
/// - `500 INTERNAL SERVER ERROR` for MOSS server errors or other failures
///
/// # Example Request
///
/// ```json
/// {
///   "language": "c"
/// }
/// ```
///
/// # Example Response (200 OK)
///
/// ```json
/// {
///   "success": true,
///   "message": "MOSS check completed successfully",
///   "data": { "report_url": "http://moss.stanford.edu/results/123456789" }
/// }
/// ```
///
/// # Notes
/// - Base (starter) files attached to the assignment are included in the comparison if present.
/// - The selected submissions are the most recent (highest `attempt`) per user.
pub async fn run_moss_check(
    State(app_state): State<AppState>,
    Path((module_id, assignment_id)): Path<(i64, i64)>,
    Json(payload): Json<MossRequest>,
) -> impl IntoResponse {
    let submissions = assignment_submission::Model::get_latest_submissions_for_assignment(
        app_state.db(),
        assignment_id
    ).await;
    
    match submissions {
        Ok(submissions) => {
            let mut submission_files = Vec::new();
            for submission in &submissions {
                let user = UserEntity::find_by_id(submission.user_id)
                    .one(app_state.db())
                    .await
                    .map_err(|_| "Failed to fetch user")
                    .unwrap();
                
                let username = user.map(|u| u.username);
                submission_files.push((
                    submission.full_path(),
                    username,
                    Some(submission.id)
                ));
            }
            
            let base_files = match assignment_file::Model::get_base_files(app_state.db(), assignment_id).await {
                Ok(files) => files.into_iter().map(|f| f.full_path()).collect(),
                Err(_) => vec![],
            };
            
            let moss_user_id = env::var("MOSS_USER_ID")
                .unwrap_or_else(|_| "YOUR_MOSS_USER_ID".to_string());
            let moss_service = MossService::new(&moss_user_id);

            match moss_service.run(
                base_files,
                submission_files,
                &payload.language,
            ).await {
                Ok(result) => {
                    let report_dir = assignment_submission::Model::storage_root()
                        .join(format!("module_{}", module_id))
                        .join(format!("assignment_{}", assignment_id));

                    if let Err(e) = fs::create_dir_all(&report_dir) {
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(ApiResponse::<()>::error(format!("Failed to create report directory: {}", e))),
                        ).into_response();
                    }

                    let report_path = report_dir.join("reports.txt");
                    let content = format!(
                        "Report URL: {}\nDate: {}",
                        result,
                        Utc::now().to_rfc3339()
                    );

                    if let Err(e) = fs::write(&report_path, content) {
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(ApiResponse::<()>::error(format!("Failed to write MOSS report: {}", e))),
                        ).into_response();
                    }

                    (
                        StatusCode::OK,
                        Json(ApiResponse::success(
                            serde_json::json!({ "report_url": result }),
                            "MOSS check completed successfully",
                        )),
                    )
                        .into_response()
                }
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()>::error(format!("Failed to run MOSS check: {}", e))),
                ).into_response(),
            }
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error("Failed to retrieve submissions".to_string())),
        ).into_response(),
    }
}