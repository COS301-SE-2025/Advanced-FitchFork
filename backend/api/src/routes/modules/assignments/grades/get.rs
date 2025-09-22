//! Assignment grade routes.
//!
//! Provides endpoints to retrieve grades under an assignment (students only):
//! - `GET /api/modules/{module_id}/assignments/{assignment_id}/grades` → Paginated, filterable list of grades
//!   (includes per-task marks from the submission report for the chosen submission)
//! - `GET /api/modules/{module_id}/assignments/{assignment_id}/grades/export` → CSV export (final score only)
//!
//! All responses follow the standard `ApiResponse` format.

use crate::{auth::AuthUser, response::ApiResponse};
use axum::{
    Json,
    body::Body,
    extract::{Extension, Path, Query},
    http::{
        HeaderValue, StatusCode,
        header::{CONTENT_DISPOSITION, CONTENT_TYPE},
    },
    response::{IntoResponse, Response},
};
use db::models::{
    assignment::{Column as AssignmentCol, Entity as AssignmentEntity, Model as AssignmentModel},
    assignment_submission::{
        Column as SubCol, Entity as SubmissionEntity, Model as SubModel, Relation as SubRel,
    },
    assignment_task::{Column as TaskCol, Entity as TaskEntity, Model as TaskModel},
    user::{Column as UserCol, Entity as UserEntity, Model as UserModel},
    user_module_role::{Column as UmrCol, Entity as UmrEntity, Role as ModuleRole},
};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, JoinType, QueryFilter, QueryOrder, QuerySelect,
    QueryTrait, RelationTrait,
};
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, collections::HashMap};
use util::state::AppState;
use util::{
    execution_config::{ExecutionConfig, GradingPolicy},
    paths::submission_report_path,
};

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

/// Per-task score row returned in the list endpoint
#[derive(Debug, Serialize)]
pub struct TaskBreakdown {
    pub task_number: Option<i64>,
    pub name: Option<String>,
    pub earned: i64,
    pub total: i64,
    pub score: f32, // percentage 0..100
}

#[derive(Debug, Serialize)]
pub struct GradeResponse {
    pub id: i64, // submission id chosen per policy
    pub assignment_id: i64,
    pub user_id: i64,
    pub submission_id: Option<i64>, // mirrors id for compatibility
    pub score: f32,                 // percentage 0..100 (final)
    pub username: String,
    pub created_at: String, // submission.created_at
    pub updated_at: String, // submission.updated_at
    pub tasks: Vec<TaskBreakdown>,
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
    score_pct: f32, // 0..100
}

/// Minimal shapes we need from submission_report.json
#[derive(Debug, Deserialize)]
struct ReportScore {
    earned: i64,
    total: i64,
}

#[derive(Debug, Deserialize)]
struct ReportTask {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    task_number: Option<i64>,
    #[serde(default)]
    score: Option<ReportScore>,
}

#[derive(Debug, Deserialize)]
struct ReportRoot {
    #[serde(default)]
    tasks: Vec<ReportTask>,
}

fn pct(earned: i64, total: i64) -> f32 {
    if total <= 0 {
        0.0
    } else {
        (earned as f32) * 100.0 / (total as f32)
    }
}

/// Build maps for task name enrichment:
///  - by_id:  assignment_task.id          -> name
///  - by_num: assignment_task.task_number -> name
async fn load_task_maps(
    db: &DatabaseConnection,
    assignment_id: i64,
) -> Result<(HashMap<i64, String>, HashMap<i64, String>), sea_orm::DbErr> {
    let rows: Vec<TaskModel> = TaskEntity::find()
        .filter(TaskCol::AssignmentId.eq(assignment_id))
        .all(db)
        .await?;

    let mut by_id = HashMap::with_capacity(rows.len());
    let mut by_num = HashMap::with_capacity(rows.len());
    for t in rows {
        by_id.insert(t.id, t.name.clone());
        by_num.insert(t.task_number, t.name);
    }
    Ok((by_id, by_num))
}

