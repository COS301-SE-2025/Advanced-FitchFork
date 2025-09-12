use std::fs;
use std::path::PathBuf;

use crate::{response::ApiResponse, services::moss::MossService};
use crate::services::moss_archiver::{archive_moss_to_fs_and_zip, ArchiveOptions};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use db::models::{
    assignment_file,
    assignment_submission::{self, Entity as SubmissionEntity},
    plagiarism_case,
    user::Entity as UserEntity,
};
use db::models::assignment::{Entity as AssignmentEntity, Model as AssignmentModel};
use moss_parser::{ParseOptions, parse_moss};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use tokio::task;
use util::config;
use util::paths::assignment_dir;
use util::{
    execution_config::{ExecutionConfig, execution_config::Language},
    state::AppState,
};
use tracing::{error, info};

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
            Json(ApiResponse::<()>::error(
                "Submissions cannot be the same".to_string(),
            )),
        )
            .into_response();
    }

    // Validate the similarity range (strictly enforced, no clamping)
    if !(0.0_f32..=100.0_f32).contains(&payload.similarity) || !payload.similarity.is_finite() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(
                "Similarity must be between 0.0 and 100.0".to_string(),
            )),
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
            Json(ApiResponse::<()>::error(
                "One or both submissions do not exist or belong to a different assignment"
                    .to_string(),
            )),
        )
            .into_response();
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
        )
            .into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(
                "Failed to create plagiarism case".to_string(),
            )),
        )
            .into_response(),
    }
}

// somewhere in your types for this route
#[derive(serde::Deserialize)]
pub struct MossRequest {
    pub language: String,
}

