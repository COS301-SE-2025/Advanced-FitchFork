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
use util::{execution_config::{execution_config::Language, ExecutionConfig}, state::AppState};
use chrono::Utc;
use moss_parser::{parse_moss, ParseOptions};


#[derive(Serialize, Deserialize)]
pub struct CreatePlagiarismCasePayload {
    pub submission_id_1: i64,
    pub submission_id_2: i64,
    pub description: String,
    pub similarity: f32,
}

#[derive(Serialize)]
pub struct PlagiarismCaseResponse {
    pub id: i64,
    pub assignment_id: i64,
    pub submission_id_1: i64,
    pub submission_id_2: i64,
    pub description: String,
    pub status: String,
    pub similarity: f32,
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
/// - `submission_id_1` (number): First submission ID (must differ from second)
/// - `submission_id_2` (number): Second submission ID (must differ from first)
/// - `description` (string): Explanation of the plagiarism case
/// - `similarity` (number, required): Float **percentage** in the range **0.0–100.0** (inclusive)
///
/// # Returns
///
/// - `201 Created` with the newly created plagiarism case on success
/// - `400 BAD REQUEST` for invalid payload (same submissions, missing/invalid similarity, bad range, etc.)
/// - `403 FORBIDDEN` if user lacks required permissions
/// - `500 INTERNAL SERVER ERROR` for database errors or creation failures
///
/// # Example Request
///
/// ```json
/// {
///   "submission_id_1": 42,
///   "submission_id_2": 51,
///   "description": "Similarity in logic and structure between both files.",
///   "similarity": 72.5
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
///     "similarity": 72.5,
///     "created_at": "2024-05-20T14:30:00Z",
///     "updated_at": "2024-05-20T14:30:00Z"
///   }
/// }
/// ```
///
/// # Example Error Responses
///
/// - `400 Bad Request` (same submission IDs)
/// ```json
/// { "success": false, "message": "Submissions cannot be the same" }
/// ```
///
/// - `400 Bad Request` (submission not found)
/// ```json
/// { "success": false, "message": "One or both submissions do not exist or belong to a different assignment" }
/// ```
///
/// - `400 Bad Request` (similarity out of range)
/// ```json
/// { "success": false, "message": "Similarity must be between 0.0 and 100.0" }
/// ```
///
/// - `403 Forbidden`
/// ```json
/// { "success": false, "message": "Forbidden: Insufficient permissions" }
/// ```
///
/// - `500 Internal Server Error`
/// ```json
/// { "success": false, "message": "Failed to create plagiarism case" }
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
        ).into_response();
    }

    // Validate the similarity range (strictly enforced, no clamping)
    if !(0.0_f32..=100.0_f32).contains(&payload.similarity) || !payload.similarity.is_finite() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error("Similarity must be between 0.0 and 100.0".to_string())),
        ).into_response();
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
            Json(ApiResponse::<()>::error(
                "One or both submissions do not exist or belong to a different assignment".to_string(),
            )),
        ).into_response();
    }

    let new_case = plagiarism_case::Model::create_case(
        app_state.db(),
        assignment_id,
        payload.submission_id_1,
        payload.submission_id_2,
        &payload.description,
        payload.similarity, // required f32
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
                    similarity: case.similarity,
                    created_at: case.created_at,
                    updated_at: case.updated_at,
                },
                "Plagiarism case created successfully",
            )),
        ).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error("Failed to create plagiarism case".to_string())),
        ).into_response(),
    }
}

// somewhere in your types for this route
#[derive(serde::Deserialize)]
pub struct MossRequest {
    pub language: String,
}