/// Read per-task scores from submission_report.json for a chosen submission.
/// If a task "name" looks like a numeric string, it's treated as a task_id and
/// resolved via `task_name_by_id`. If that fails but `task_number` is present,
/// resolve via `task_name_by_num`. Otherwise keep the original string.
/// If the file/shape is missing or invalid, returns an empty vector gracefully.
fn read_tasks_from_report(
    module_id: i64,
    assignment_id: i64,
    submission: &SubModel,
    task_name_by_id: &HashMap<i64, String>,
    task_name_by_num: &HashMap<i64, String>,
) -> Vec<TaskBreakdown> {
    let path = submission_report_path(
        module_id,
        assignment_id,
        submission.user_id,
        submission.attempt,
    );

    let Ok(raw) = std::fs::read_to_string(&path) else {
        return vec![];
    };

    match serde_json::from_str::<ReportRoot>(&raw) {
        Ok(rep) => rep
            .tasks
            .into_iter()
            .filter_map(|t| {
                let Some(sc) = t.score else {
                    return None;
                };

                // 1) Try name as task_id (numeric string)
                // 2) Fallback to task_number mapping
                // 3) Else keep the original string (if any)
                let resolved_name: Option<String> = t
                    .name
                    .as_ref()
                    .and_then(|s| s.trim().parse::<i64>().ok())
                    .and_then(|id| task_name_by_id.get(&id).cloned())
                    .or_else(|| {
                        t.task_number
                            .and_then(|n| task_name_by_num.get(&n).cloned())
                    })
                    .or_else(|| t.name.clone());

                Some(TaskBreakdown {
                    task_number: t.task_number,
                    name: resolved_name,
                    earned: sc.earned,
                    total: sc.total,
                    score: pct(sc.earned, sc.total),
                })
            })
            .collect(),
        Err(_) => vec![],
    }
}

async fn compute_grades_for_assignment(
    module_id: i64,
    assignment_id: i64,
    username_filter: Option<&str>,
) -> Result<(Vec<GradeRow>, ExecutionConfig), sea_orm::DbErr> {
    // Ensure assignment exists under module
    let _assignment: AssignmentModel = AssignmentEntity::find()
        .filter(AssignmentCol::Id.eq(assignment_id))
        .filter(AssignmentCol::ModuleId.eq(module_id))
        .one(db)
        .await?
        .ok_or_else(|| sea_orm::DbErr::Custom("Assignment not found".into()))?;

    // Load execution config
    let exec_cfg = load_execution_config_for(module_id, assignment_id)
        .await
        .map_err(|e| sea_orm::DbErr::Custom(format!("Execution config error: {e}")))?;

    // ---- ONLY STUDENTS: user_ids with role=Student in this module ----
    let student_ids_subq = UmrEntity::find()
        .select_only()
        .column(UmrCol::UserId)
        .filter(UmrCol::ModuleId.eq(module_id))
        .filter(UmrCol::Role.eq(ModuleRole::Student));

    // Base query
    let mut q = SubmissionEntity::find()
        .filter(SubCol::AssignmentId.eq(assignment_id))
        .filter(SubCol::IsPractice.eq(false))
        .filter(SubCol::Ignored.eq(false))
        .filter(SubCol::UserId.in_subquery(student_ids_subq.as_query().to_owned()))
        .join(JoinType::InnerJoin, SubRel::User.def())
        .select_also(UserEntity)
        .order_by_asc(SubCol::UserId)
        .order_by_desc(SubCol::CreatedAt)
        .order_by_desc(SubCol::Attempt);

    if let Some(qs) = username_filter {
        let qs = qs.trim();
        if !qs.is_empty() {
            q = q.filter(UserCol::Username.contains(qs));
        }
    }

    let rows: Vec<(SubModel, Option<UserModel>)> = q.all(db).await?;

    // Group attempts by user
    let mut per_user: HashMap<i64, Vec<(AssignmentSubmission, User)>> = HashMap::new();
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
            GradingPolicy::Last => attempts.first().cloned(),
            GradingPolicy::Best => attempts.into_iter().max_by(|(a, _), (b, _)| {
                let a_ratio = (a.earned as f64) / (a.total.max(1) as f64);
                let b_ratio = (b.earned as f64) / (b.total.max(1) as f64);
                match a_ratio.partial_cmp(&b_ratio).unwrap_or(Ordering::Equal) {
                    Ordering::Equal => match a.created_at.cmp(&b.created_at) {
                        Ordering::Equal => a.attempt.cmp(&b.attempt),
                        ord => ord,
                    },
                    ord => ord,
                }
            }),
        };

        if let Some((s, u)) = chosen {
            let pct_final = pct(s.earned, s.total);
            out.push(GradeRow {
                user: u,
                score_pct: pct_final,
                submission: s,
            });
        }
    }

    Ok((out, exec_cfg))
}

