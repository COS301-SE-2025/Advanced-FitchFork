//! Assignment grade routes.
//!
//! Provides endpoints to retrieve grades under an assignment:
//! - `GET /api/modules/{module_id}/assignments/{assignment_id}/grades` → Paginated, filterable list of grades
//! - `GET /api/modules/{module_id}/assignments/{assignment_id}/grades/export` → CSV export
//!
//! All responses follow the standard `ApiResponse` format.

use crate::{auth::AuthUser, response::ApiResponse};
use axum::{
    body::Body,
    extract::{Extension, Path, Query, State},
    http::{
        header::{CONTENT_DISPOSITION, CONTENT_TYPE},
        HeaderValue, StatusCode,
    },
    response::{IntoResponse, Response},
    Json,
};
use db::models::{
    assignment::{Column as AssignmentCol, Entity as AssignmentEntity, Model as AssignmentModel},
    assignment_submission::{
        Column as SubCol, Entity as SubmissionEntity, Model as SubModel, Relation as SubRel,
    },
    user::{Column as UserCol, Entity as UserEntity, Model as UserModel},
};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, JoinType, QueryFilter, QueryOrder,
    QuerySelect, RelationTrait,
};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;
use util::{execution_config::{execution_config::GradingPolicy, ExecutionConfig}, state::AppState};

#[derive(Debug, Deserialize)]
pub struct ListGradeQueryParams {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_per_page")]
    pub per_page: u64,
    #[serde(default)]
    pub query: Option<String>, // username contains
    #[serde(default)]
    pub sort: Option<String>, // "score", "-score", "username", "-username", "created_at", "-created_at"
}

fn default_page() -> u64 {
    1
}
fn default_per_page() -> u64 {
    20
}

#[derive(Debug, Serialize)]
pub struct GradeResponse {
    pub id: i64, // submission id chosen per policy
    pub assignment_id: i64,
    pub user_id: i64,
    pub submission_id: Option<i64>, // mirrors id for compatibility
    pub score: f32,                 // percentage 0..100
    pub username: String,
    pub created_at: String, // submission.created_at
    pub updated_at: String, // submission.updated_at
}

#[derive(Debug, Serialize)]
pub struct PaginatedGradeResponse {
    pub grades: Vec<GradeResponse>,
    pub page: u64,
    pub per_page: u64,
    pub total: u64,
}

// Internal row after applying policy
struct GradeRow {
    submission: SubModel,
    user: UserModel,
    score_pct: f32,       // 0..100
}

async fn load_execution_config_for(
    module_id: i64,
    assignment_id: i64,
) -> Result<ExecutionConfig, String> {
    ExecutionConfig::get_execution_config(module_id, assignment_id)
}

async fn compute_grades_for_assignment(
    db: &DatabaseConnection,
    module_id: i64,
    assignment_id: i64,
    username_filter: Option<&str>,
) -> Result<(Vec<GradeRow>, ExecutionConfig), sea_orm::DbErr> {
    // Ensure assignment exists under module (also lets you reuse dates later if needed)
    let _assignment: AssignmentModel = AssignmentEntity::find()
        .filter(AssignmentCol::Id.eq(assignment_id))
        .filter(AssignmentCol::ModuleId.eq(module_id))
        .one(db)
        .await?
        .ok_or_else(|| sea_orm::DbErr::Custom("Assignment not found".into()))?;

        // Load execution config from disk (propagate as DbErr::Custom on failure)
        let exec_cfg = load_execution_config_for(module_id, assignment_id)
            .await
            .map_err(|e| sea_orm::DbErr::Custom(format!("Execution config error: {e}")))?;


    // Base query: submissions for assignment, joined with user
    // - Exclude practice submissions (adjust if you want to include them)
    // - Sort to make "Last" easy: by user asc, attempt desc, created_at desc
    let mut q = SubmissionEntity::find()
        .filter(SubCol::AssignmentId.eq(assignment_id))
        .filter(SubCol::IsPractice.eq(false))
        .join(JoinType::InnerJoin, SubRel::User.def())
        .select_also(UserEntity)
        .order_by_asc(SubCol::UserId)
        .order_by_desc(SubCol::Attempt)
        .order_by_desc(SubCol::CreatedAt);

    if let Some(qs) = username_filter {
        let qs = qs.trim();
        if !qs.is_empty() {
            q = q.filter(UserCol::Username.contains(qs));
        }
    }

    let rows: Vec<(SubModel, Option<UserModel>)> = q.all(db).await?;

    // Group attempts by user
    let mut per_user: HashMap<i64, Vec<(SubModel, UserModel)>> = HashMap::new();
    for (s, u_opt) in rows {
        if let Some(u) = u_opt {
            per_user.entry(s.user_id).or_default().push((s, u));
        }
    }

    // Choose per policy
    let policy = exec_cfg.marking.grading_policy;
    let mut out = Vec::with_capacity(per_user.len());

    for (_uid, attempts) in per_user.into_iter() {
        let chosen = match policy {
            GradingPolicy::Last => {
                // Because we sorted by (user_id asc, attempt desc, created_at desc),
                // the first entry is the last/latest.
                attempts.first().cloned()
            }
            GradingPolicy::Best => attempts
                .into_iter()
                .max_by(|(a, _), (b, _)| {
                    // Compare by ratio; tie-break newer created_at, then higher attempt
                    let a_ratio = (a.earned as f64) / (a.total.max(1) as f64);
                    let b_ratio = (b.earned as f64) / (b.total.max(1) as f64);
                    match a_ratio
                        .partial_cmp(&b_ratio)
                        .unwrap_or(Ordering::Equal)
                    {
                        Ordering::Equal => match a.created_at.cmp(&b.created_at) {
                            Ordering::Equal => a.attempt.cmp(&b.attempt),
                            ord => ord,
                        },
                        ord => ord,
                    }
                }),
        };

        if let Some((s, u)) = chosen {
            let earned = s.earned; // i64 is Copy
            let total  = s.total;  // i64 is Copy
            let pct = if total <= 0 { 0.0 } else { (earned as f32) * 100.0 / (total as f32) };

            out.push(GradeRow {
                user: u,
                score_pct: pct,
                submission: s, // move happens last
            });

        }
    }

    Ok((out, exec_cfg))
}

