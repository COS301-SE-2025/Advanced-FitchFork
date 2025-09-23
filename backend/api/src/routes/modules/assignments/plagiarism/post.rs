use std::fs;
use std::path::PathBuf;

use crate::services::moss_archiver::{ArchiveOptions, archive_moss_to_fs_and_zip};
use crate::{
    response::ApiResponse,
    services::moss::{MossRunOptions, MossService},
};
use axum::{
    Json,
    extract::{Path as AxumPath, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::{DateTime, Utc};
use db::models::assignment::{Entity as AssignmentEntity, Model as AssignmentModel};
use db::models::assignment_file::{
    Column as AssignmentFileCol, Entity as AssignmentFileEntity, FileType as AssignmentFileType,
};
use db::models::moss_report::{self, FilterMode as MossFilterMode};
use db::models::{
    assignment_submission::{self, Entity as SubmissionEntity},
    plagiarism_case,
    user::Entity as UserEntity,
};
use moss_parser::{ParseOptions, parse_moss};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{error, info};
use util::config;
use util::paths::{assignment_dir, ensure_parent_dir, moss_archive_zip_path};
use util::{execution_config::ExecutionConfig, state::AppState};

#[derive(Serialize, Deserialize)]
pub struct CreatePlagiarismCasePayload {
    pub submission_id_1: i64,
    pub submission_id_2: i64,
    pub description: String,
    pub similarity: f32,
    pub lines_matched: i64,
    pub report_id: Option<i64>,
}

#[derive(Serialize, Deserialize)]
pub struct PlagiarismCaseResponse {
    pub id: i64,
    pub assignment_id: i64,
    pub submission_id_1: i64,
    pub submission_id_2: i64,
    pub description: String,
    pub status: String,
    pub similarity: f32,
    pub lines_matched: i64,
    pub report_id: Option<i64>,
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
    AxumPath((_, assignment_id)): AxumPath<(i64, i64)>,
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

    if !(0.0_f32..=100.0_f32).contains(&payload.similarity) || !payload.similarity.is_finite() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(
                "Similarity must be between 0.0 and 100.0".to_string(),
            )),
        )
            .into_response();
    }

    if payload.lines_matched < 0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(
                "lines_matched must be >= 0".to_string(),
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
        payload.similarity,
        payload.lines_matched, // NEW
        payload.report_id,     // NEW
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
                    lines_matched: case.lines_matched,
                    report_id: case.report_id,
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

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
struct ArchiveManifest {
    id: String,
    created_at: DateTime<Utc>,
    report_url: String,
    root_rel: String,
    zip_rel: String,
    bytes: Option<u64>,
    files: Option<usize>,
}

#[derive(Deserialize)]
pub struct RunMossPayload {
    pub experimental: Option<bool>,
    pub max_matches: Option<u32>,
    pub show_limit: Option<u32>,
    pub filter_mode: Option<MossFilterMode>,
    pub filter_patterns: Option<Vec<String>>,
    pub description: String,
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
    AxumPath((module_id, assignment_id)): AxumPath<(i64, i64)>,
    Json(body): Json<RunMossPayload>,
) -> impl IntoResponse {
    // 0) Load assignment config
    let cfg = match ExecutionConfig::get_execution_config(module_id, assignment_id) {
        Ok(c) => c,
        Err(_) => ExecutionConfig::default_config(),
    };

    let moss_language: &str = cfg.project.language.to_moss();

    // 0.1) Assignment model
    let assignment_model: AssignmentModel = match AssignmentEntity::find_by_id(assignment_id)
        .one(app_state.db())
        .await
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

    // 1) Choose one submission per user according to policy
    let selected_submissions: Vec<assignment_submission::Model> = match SubmissionEntity::find()
        .filter(assignment_submission::Column::AssignmentId.eq(assignment_id))
        .all(app_state.db())
        .await
    {
        Ok(all_for_assignment) => {
            use std::collections::HashSet;
            let mut user_ids = HashSet::<i64>::new();
            for s in &all_for_assignment {
                user_ids.insert(s.user_id);
            }
            let mut chosen = Vec::with_capacity(user_ids.len());
            for uid in user_ids {
                if let Ok(Some(s)) = assignment_submission::Model::get_best_for_user(
                    app_state.db(),
                    &assignment_model,
                    uid,
                )
                .await
                {
                    chosen.push(s);
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

    // 2) Prepare files
    let mut submission_files = Vec::with_capacity(selected_submissions.len());
    for submission in &selected_submissions {
        let username = UserEntity::find_by_id(submission.user_id)
            .one(app_state.db())
            .await
            .ok()
            .flatten()
            .map(|u| u.username);
        submission_files.push((submission.full_path(), username, Some(submission.id)));
    }

    // 2.1) Gather skeleton/base from SPEC files
    let spec_files = AssignmentFileEntity::find()
        .filter(AssignmentFileCol::AssignmentId.eq(assignment_id))
        .filter(AssignmentFileCol::FileType.eq(AssignmentFileType::Spec))
        .all(app_state.db())
        .await
        .unwrap_or_default();

    let mut spec_zips: Vec<PathBuf> = Vec::new();
    let mut base_files: Vec<PathBuf> = Vec::new();
    for f in spec_files {
        let p = f.full_path();
        if p.extension().and_then(|s| s.to_str()) == Some("zip") {
            spec_zips.push(p);
        } else {
            base_files.push(p);
        }
    }

    // 3) Build MOSS run options (with filters from request)
    let mut opts = MossRunOptions::default();
    opts.language = moss_language.to_string();
    if let Some(v) = body.experimental {
        opts.experimental = v;
    }
    if let Some(v) = body.max_matches {
        opts.max_matches = v;
    }
    if let Some(v) = body.show_limit {
        opts.show_limit = v;
    }

    let filter_mode = body.filter_mode.unwrap_or(MossFilterMode::All);
    let filter_patterns = body.filter_patterns.clone();
    opts.filter_mode = filter_mode.clone();
    opts.filter_patterns = filter_patterns.clone();
    opts.spec_zips = spec_zips;

    // Validate filters before spawning
    if matches!(opts.filter_mode, MossFilterMode::All)
        && opts
            .filter_patterns
            .as_ref()
            .map(|v| !v.is_empty())
            .unwrap_or(false)
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(
                "filter_mode=all does not accept filter_patterns",
            )),
        )
            .into_response();
    }
    if matches!(
        opts.filter_mode,
        MossFilterMode::Whitelist | MossFilterMode::Blacklist
    ) && opts
        .filter_patterns
        .as_ref()
        .map(|v| v.is_empty())
        .unwrap_or(true)
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(
                "filter_patterns must be non-empty for whitelist/blacklist",
            )),
        )
            .into_response();
    }

    // 4) Prepare async job inputs
    let moss_user_id = config::moss_user_id();
    let moss_service = MossService::new(&moss_user_id);
    let db = app_state.db().clone();
    let started_at = Utc::now();

    // Respond immediately; background task does: run → save → parse → archive
    tokio::spawn(async move {
        let res = moss_service
            .run_with_options(base_files, submission_files, opts)
            .await;

        match res {
            Ok(report_url) => {
                // Write best-effort report pointer file
                let base_dir = assignment_dir(module_id, assignment_id);
                if let Err(e) = fs::create_dir_all(&base_dir) {
                    error!("MOSS: failed to create report dir: {e}");
                } else {
                    let report_path = base_dir.join("reports.txt");
                    let content = format!(
                        "Report URL: {}\nDate: {}",
                        &report_url,
                        Utc::now().to_rfc3339()
                    );
                    if let Err(e) = fs::write(&report_path, content) {
                        error!("MOSS: failed to write reports.txt: {e}");
                    }
                }

                // (A) Save moss_report first (description is REQUIRED now)
                // NOTE: fix arg order → (filter_mode, filter_patterns, description)
                let report_row = match moss_report::Entity::create_report(
                    &db,
                    assignment_id,
                    &report_url,
                    filter_mode.clone(),
                    body.description.clone(),
                    filter_patterns.clone(),
                )
                .await
                {
                    Ok(m) => Some(m),
                    Err(e) => {
                        error!("MOSS: failed to save moss_report: {e}");
                        None
                    }
                };

                // (B) Parse → create cases (NO deletion of prior cases; link to this report)
                let parse_opts = ParseOptions {
                    min_lines: 0,
                    include_matches: false,
                };
                match parse_moss(&report_url, parse_opts).await {
                    Ok(parsed) => {
                        use std::collections::HashSet;
                        let mut seen = HashSet::<(i64, i64)>::new(); // dedupe within THIS run only
                        let report_id_opt = report_row.as_ref().map(|m| m.id);

                        for r in parsed.reports {
                            let (Some(sub_a), Some(sub_b)) = (r.submission_id_a, r.submission_id_b)
                            else {
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
                                &ua,
                                &ub,
                                a,
                                b,
                                r.total_lines_matched,
                                r.total_percent,
                            );
                            let similarity: f32 =
                                r.total_percent.unwrap_or(0.0).clamp(0.0, 100.0) as f32;
                            let lines_matched = r.total_lines_matched.max(0);

                            // NEW signature: (similarity, lines_matched, report_id)
                            if let Err(e) = plagiarism_case::Model::create_case(
                                &db,
                                assignment_id,
                                a,
                                b,
                                &description,
                                similarity,
                                lines_matched,
                                report_id_opt,
                            )
                            .await
                            {
                                error!("MOSS: failed to create case for ({a},{b}): {e}");
                            }
                        }
                    }
                    Err(e) => error!("MOSS parse failed: {e}"),
                }

                // (C) Archive (zip) if report row exists (unchanged semantics)
                if let Some(report) = report_row {
                    let report_id_str = report.id.to_string();
                    let final_zip: PathBuf =
                        moss_archive_zip_path(module_id, assignment_id, &report_id_str);

                    let still_exists = moss_report::Entity::find_by_id(report.id)
                        .one(&db)
                        .await
                        .ok()
                        .flatten()
                        .is_some();
                    if !still_exists {
                        info!(
                            "MOSS: skipping archive; report {} was deleted before archive started",
                            report.id
                        );
                        return;
                    }

                    if let Err(e) = ensure_parent_dir(&final_zip) {
                        error!("MOSS: archive prepare failed: {e}");
                        return;
                    }

                    let work_dir: PathBuf = std::env::temp_dir().join(format!(
                        "moss_arch_{}_{}_{}",
                        module_id, assignment_id, &report_id_str
                    ));
                    if let Err(e) = fs::create_dir_all(&work_dir) {
                        error!("MOSS: failed to create temp work dir: {e}");
                        return;
                    }

                    let opts = ArchiveOptions { concurrency: 12 };
                    match archive_moss_to_fs_and_zip(&report_url, &work_dir, &final_zip, opts).await
                    {
                        Ok((_manifest, _zip_abs)) => {
                            let _ = fs::remove_dir_all(&work_dir);

                            let exists_after = moss_report::Entity::find_by_id(report.id)
                                .one(&db)
                                .await
                                .ok()
                                .flatten()
                                .is_some();

                            if !exists_after {
                                if let Err(e) = fs::remove_file(&final_zip) {
                                    error!(
                                        "MOSS: report deleted during archive; failed to remove zip {}: {e}",
                                        final_zip.display()
                                    );
                                } else {
                                    info!(
                                        "MOSS: report deleted during archive; zip removed {}",
                                        final_zip.display()
                                    );
                                }
                                return;
                            }

                            if let Err(e) = moss_report::Entity::set_archive_state(
                                &db,
                                report.id,
                                true,
                                Some(Utc::now()),
                            )
                            .await
                            {
                                error!(
                                    "MOSS: failed to update archive state for report {}: {e}",
                                    report.id
                                );
                            } else {
                                info!(
                                    "MOSS archive saved for report {}: {}",
                                    report.id,
                                    final_zip.display()
                                );
                            }
                        }
                        Err(e) => {
                            let _ = fs::remove_dir_all(&work_dir);
                            if final_zip.exists() {
                                let _ = fs::remove_file(&final_zip);
                            }
                            error!("MOSS archive failed: {e}");
                        }
                    }
                } else {
                    info!("MOSS: skipping archive because the moss_report row was not created");
                }
            }
            Err(e) => {
                error!("MOSS run failed: {e}");
            }
        }
    });

    (
        StatusCode::ACCEPTED,
        Json(ApiResponse::success(
            serde_json::json!({
                "started_at": started_at,
                "message": "MOSS job started; report will be parsed and archived when ready"
            }),
            "Started MOSS job",
        )),
    )
        .into_response()
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