/// GET /api/modules/{module_id}/assignments/{assignment_id}/grades
///
/// Returns a **paginated** list of **student** grades for an assignment, computed from
/// `assignment_submissions` using the assignment’s grading policy (**best** or **last**)
/// as defined in its `ExecutionConfig`.
///
/// Staff submissions (lecturer/assistant_lecturer/tutor/admin) are **excluded** via the
/// `user_module_roles` table — only users with role **student** in the given module
/// are considered. **Practice** submissions are also excluded.
///
/// Each grade row represents the single *chosen* submission for that student (per policy).
/// It includes the **final percentage score** and a **per-task breakdown** parsed from the
/// student’s `submission_report.json`.
///
/// #### Task name enrichment
/// If a task’s `"name"` in the report is a numeric string (e.g. `"123"`), it is treated as
/// `assignment_task.id` and **resolved to the real task name** by looking it up in
/// `assignment_tasks`. If it’s not numeric, the report’s string is used as-is.
///
/// ---
///
/// # Path parameters
/// - `module_id` — Module ID
/// - `assignment_id` — Assignment ID
///
/// # Query parameters
/// - `page` *(u64, default=1)* — Page number (min 1)
/// - `per_page` *(u64, default=20, max=100)* — Page size
/// - `query` *(string)* — Substring filter on `username`
/// - `sort` *(string)* — Comma-separated order specifiers:
///   - Allowed fields: `"score"`, `"username"`, `"created_at"`
///   - Prefix with `-` for descending. Examples: `"score"`, `"-score"`, `"username,-created_at"`
///   - If omitted, default sort is by `username` ascending.
///   - Sorting happens **in-memory after policy selection** (one row per student).
///
/// # Grading policy
/// - **Last** — Uses the most recent submission per student (by `created_at`, then `attempt`)
/// - **Best** — Uses the submission with the highest ratio `earned/total`
///   (ties broken by newer `created_at`, then higher `attempt`)
///
/// # Response (200 OK)
/// ```json
/// {
///   "success": true,
///   "message": "Grades retrieved successfully",
///   "data": {
///     "grades": [
///       {
///         "id": 191,
///         "assignment_id": 7,
///         "user_id": 42,
///         "submission_id": 191,
///         "score": 77.77778,
///         "username": "student1",
///         "created_at": "2025-09-06T20:33:30.843484Z",
///         "updated_at": "2025-09-06T20:33:30.843484Z",
///         "tasks": [
///           { "task_number": 1, "name": "FizzBuzz",   "earned": 8, "total": 9, "score": 88.88889 },
///           { "task_number": 2, "name": "Palindrome", "earned": 8, "total": 9, "score": 88.88889 },
///           { "task_number": 3, "name": "Sorting",    "earned": 5, "total": 9, "score": 55.55556 }
///         ]
///       }
///     ],
///     "page": 1,
///     "per_page": 20,
///     "total": 47
///   }
/// }
/// ```
///
/// # Errors
/// - **400 Bad Request** — `sort` includes an unsupported field
/// - **500 Internal Server Error** — Failed to compute grades (includes cases where the
///   assignment does not exist under the module), or an I/O/JSON error while reading reports
///
/// # Notes
/// - If the report is missing or malformed, `tasks` is returned as an empty array for that row.
/// - `score` is computed as `(earned / total) * 100`. Practice submissions are ignored.
/// - Only **students** (role `student` in `user_module_roles`) are included.
/// - Pagination and sorting are done **after** policy selection (one row per student).
///
/// # Example
/// `GET /api/modules/12/assignments/34/grades?query=stud&sort=-score,username&page=1&per_page=50`
pub async fn list_grades(
    Extension(_): Extension<AuthUser>,
    Path((module_id, assignment_id)): Path<(i64, i64)>,
    Query(params): Query<ListGradeQueryParams>,
) -> Response {
    let page = params.page.max(1);
    let per_page = params.per_page.clamp(1, 100);

    let username_filter = params.query.as_deref();

    let (mut grades, _cfg) =
        match compute_grades_for_assignment(db, module_id, assignment_id, username_filter).await {
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

    // Load task name map once
    let (task_name_by_id, task_name_by_num) = match load_task_maps(db, assignment_id).await {
        Ok(maps) => maps,
        Err(e) => {
            eprintln!("list_grades: task maps load error: {e}");
            (HashMap::new(), HashMap::new())
        }
    };

    // Apply sorting
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
                ("score", false) => grades.sort_by(|a, b| {
                    a.score_pct
                        .partial_cmp(&b.score_pct)
                        .unwrap_or(Ordering::Equal)
                }),
                ("score", true) => grades.sort_by(|a, b| {
                    b.score_pct
                        .partial_cmp(&a.score_pct)
                        .unwrap_or(Ordering::Equal)
                }),

                ("username", false) => grades.sort_by(|a, b| a.user.username.cmp(&b.user.username)),
                ("username", true) => grades.sort_by(|a, b| b.user.username.cmp(&a.user.username)),

                ("created_at", false) => {
                    grades.sort_by(|a, b| a.submission.created_at.cmp(&b.submission.created_at))
                }
                ("created_at", true) => {
                    grades.sort_by(|a, b| b.submission.created_at.cmp(&a.submission.created_at))
                }

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
    let page_slice = if start < grades.len() {
        &grades[start..end]
    } else {
        &[]
    };

    let payload_rows: Vec<GradeResponse> = page_slice
        .iter()
        .map(|g| {
            let tasks = read_tasks_from_report(
                module_id,
                assignment_id,
                &g.submission,
                &task_name_by_id,
                &task_name_by_num,
            );
            GradeResponse {
                id: g.submission.id,
                assignment_id,
                user_id: g.submission.user_id,
                submission_id: Some(g.submission.id),
                score: g.score_pct,
                username: g.user.username.clone(),
                created_at: g.submission.created_at.to_rfc3339(),
                updated_at: g.submission.updated_at.to_rfc3339(),
                tasks,
            }
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
        Json(ApiResponse::success(
            payload,
            "Grades retrieved successfully",
        )),
    )
        .into_response()
}

/// GET /api/modules/{module_id}/assignments/{assignment_id}/grades/export
///
/// Exports **student** grades for an assignment as a **CSV attachment**.
/// Uses the assignment’s grading policy (**best/last**) to pick **one submission per student**,
/// then outputs:
/// - `username`
/// - `score` (final %)
/// - **one column per task** (task percentage only)
///
/// #### Task columns (union & naming)
/// Task columns are the **union** of all tasks seen across the chosen submissions,
/// in the order they are first encountered. Each column header is:
/// 1) the resolved task **name** (see enrichment note below), else
/// 2) `task{task_number}` if present, else
/// 3) `task_{index}` (1-based fallback).
///
/// Missing tasks for a given student → **empty cell**.
///
/// #### Task name enrichment
/// If a task `"name"` inside `submission_report.json` is a numeric string (e.g. `"123"`),
/// it is treated as `assignment_task.id` and **resolved to the real task name** (`assignment_tasks.name`).
/// Otherwise the report’s string is used as-is.
///
/// ---
///
/// # Path parameters
/// - `module_id` — Module ID
/// - `assignment_id` — Assignment ID
///
/// # Response (200 OK)
/// Content-Type: `text/csv`  
/// Content-Disposition: `attachment; filename="grades-assignment-{assignment_id}.csv"`
///
/// **Header:**
/// ```csv
/// username,score,FizzBuzz,Palindrome,Sorting
/// ```
///
/// **Rows:**
/// ```csv
/// student1,77.77778,88.88889,88.88889,55.55556
/// student2,88.88889,100,77.77778,
/// ```
///
/// # Errors
/// - **404 Not Found** — Assignment doesn’t exist under the given module
/// - **500 Internal Server Error** — Database error, compute error, or response build error
///
/// # Notes
/// - Staff submissions are excluded; only students (role `student` in `user_module_roles`) are exported.
/// - Practice submissions are ignored.
/// - Task columns are derived **after** task-name enrichment (numeric `"name"` → lookup in `assignment_tasks`).
/// - Numeric values are written using Rust’s default `to_string()` for `f32`.
///
/// # Example
/// `GET /api/modules/12/assignments/34/grades/export` → downloads a CSV with union task columns.
pub async fn export_grades(Path((module_id, assignment_id)): Path<(i64, i64)>) -> Response {
    let db = state.db();

    // Guard: assignment exists in module
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
                Json(ApiResponse::<()>::error(
                    "Database error retrieving assignment",
                )),
            )
                .into_response();
        }
    }

    // Per-policy chosen submissions (students only)
    let (rows, _cfg) = match compute_grades_for_assignment(db, module_id, assignment_id, None).await
    {
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

    // Load task name map once
    let (task_name_by_id, task_name_by_num) = match load_task_maps(db, assignment_id).await {
        Ok(maps) => maps,
        Err(e) => {
            eprintln!("export_grades: task maps load error: {e}");
            (HashMap::new(), HashMap::new())
        }
    };

    // Helpers
    fn task_label(t: &TaskBreakdown, fallback_index: usize) -> String {
        if let Some(name) = &t.name {
            if !name.trim().is_empty() {
                return name.clone();
            }
        }
        if let Some(n) = t.task_number {
            return format!("task{}", n);
        }
        format!("task_{}", fallback_index + 1)
    }

    fn csv_escape(s: &str) -> String {
        if s.contains([',', '"', '\n', '\r']) {
            let mut out = String::with_capacity(s.len() + 2);
            out.push('"');
            for ch in s.chars() {
                if ch == '"' {
                    out.push('"'); // escape quotes
                }
                out.push(ch);
            }
            out.push('"');
            out
        } else {
            s.to_string()
        }
    }

    use std::collections::{HashMap as Map, HashSet};
    let mut task_order: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    // Cache per row: username, final %, map<label -> pct>
    let mut row_task_maps: Vec<(String, f32, Map<String, f32>)> = Vec::with_capacity(rows.len());

    for r in &rows {
        // Read per-task from report for this chosen submission (with name resolution)
        let tasks = read_tasks_from_report(
            module_id,
            assignment_id,
            &r.submission,
            &task_name_by_id,
            &task_name_by_num,
        );

        let mut map: Map<String, f32> = Map::new();
        for (idx, t) in tasks.iter().enumerate() {
            let label = task_label(t, idx);
            let p = pct(t.earned, t.total);
            if seen.insert(label.clone()) {
                task_order.push(label.clone());
            }
            map.insert(label, p);
        }

        row_task_maps.push((r.user.username.clone(), r.score_pct, map));
    }

    // Build CSV: username,score,<task1>,<task2>,...
    let mut csv = String::new();
    csv.push_str("username,score");
    for label in &task_order {
        csv.push(',');
        csv.push_str(&csv_escape(label));
    }
    csv.push('\n');

    for (username, final_score, taskmap) in row_task_maps {
        csv.push_str(&csv_escape(&username));
        csv.push(',');
        csv.push_str(&final_score.to_string());

        for label in &task_order {
            csv.push(',');
            if let Some(p) = taskmap.get(label) {
                csv.push_str(&p.to_string());
            } // else leave empty cell
        }
        csv.push('\n');
    }

    let filename_hdr = format!(
        "attachment; filename=\"grades-assignment-{}.csv\"",
        assignment_id
    );
    let content_type = HeaderValue::from_static("text/csv");
    let content_disposition = HeaderValue::from_str(&filename_hdr)
        .unwrap_or_else(|_| HeaderValue::from_static("attachment; filename=\"grades.csv\""));

    match Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, content_type)
        .header(CONTENT_DISPOSITION, content_disposition)
        .body(Body::from(csv))
    {
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