/// GET /api/modules/{module_id}/assignments/{assignment_id}/grades
///
/// Retrieves a paginated and optionally filtered list of **derived grades** for an assignment.
/// Grades are computed directly from `assignment_submissions` based on the assignment’s
/// `ExecutionConfig.marking.grading_policy` (`Last` or `Best`).
///
/// # Arguments
///
/// Arguments are extracted automatically from the path and query parameters:
/// - `module_id`: The ID of the parent module (path parameter).
/// - `assignment_id`: The ID of the assignment (path parameter).
///
/// Query parameters are provided via [`ListGradeQueryParams`]:
/// - `page`: (Optional) The page number for pagination. Defaults to 1 if not provided. Minimum value is 1.
/// - `per_page`: (Optional) The number of items per page. Defaults to 20. Maximum is 100. Minimum is 1.
/// - `query`: (Optional) A case-insensitive search filter applied to usernames.
/// - `sort`: (Optional) A comma-separated list of fields to sort by.  
///   Prefix with `-` for descending order.  
///   Allowed sort fields: `"score"`, `"username"`, `"created_at"`.
///
/// # Grading Policy
///
/// - **Last** → Uses the most recent submission per student.
/// - **Best** → Uses the submission with the highest percentage score per student.  
///   Ties are broken by newer `created_at` or higher attempt number.
///
/// # Returns
///
/// Returns an HTTP response indicating the result:
/// - `200 OK` with a paginated list of grades, wrapped in the standard API response format.
/// - `400 BAD REQUEST` if an invalid sort field is provided.
/// - `500 INTERNAL SERVER ERROR` if grade computation fails or a database error occurs.
///
/// The response body contains:
/// - `grades`: A list of derived grades, each with submission ID, user ID, username, score percentage,
///   and submission timestamps.
/// - `page`, `per_page`, and `total` pagination metadata.
///
/// # Example Response
///
/// - `200 OK`  
/// ```json
/// {
///   "success": true,
///   "data": {
///     "grades": [
///       {
///         "id": 42,
///         "assignment_id": 7,
///         "user_id": 15,
///         "submission_id": 42,
///         "score": 85.0,
///         "username": "alice",
///         "created_at": "2025-09-01T14:30:00Z",
///         "updated_at": "2025-09-01T14:30:00Z"
///       }
///     ],
///     "page": 1,
///     "per_page": 20,
///     "total": 47
///   },
///   "message": "Grades retrieved successfully"
/// }
/// ```
///
/// - `400 Bad Request`  
/// ```json
/// {
///   "success": false,
///   "message": "Invalid sort field"
/// }
/// ```
///
/// - `500 Internal Server Error`  
/// ```json
/// {
///   "success": false,
///   "message": "Failed to compute grades"
/// }
/// ```
pub async fn list_grades(
    State(state): State<AppState>,
    Extension(_user): Extension<AuthUser>, // route already protected
    Path((module_id, assignment_id)): Path<(i64, i64)>,
    Query(params): Query<ListGradeQueryParams>,
) -> Response {
    let db = state.db();
    let page = params.page.max(1);
    let per_page = params.per_page.clamp(1, 100);

    let username_filter = params.query.as_deref();

    let (mut grades, _cfg) = match compute_grades_for_assignment(db, module_id, assignment_id, username_filter).await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("list_grades: compute error: {e}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("Failed to compute grades")),
            )
                .into_response();
        }
    };

    // Apply sorting in-memory after policy selection (we have one row per user now)
    if let Some(sort_str) = &params.sort {
        for field in sort_str.split(',') {
            let s = field.trim();
            if s.is_empty() {
                continue;
            }
            let (col, desc) = if let Some(stripped) = s.strip_prefix('-') {
                (stripped, true)
            } else {
                (s, false)
            };

            match (col, desc) {
                ("score", false) => grades.sort_by(|a, b| a.score_pct.partial_cmp(&b.score_pct).unwrap_or(Ordering::Equal)),
                ("score", true) => grades.sort_by(|a, b| b.score_pct.partial_cmp(&a.score_pct).unwrap_or(Ordering::Equal)),

                ("username", false) => grades.sort_by(|a, b| a.user.username.cmp(&b.user.username)),
                ("username", true) => grades.sort_by(|a, b| b.user.username.cmp(&a.user.username)),

                ("created_at", false) => grades.sort_by(|a, b| a.submission.created_at.cmp(&b.submission.created_at)),
                ("created_at", true) => grades.sort_by(|a, b| b.submission.created_at.cmp(&a.submission.created_at)),

                _ => {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(ApiResponse::<()>::error("Invalid sort field")),
                    )
                        .into_response();
                }
            }
        }
    } else {
        // default: username asc
        grades.sort_by(|a, b| a.user.username.cmp(&b.user.username));
    }

    // Pagination (in-memory)
    let total = grades.len() as u64;
    let start = ((page - 1) * per_page) as usize;
    let end = (start + per_page as usize).min(grades.len());
    let page_slice = if start < grades.len() { &grades[start..end] } else { &[] };

    let payload_rows: Vec<GradeResponse> = page_slice
        .iter()
        .map(|g| GradeResponse {
            id: g.submission.id,
            assignment_id,
            user_id: g.submission.user_id,
            submission_id: Some(g.submission.id),
            score: g.score_pct,
            username: g.user.username.clone(),
            created_at: g.submission.created_at.to_rfc3339(),
            updated_at: g.submission.updated_at.to_rfc3339(),
        })
        .collect();

    let payload = PaginatedGradeResponse {
        grades: payload_rows,
        page,
        per_page,
        total,
    };

    (
        StatusCode::OK,
        Json(ApiResponse::success(payload, "Grades retrieved successfully")),
    )
        .into_response()
}