/// POST /api/modules/{module_id}/assignments/{assignment_id}/plagiarism/moss
///
/// Runs a MOSS check on the **latest submission for every student** on the assignment,
/// then parses the report and **auto-creates plagiarism cases** for each matched pair.
/// Each created case is inserted with:
/// - `status = "review"`
/// - `similarity` as a **float percentage (0.0–100.0)** taken from MOSS’ `total_percent`
/// - a generated, human-readable `description`
///
/// Accessible only to lecturers and assistant lecturers assigned to the module.
///
/// # Path Parameters
///
/// - `module_id`: The ID of the parent module
/// - `assignment_id`: The ID of the assignment containing the submissions
///
/// # Request Body
///
/// **None.** The programming language is read from the assignment configuration
/// (`project.language`) persisted for this assignment.
///
/// # Returns
///
/// - `200 OK` on success with details about the MOSS run and case creation:
///   ```json
///   {
///     "success": true,
///     "message": "MOSS check completed successfully; cases created from report",
///     "data": {
///       "report_url": "http://moss.stanford.edu/results/123456789",
///       "cases_created": 7,
///       "cases_skipped": 2,
///       "title": "moss results for ... (optional)"
///     }
///   }
///   ```
/// - `500 INTERNAL SERVER ERROR` for MOSS server errors, parsing failures,
///   or other unexpected failures. The response body contains an error message.
///
/// # Notes
/// - Language is taken from the saved assignment config (`project.language`).
/// - Base (starter) files attached to the assignment are included in the comparison if present.
/// - The selected submissions are the most recent (highest `attempt`) per user.
/// - Case creation is **deduplicated** per pair of submissions (order-independent).
/// - `similarity` is stored as an `f32` percent, clamped to **0.0–100.0**.
/// - Newly created cases start in `"review"` status and can be managed via the plagiarism cases API/UI.
pub async fn run_moss_check(
    State(app_state): State<AppState>,
    Path((module_id, assignment_id)): Path<(i64, i64)>,
) -> impl IntoResponse {
    // 0) Load assignment config to determine language
    let cfg = match ExecutionConfig::get_execution_config(module_id, assignment_id) {
        Ok(c) => c,
        Err(_e) => ExecutionConfig::default_config(), // fallback to defaults
    };

    // Map your enum -> MOSS language string
    let moss_language: &str = match cfg.project.language {
        Language::Cpp => "cpp",
        Language::Java => "java",
        Language::Python => "python",
    };

    // 1) Collect latest submissions
    let submissions = assignment_submission::Model::get_latest_submissions_for_assignment(
        app_state.db(),
        assignment_id,
    )
    .await;

    match submissions {
        Ok(submissions) => {
            let mut submission_files = Vec::new();
            for submission in &submissions {
                let user = UserEntity::find_by_id(submission.user_id)
                    .one(app_state.db())
                    .await
                    .map_err(|_| "Failed to fetch user")
                    .unwrap();

                // Optional username helps attribution in MOSS rows.
                let username = user.map(|u| u.username);
                submission_files.push((submission.full_path(), username, Some(submission.id)));
            }

            let base_files = match assignment_file::Model::get_base_files(app_state.db(), assignment_id).await {
                Ok(files) => files.into_iter().map(|f| f.full_path()).collect::<Vec<_>>(),
                Err(_) => vec![],
            };

            let moss_user_id =
                std::env::var("MOSS_USER_ID").unwrap_or_else(|_| "YOUR_MOSS_USER_ID".to_string());
            let moss_service = MossService::new(&moss_user_id);

            // 🔁 Use language from config, not from request payload
            match moss_service.run(base_files, submission_files, moss_language).await {
                Ok(report_url) => {
                    // 1) Persist a tiny text file (unchanged)
                    let report_dir = assignment_submission::Model::storage_root()
                        .join(format!("module_{}", module_id))
                        .join(format!("assignment_{}", assignment_id));

                    if let Err(e) = std::fs::create_dir_all(&report_dir) {
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(ApiResponse::<()>::error(format!(
                                "Failed to create report directory: {}",
                                e
                            ))),
                        )
                            .into_response();
                    }

                    let report_path = report_dir.join("reports.txt");
                    let content = format!("Report URL: {}\nDate: {}", report_url, Utc::now().to_rfc3339());

                    if let Err(e) = std::fs::write(&report_path, content) {
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(ApiResponse::<()>::error(format!(
                                "Failed to write MOSS report: {}",
                                e
                            ))),
                        )
                            .into_response();
                    }

                    // 2) Parse the report and auto-create plagiarism cases (with similarity)
                    let parse_opts = ParseOptions {
                        min_lines: 0,           // tweak as needed
                        include_matches: false, // we only need aggregate % per pair
                    };

                    let parsed = match parse_moss(&report_url, parse_opts).await {
                        Ok(out) => out,
                        Err(e) => {
                            return (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                Json(ApiResponse::<()>::error(format!(
                                    "MOSS report parse failed: {}",
                                    e
                                ))),
                            )
                                .into_response();
                        }
                    };

                    use std::collections::HashSet;
                    let mut seen: HashSet<(i64, i64)> = HashSet::new();
                    let mut created_count = 0usize;
                    let mut skipped_count = 0usize;

                    for r in parsed.reports {
                        let (Some(sub_a), Some(sub_b)) = (r.submission_id_a, r.submission_id_b) else {
                            skipped_count += 1;
                            continue;
                        };

                        // Normalize order: (min, max)
                        let (a, b, ua, ub) = if sub_a <= sub_b {
                            (sub_a, sub_b, r.user_a, r.user_b)
                        } else {
                            (sub_b, sub_a, r.user_b, r.user_a)
                        };

                        if !seen.insert((a, b)) {
                            continue;
                        }

                        let description = generate_description(
                            &ua,
                            &ub,
                            a,
                            b,
                            r.total_lines_matched,
                            r.total_percent,
                        );

                        // Convert Option<f64> -> f32 and clamp to 0..=100
                        let similarity: f32 = r
                            .total_percent
                            .unwrap_or(0.0)
                            .max(0.0)
                            .min(100.0) as f32;

                        match plagiarism_case::Model::create_case(
                            app_state.db(),
                            assignment_id,
                            a,
                            b,
                            &description,
                            similarity,
                        )
                        .await
                        {
                            Ok(_) => created_count += 1,
                            Err(_db_err) => {
                                // Log if desired
                                skipped_count += 1;
                            }
                        }
                    }

                    (
                        StatusCode::OK,
                        Json(ApiResponse::success(
                            serde_json::json!({
                                "report_url": report_url,
                                "cases_created": created_count,
                                "cases_skipped": skipped_count,
                                "title": parsed.title,
                            }),
                            "MOSS check completed successfully; cases created from report",
                        )),
                    )
                        .into_response()
                }
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()>::error(format!("Failed to run MOSS check: {}", e))),
                )
                    .into_response(),
            }
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error("Failed to retrieve submissions".to_string())),
        )
            .into_response(),
    }
}


fn generate_description(
    user_a: &str,
    user_b: &str,
    sub_a: i64,
    sub_b: i64,
    total_lines: i64,
    total_percent: Option<f64>
) -> String {
    let percent = total_percent.unwrap_or(0.0);

    let level = if percent >= 90.0 {
        "extensive similarity, indicating a strong likelihood of shared or reused code"
    } else if percent >= 70.0 {
        "substantial overlap, suggesting possible reuse or collaboration"
    } else if percent >= 50.0 {
        "moderate similarity, which may indicate shared structural elements"
    } else if percent >= 30.0 {
        "notable similarity, which could reflect influence or common coding approaches"
    } else {
        "limited similarity, likely due to common coding patterns or libraries"
    };

    format!(
        "Submissions `{}` ({}) and `{}` ({}) show {}, with {} lines matched and a similarity score of {:.1}%.",
        sub_a, user_a, sub_b, user_b, level, total_lines, percent
    )
}