#[derive(Serialize, Deserialize)]
pub struct HashScanPayload {
    #[serde(default)]
    pub create_cases: bool,
}

#[derive(Serialize, Deserialize)]
pub struct HashScanResponse {
    pub assignment_id: i64,
    pub policy_used: String,
    pub group_count: usize,
    pub groups: Vec<CollisionGroup>,
    pub cases: CreatedCases,
}

#[derive(Serialize, Deserialize)]
pub struct CollisionGroup {
    pub file_hash: String,
    pub submissions: Vec<SubmissionInfo>,
}

#[derive(Serialize, Deserialize)]
pub struct SubmissionInfo {
    pub submission_id: i64,
    pub user_id: i64,
    pub attempt: i64,
    pub filename: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
pub struct CreatedCases {
    pub created: Vec<PlagiarismCaseResponse>,
    pub skipped_existing: i64,
}

/// POST /api/modules/{module_id}/assignments/{assignment_id}/plagiarism/hash-scan
///
/// Performs a hash-based collision scan to detect potentially identical submissions
/// within an assignment. This endpoint identifies submissions with identical file
/// hashes, which typically indicates exact file duplication or copy-paste plagiarism.
///
/// Accessible only to lecturers and assistant lecturers assigned to the module.
///
/// # Path Parameters
///
/// - `module_id`: The ID of the parent module
/// - `assignment_id`: The ID of the assignment to scan
///
/// # Request Body
///
/// Requires a JSON payload with the following optional field:
/// - `create_cases` (boolean, default: false): Whether to automatically create plagiarism
///   cases for detected hash collisions. When true, creates cases with 100% similarity
///   for each pair of submissions with identical hashes.
///
/// # Behavior
///
/// - **Submission selection respects the assignment's grading policy**:
///   - `grading_policy = "last"` → uses each student's **most recent** non-practice, non-ignored submission
///   - `grading_policy = "best"` → uses each student's **best-scoring** non-practice, non-ignored submission
/// - Groups submissions by their SHA-256 file hash
/// - Only includes groups with 2+ submissions (actual collisions)
/// - Filters out submissions with empty/missing file hashes
/// - When `create_cases = true`:
///   - Creates plagiarism cases for each unique pair within collision groups
///   - Skips pairs from the same user (self-collision)
///   - Deduplicates against existing cases (order-independent)
///   - Sets similarity to 100.0% and lines_matched to 0
///   - Uses description: "Identical file hash collision"
///
/// # Returns
///
/// - `200 OK` on success with collision analysis and optional case creation results
/// - `404 NOT FOUND` if the assignment doesn't exist
/// - `403 FORBIDDEN` if user lacks required permissions
/// - `500 INTERNAL SERVER ERROR` for database errors or processing failures
///
/// # Example Request
///
/// ```json
/// {
///   "create_cases": true
/// }
/// ```
///
/// # Example Response (200 OK)
///
/// ```json
/// {
///   "success": true,
///   "message": "Hash scan complete.",
///   "data": {
///     "assignment_id": 42,
///     "policy_used": "Best",
///     "group_count": 3,
///     "groups": [
///       {
///         "file_hash": "a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456",
///         "submissions": [
///           {
///             "submission_id": 101,
///             "user_id": 15,
///             "attempt": 2,
///             "filename": "solution.py",
///             "created_at": "2024-05-20T14:30:00Z"
///           },
///           {
///             "submission_id": 108,
///             "user_id": 23,
///             "attempt": 1,
///             "filename": "main.py",
///             "created_at": "2024-05-20T15:45:00Z"
///           }
///         ]
///       }
///     ],
///     "cases": {
///       "created": [
///         {
///           "id": 67,
///           "assignment_id": 42,
///           "submission_id_1": 101,
///           "submission_id_2": 108,
///           "description": "Identical file hash collision",
///           "status": "review",
///           "similarity": 100.0,
///           "lines_matched": 0,
///           "report_id": null,
///           "created_at": "2024-05-20T16:00:00Z",
///           "updated_at": "2024-05-20T16:00:00Z"
///         }
///       ],
///       "skipped_existing": 0
///     }
///   }
/// }
/// ```
///
/// # Example Error Responses
///
/// - `404 Not Found`
/// ```json
/// { "success": false, "message": "Assignment not found." }
/// ```
///
/// - `500 Internal Server Error`
/// ```json
/// { "success": false, "message": "Failed to retrieve submissions." }
/// ```
///
/// # Notes
///
/// - Hash collisions indicate **exact file duplication**, which is stronger evidence
///   of plagiarism than similarity-based detection methods
/// - This scan is complementary to MOSS-based similarity detection
/// - Empty or missing file hashes are excluded from analysis
/// - Created cases start in "review" status and can be managed via plagiarism APIs/UI
/// - The scan only considers the selected submission per user based on grading policy
/// - Practice and ignored submissions are automatically excluded
pub async fn hash_scan(
    State(app_state): State<AppState>,
    AxumPath((module_id, assignment_id)): AxumPath<(i64, i64)>,
    Json(payload): Json<HashScanPayload>,
) -> impl IntoResponse {
    let assignment = match AssignmentEntity::find_by_id(assignment_id)
        .one(app_state.db())
        .await
    {
        Ok(Some(a)) => a,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<()>::error("Assignment not found.")),
            )
                .into_response();
        }
        Err(e) => {
            error!("Failed to fetch assignment: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("Internal server error.")),
            )
                .into_response();
        }
    };

    let exec_config = ExecutionConfig::get_execution_config(module_id, assignment_id)
        .unwrap_or_else(|_| ExecutionConfig::default_config());
    let policy_used = match exec_config.marking.grading_policy {
        util::execution_config::GradingPolicy::Best => "Best",
        util::execution_config::GradingPolicy::Last => "Last",
    }
    .to_string();

    let selected_submissions: Vec<assignment_submission::Model> =
        match assignment_submission::Model::get_selected_submissions_for_assignment(
            app_state.db(),
            &assignment,
        )
        .await
        {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to retrieve submissions: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()>::error("Failed to retrieve submissions.")),
                )
                    .into_response();
            }
        };

    let mut groups_map: HashMap<String, Vec<assignment_submission::Model>> = HashMap::new();
    for s in selected_submissions
        .into_iter()
        .filter(|s| !s.file_hash.is_empty())
    {
        groups_map.entry(s.file_hash.clone()).or_default().push(s);
    }

    let groups: Vec<CollisionGroup> = groups_map
        .into_iter()
        .filter(|(_, v)| v.len() >= 2)
        .map(|(file_hash, submissions_vec)| CollisionGroup {
            file_hash,
            submissions: submissions_vec
                .into_iter()
                .map(|s| SubmissionInfo {
                    submission_id: s.id,
                    user_id: s.user_id,
                    attempt: s.attempt,
                    filename: s.filename,
                    created_at: s.created_at,
                })
                .collect(),
        })
        .collect();

    let cases = if payload.create_cases {
        let mut created_cases = vec![];
        let mut skipped_existing = 0;

        for group in &groups {
            for (i, s1) in group.submissions.iter().enumerate() {
                for s2 in group.submissions.iter().skip(i + 1) {
                    if s1.user_id == s2.user_id {
                        continue;
                    }

                    let submission_id_1 = std::cmp::min(s1.submission_id, s2.submission_id);
                    let submission_id_2 = std::cmp::max(s1.submission_id, s2.submission_id);

                    let existing_case = match plagiarism_case::Entity::find()
                        .filter(
                            sea_orm::Condition::all()
                                .add(plagiarism_case::Column::SubmissionId1.eq(submission_id_1))
                                .add(plagiarism_case::Column::SubmissionId2.eq(submission_id_2)),
                        )
                        .one(app_state.db())
                        .await
                    {
                        Ok(case) => case,
                        Err(e) => {
                            error!("Failed to check for existing plagiarism case: {}", e);
                            return (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                Json(ApiResponse::<()>::error(
                                    "Failed to check for existing plagiarism case.",
                                )),
                            )
                                .into_response();
                        }
                    };

                    if existing_case.is_some() {
                        skipped_existing += 1;
                        continue;
                    }

                    let new_case = match plagiarism_case::Model::create_case(
                        app_state.db(),
                        assignment_id,
                        submission_id_1,
                        submission_id_2,
                        "Identical file hash collision",
                        100.0,
                        0,
                        None,
                    )
                    .await
                    {
                        Ok(case) => case,
                        Err(e) => {
                            error!("Failed to create plagiarism case: {}", e);
                            return (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                Json(ApiResponse::<()>::error(
                                    "Failed to create plagiarism case.",
                                )),
                            )
                                .into_response();
                        }
                    };

                    created_cases.push(new_case);
                }
            }
        }

        CreatedCases {
            created: created_cases
                .into_iter()
                .map(|case| PlagiarismCaseResponse {
                    id: case.id,
                    assignment_id: case.assignment_id,
                    submission_id_1: case.submission_id_1,
                    submission_id_2: case.submission_id_2,
                    description: case.description,
                    status: case.status.to_string(),
                    similarity: case.similarity,
                    lines_matched: case.lines_matched,
                    report_id: case.report_id,
                    created_at: case.created_at,
                    updated_at: case.updated_at,
                })
                .collect(),
            skipped_existing,
        }
    } else {
        CreatedCases {
            created: vec![],
            skipped_existing: 0,
        }
    };

    (
        StatusCode::OK,
        Json(ApiResponse::success(
            HashScanResponse {
                assignment_id,
                policy_used,
                group_count: groups.len(),
                groups,
                cases,
            },
            "Hash scan complete.",
        )),
    )
        .into_response()
}