/// GET /api/modules/{module_id}/assignments/{assignment_id}/grades/export
///
/// Exports **derived grades** for an assignment as a CSV file.
/// Grades are computed from `assignment_submissions` using the assignment’s
/// `ExecutionConfig.marking.grading_policy` (`Last` or `Best`).
///
/// # Arguments
///
/// Path parameters:
/// - `module_id`: The ID of the parent module.
/// - `assignment_id`: The ID of the assignment.
///
/// No query parameters are supported for this endpoint.
///
/// # Grading Policy
///
/// Same as [`list_grades`]:
/// - **Last** → Most recent submission per student.
/// - **Best** → Submission with the highest percentage score per student.
///
/// # Returns
///
/// Returns an HTTP response indicating the result:
/// - `200 OK` with a CSV file attachment containing all grades.
/// - `404 NOT FOUND` if the assignment is not found in the given module.
/// - `500 INTERNAL SERVER ERROR` if grade computation or CSV generation fails.
///
/// The CSV format contains:
/// - `username`: The student’s username.
/// - `score`: The computed score percentage (0–100).
///
/// # Example Response
///
/// - `200 OK` with headers:  
/// ```http
/// Content-Type: text/csv
/// Content-Disposition: attachment; filename="grades-assignment-7.csv"
/// ```
///
/// CSV body:
/// ```csv
/// username,score
/// alice,85.0
/// bob,92.5
/// charlie,74.0
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
///   "message": "Failed to compute grades"
/// }
/// ```
pub async fn export_grades(
    State(state): State<AppState>,
    Path((module_id, assignment_id)): Path<(i64, i64)>,
) -> Response {
    let db = state.db();

    // Ensure assignment exists in the module (cheap guard)
    match AssignmentEntity::find()
        .filter(AssignmentCol::Id.eq(assignment_id))
        .filter(AssignmentCol::ModuleId.eq(module_id))
        .one(db)
        .await
    {
        Ok(Some(_)) => {}
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<()>::error("Assignment not found")),
            )
                .into_response();
        }
        Err(e) => {
            eprintln!("export_grades: assignment lookup error: {e}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("Database error retrieving assignment")),
            )
                .into_response();
        }
    }

    let (rows, _cfg) = match compute_grades_for_assignment(db, module_id, assignment_id, None).await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("export_grades: compute error: {e}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("Failed to compute grades")),
            )
                .into_response();
        }
    };

    // Build CSV (username,score). If you need raw earned/total add columns here.
    let mut csv = String::from("username,score\n");
    for r in rows {
        // NOTE: If usernames can contain commas, add proper CSV escaping.
        csv.push_str(&format!("{},{}\n", r.user.username, r.score_pct));
    }

    let filename_hdr = format!("attachment; filename=\"grades-assignment-{}.csv\"", assignment_id);
    let content_type = HeaderValue::from_static("text/csv");
    let content_disposition = HeaderValue::from_str(&filename_hdr)
        .unwrap_or_else(|_| HeaderValue::from_static("attachment; filename=\"grades.csv\""));

    let mut builder = Response::builder();
    builder = builder
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, content_type)
        .header(CONTENT_DISPOSITION, content_disposition);

    match builder.body(Body::from(csv)) {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("export_grades: response build error: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("Failed to build CSV response")),
            )
                .into_response()
        }
    }
}