/// POST /api/modules/{module_id}/assignments/{assignment_id}/plagiarism/moss
///
/// Runs a MOSS check for the assignment, then parses the result and **auto-creates
/// plagiarism cases** for each matched pair. After a successful run, a **background job**
/// also mirrors (archives) the full MOSS report (index, matches, and images) to local
/// storage and generates a ZIP — this archiving step is **non-blocking**.
///
/// Each created case is inserted with:
/// - `status = "review"`
/// - `similarity` as a **float percentage (0.0–100.0)** from MOSS’ `total_percent`
/// - an auto-generated, human-readable `description`
///
/// Accessible only to lecturers and assistant lecturers assigned to the module.
///
/// # Path Parameters
///
/// - `module_id`: The ID of the parent module.
/// - `assignment_id`: The ID of the assignment.
///
/// # Request Body
///
/// **None.** The language is read from the assignment’s execution config
/// (`project.language`).
///
/// # Behavior
///
/// - **Submission selection respects the assignment’s grading policy**:
///   - `grading_policy = "last"` → uses each student’s **most recent** non-practice, non-ignored submission.
///   - `grading_policy = "best"` → uses each student’s **best-scoring** non-practice, non-ignored submission.
/// - Base (starter) files attached to the assignment are included if present.
/// - The MOSS result URL and timestamp are saved to:
///   `<storage_root>/module_{module_id}/assignment_{assignment_id}/reports.txt`.
/// - A **fire-and-forget archive job** mirrors the report for offline viewing to:
///   - Folder: `<storage_root>/module_{module_id}/assignment_{assignment_id}/moss_archive/`
///   - ZIP:    `<storage_root>/module_{module_id}/assignment_{assignment_id}/moss_archive.zip`
///   The archive includes **all images** referenced by the report. Existing archives are overwritten.
/// - Case creation is **deduplicated** per pair (order-independent).
/// - `similarity` is stored as an `f32` percent, clamped to **0.0–100.0**.
/// - Newly created cases start in `"review"` status and can be managed via the plagiarism APIs/UI.
///
/// # Returns
///
/// - `200 OK` on success with details about the MOSS run and created cases:
///   ```json
///   {
///     "success": true,
///     "message": "MOSS check completed successfully; cases created from report",
///     "data": {
///       "report_url": "http://moss.stanford.edu/results/123456789",
///       "cases_created": 7,
///       "cases_skipped": 2,
///       "title": "moss results for COS123 / Assignment 1",
///       "archive_started": true
///     }
///   }
///   ```
/// - `500 INTERNAL SERVER ERROR` for MOSS errors, parsing failures, or other unexpected failures.
///   The response message explains the reason.
///
/// # Notes
///
/// - The external `report_url` may expire per MOSS retention policy, but the **local archive** and
///   **ZIP** remain available on the server once the background job finishes.
/// - “Practice” and “ignored” submissions are excluded from selection.
/// - If no eligible submissions exist, no cases will be created.
pub async fn run_moss_check(
    State(app_state): State<AppState>,
    Path((module_id, assignment_id)): Path<(i64, i64)>,
) -> impl IntoResponse {
    // 0) Load assignment config to determine language
    let cfg = match ExecutionConfig::get_execution_config(module_id, assignment_id) {
        Ok(c) => c,
        Err(_e) => ExecutionConfig::default_config(),
    };

    let moss_language: &str = match cfg.project.language {
        Language::Cpp => "c++",
        Language::Java => "java",
    };

    // 0.1) Load the assignment model (needed by get_best_for_user to read policy)
    let assignment_model: AssignmentModel = match AssignmentEntity::find_by_id(assignment_id)
        .one(app_state.db())
        .await
        .map_err(|_| ())
        .ok()
        .flatten()
    {
        Some(a) => a,
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("Assignment not found")),
            )
                .into_response();
        }
    };

    // 1) Select exactly ONE submission per student based on grading policy:
    //    - exclude practice & ignored; selection happens inside get_best_for_user
    let selected_submissions: Vec<assignment_submission::Model> = match SubmissionEntity::find()
        .filter(assignment_submission::Column::AssignmentId.eq(assignment_id))
        .all(app_state.db())
        .await
    {
        Ok(all_for_assignment) => {
            use std::collections::HashSet;
            let mut user_ids: HashSet<i64> = HashSet::new();
            for s in &all_for_assignment {
                user_ids.insert(s.user_id);
            }

            // For each user, pick Best or Last according to the assignment’s config
            let mut chosen = Vec::with_capacity(user_ids.len());
            for uid in user_ids {
                match assignment_submission::Model::get_best_for_user(app_state.db(), &assignment_model, uid).await {
                    Ok(Some(s)) => chosen.push(s),
                    Ok(None) => { /* no eligible subs for this user; skip */ }
                    Err(_) => { /* DB issue for this user; skip */ }
                }
            }
            chosen
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("Failed to retrieve submissions")),
            )
                .into_response();
        }
    };

    // 2) Prepare file tuples for MOSS (path, optional username, optional submission_id)
    let mut submission_files = Vec::with_capacity(selected_submissions.len());
    for submission in &selected_submissions {
        let user = UserEntity::find_by_id(submission.user_id)
            .one(app_state.db())
            .await
            .map_err(|_| "Failed to fetch user")
            .ok()
            .flatten();

        let username = user.map(|u| u.username);
        submission_files.push((submission.full_path(), username, Some(submission.id)));
    }

    // Base files (starter code)
    let base_files =
        match assignment_file::Model::get_base_files(app_state.db(), assignment_id).await {
            Ok(files) => files.into_iter().map(|f| f.full_path()).collect::<Vec<_>>(),
            Err(_) => vec![],
        };

    let moss_user_id = config::moss_user_id();
    let moss_service = MossService::new(&moss_user_id);

    match moss_service.run(base_files, submission_files, moss_language).await {
        Ok(report_url) => {
            // Persist minimal metadata
            let base_dir = assignment_dir(module_id, assignment_id);

            if let Err(e) = std::fs::create_dir_all(&base_dir) {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()>::error(format!(
                        "Failed to create report directory: {e}"
                    ))),
                )
                    .into_response();
            }

            let report_path = base_dir.join("reports.txt");
            let content = format!(
                "Report URL: {}\nDate: {}",
                report_url,
                Utc::now().to_rfc3339()
            );
            if let Err(e) = std::fs::write(&report_path, content) {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()>::error(format!(
                        "Failed to write MOSS report: {e}"
                    ))),
                )
                    .into_response();
            }

            // Fire-and-forget archive job
            let archive_index_url = report_url.clone();
            let dest_root: PathBuf = base_dir.join("moss_archive");
            let zip_path: PathBuf = base_dir.join("moss_archive.zip");
            let opts = ArchiveOptions { concurrency: 12 };

            task::spawn(async move {
                match archive_moss_to_fs_and_zip(
                    &archive_index_url,
                    &dest_root,
                    &zip_path,
                    opts,
                )
                .await
                {
                    Ok((_manifest, zip_abs)) => info!("MOSS archive created at {zip_abs}"),
                    Err(e) => error!("MOSS archive failed: {e}"),
                }
            });

            // Parse report and create cases (unchanged)
            let parse_opts = ParseOptions {
                min_lines: 0,
                include_matches: false,
            };
            let parsed = match parse_moss(&report_url, parse_opts).await {
                Ok(out) => out,
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiResponse::<()>::error(format!(
                            "MOSS report parse failed: {e}"
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
                let (a, b, ua, ub) = if sub_a <= sub_b {
                    (sub_a, sub_b, r.user_a, r.user_b)
                } else {
                    (sub_b, sub_a, r.user_b, r.user_a)
                };
                if !seen.insert((a, b)) {
                    continue;
                }

                let description = generate_description(
                    &ua, &ub, a, b, r.total_lines_matched, r.total_percent,
                );
                let similarity: f32 = r.total_percent.unwrap_or(0.0).clamp(0.0, 100.0) as f32;

                match plagiarism_case::Model::create_case(
                    app_state.db(), assignment_id, a, b, &description, similarity,
                )
                .await
                {
                    Ok(_) => created_count += 1,
                    Err(_) => skipped_count += 1,
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
                        "archive_started": true
                    }),
                    "MOSS check completed successfully; cases created from report",
                )),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!(
                "Failed to run MOSS check: {e}"
            ))),
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
    total_percent: Option<f64>,
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

/// POST /api/modules/{module_id}/assignments/{assignment_id}/plagiarism/moss/archive
///
/// Mirrors the **current** MOSS report (from `reports.txt`) to local storage and zips it.
/// No URL is required in the request; the handler uses the last saved MOSS URL.
///
/// # Behavior
/// - Reads `<storage_root>/module_{module_id}/assignment_{assignment_id}/reports.txt`
///   and extracts `Report URL: ...`.
/// - Mirrors index, match pages, and images into:
///   `<storage_root>/module_{module_id}/assignment_{assignment_id}/moss_archive/`
/// - Produces a zip at:
///   `<storage_root>/module_{module_id}/assignment_{assignment_id}/moss_archive.zip`
///   (overwrites if present)
///
/// # Returns
/// - `200 OK` with `{ success: true, data: null }` on success
/// - `404 NOT FOUND` if no `reports.txt` exists (MOSS hasn’t been run)
/// - `400 BAD REQUEST` if the report file is present but malformed (URL missing)
/// - `500 INTERNAL SERVER ERROR` for IO/network/archive errors
///
/// # Example (200)
/// ```json
/// { "success": true, "data": null, "message": "MOSS report archived and zipped successfully" }
/// ```
///
/// # Example (404)
/// ```json
/// { "success": false, "data": null, "message": "No MOSS report found. Run MOSS before archiving." }
/// ```
pub async fn generate_moss_archive(
    State(_app_state): State<AppState>,
    Path((module_id, assignment_id)): Path<(i64, i64)>,
) -> impl IntoResponse {
    // Resolve base dir and files
    let base_dir: PathBuf = assignment_dir(module_id, assignment_id);

    let report_path = base_dir.join("reports.txt");
    let dest_root   = base_dir.join("moss_archive");
    let zip_path    = base_dir.join("moss_archive.zip");

    // Must have a report first
    if !report_path.exists() {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("No MOSS report found. Run MOSS before archiving.")),
        ).into_response();
    }

    // Read & parse the URL from reports.txt
    let content = match fs::read_to_string(&report_path) {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(format!("Failed to read report file: {e}"))),
            ).into_response();
        }
    };

    let mut report_url: Option<String> = None;
    for line in content.lines() {
        if let Some(url) = line.strip_prefix("Report URL: ") {
            report_url = Some(url.trim().to_string());
            break;
        }
    }

    let report_url = match report_url {
        Some(u) if !u.is_empty() => u,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<()>::error("Malformed report file: missing report URL")),
            ).into_response();
        }
    };

    // (Re)create archive dir; remove old zip if present
    if dest_root.exists() {
        let _ = fs::remove_dir_all(&dest_root);
    }
    if zip_path.exists() {
        let _ = fs::remove_file(&zip_path);
    }
    if let Err(e) = fs::create_dir_all(&dest_root) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Failed to prepare archive directory: {e}"))),
        ).into_response();
    }

    // Mirror + zip
    let opts = ArchiveOptions { concurrency: 12 };
    match archive_moss_to_fs_and_zip(&report_url, &dest_root, &zip_path, opts).await {
        Ok((_manifest, _zip_abs)) => (
            StatusCode::OK,
            Json(ApiResponse::<()>::success_without_data("MOSS report archived and zipped successfully")),
        ).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Archiving failed: {e}"))),
        ).into_response(),
    }
}