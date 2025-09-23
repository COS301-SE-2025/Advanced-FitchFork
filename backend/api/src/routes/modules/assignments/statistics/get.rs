use crate::{auth::AuthUser, response::ApiResponse};
use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};
use chrono::{DateTime, Utc};
use db::models::{
    assignment::{Column as AssignmentColumn, Entity as AssignmentEntity},
    assignment_submission::Model as SubmissionModel,
    assignment_submission::{self, Entity as SubmissionEntity},
    user_module_role::{Column as UMRCol, Entity as UMREntity, Role as UMRRole},
};
use sea_orm::{ColumnTrait, EntityTrait, Order, QueryFilter, QueryOrder};
use serde::Serialize;
use serde_json::Value;
use util::{
    execution_config::{ExecutionConfig, GradingPolicy},
    paths::submission_report_path,
    state::AppState,
};

// ---------- Response DTO ----------

#[derive(Debug, Serialize)]
pub struct AssignmentStatsResponse {
    // headline
    pub total: usize,
    pub graded: usize,
    pub pending: usize,
    pub pass_rate: f64, // %
    pub avg_mark: f64,  // %
    pub median: f64,    // %
    pub p75: f64,       // %
    pub stddev: f64,
    pub best: i64,  // %
    pub worst: i64, // %

    // extras
    pub total_marks: f32, // sum of "total" across subs
    pub num_students_submitted: usize,
    pub num_passed: usize,
    pub num_failed: usize,
    pub num_full_marks: usize,

    // flags
    pub late: usize,
    pub on_time: usize,
    pub ignored: usize,
}

// ---------- Helpers ----------

#[inline]
fn to_pct(earned: Option<i64>, total: Option<i64>) -> Option<i64> {
    match (earned, total) {
        (Some(e), Some(t)) if t > 0 => Some(((e as f64 / t as f64) * 100.0).round() as i64),
        _ => None,
    }
}

fn mean(xs: &[i64]) -> f64 {
    if xs.is_empty() {
        return 0.0;
    }
    xs.iter().map(|&x| x as f64).sum::<f64>() / xs.len() as f64
}

fn median(xs: &mut [i64]) -> f64 {
    if xs.is_empty() {
        return 0.0;
    }
    xs.sort_unstable();
    let n = xs.len();
    if n % 2 == 1 {
        xs[n / 2] as f64
    } else {
        (xs[n / 2 - 1] as f64 + xs[n / 2] as f64) / 2.0
    }
}

fn percentile(xs: &mut [i64], p: f64) -> f64 {
    if xs.is_empty() {
        return 0.0;
    }
    xs.sort_unstable();
    let pos = (p * ((xs.len() - 1) as f64)).clamp(0.0, (xs.len() - 1) as f64);
    let lo = pos.floor() as usize;
    let hi = pos.ceil() as usize;
    if lo == hi {
        xs[lo] as f64
    } else {
        let w = pos - lo as f64;
        xs[lo] as f64 * (1.0 - w) + xs[hi] as f64 * w
    }
}

fn stddev(xs: &[i64]) -> f64 {
    if xs.len() < 2 {
        return 0.0;
    }
    let m = mean(xs);
    let var = xs
        .iter()
        .map(|&x| {
            let d = x as f64 - m;
            d * d
        })
        .sum::<f64>()
        / (xs.len() as f64 - 1.0);
    var.sqrt()
}

#[inline]
fn is_late(created_at: DateTime<Utc>, due: Option<DateTime<Utc>>) -> bool {
    match due {
        Some(d) => created_at > d,
        None => false,
    }
}

/// GET /api/modules/{module_id}/assignments/{assignment_id}/stats
///
/// Retrieve **aggregated submission statistics** for an assignment.
/// **Only student submissions are included** — staff submissions
/// (lecturer, assistant lecturer, tutor, or admin) are ignored in all
/// calculations.
///
/// Access: **lecturer / assistant lecturer / admin**
///
/// The response summarizes submission counts, pass/fail metrics,
/// and distribution statistics (mean, median, percentiles, stddev)
/// according to the assignment’s grading policy (`best` or `last`).
///
/// ### Path Parameters
/// - `module_id` (i64): Module ID  
/// - `assignment_id` (i64): Assignment ID
///
/// ### 200 OK
/// ```json
/// {
///   "success": true,
///   "message": "Submission stats computed",
///   "data": {
///     "total": 7,
///     "graded": 3,
///     "pending": 4,
///     "pass_rate": 66.7,
///     "avg_mark": 68.3,
///     "median": 70.0,
///     "p75": 80.0,
///     "stddev": 22.5,
///     "best": 90,
///     "worst": 45,
///     "total_marks": 700,
///     "num_students_submitted": 3,
///     "num_passed": 2,
///     "num_failed": 1,
///     "num_full_marks": 0,
///     "late": 2,
///     "on_time": 5,
///     "ignored": 0
///   }
/// }
/// ```
///
/// ### Metrics
/// - **total** — number of submission attempts (students only)  
/// - **graded** — distinct students whose effective mark was computed  
/// - **pending** — submissions without a valid effective mark  
/// - **pass_rate** — % of graded students with mark ≥ pass mark  
/// - **avg_mark / median / p75 / stddev** — distribution of effective marks  
/// - **best / worst** — highest and lowest effective marks  
/// - **total_marks** — sum of `mark.total` across student submissions  
/// - **num_students_submitted** — number of distinct students who submitted  
/// - **num_passed / num_failed** — counts by pass/fail threshold  
/// - **num_full_marks** — number of students with 100%  
/// - **late / on_time / ignored** — submission flags  
///
/// ### Errors
/// - **404 Not Found** — Assignment not found  
/// - **500 Internal Server Error** — Database or read error
pub async fn get_assignment_stats(
    State(app_state): State<AppState>,
    Path((module_id, assignment_id)): Path<(i64, i64)>,
    _auth: AuthUser, // present due to auth middleware
) -> axum::response::Response {
    let db = app_state.db();

    // Validate assignment (and grab config + due_date)
    let assignment = match AssignmentEntity::find()
        .filter(AssignmentColumn::Id.eq(assignment_id))
        .filter(AssignmentColumn::ModuleId.eq(module_id))
        .one(db)
        .await
    {
        Ok(Some(a)) => a,
        Ok(None) => {
            return (
                axum::http::StatusCode::NOT_FOUND,
                Json(ApiResponse::<AssignmentStatsResponse>::error(
                    "Assignment not found",
                )),
            )
                .into_response();
        }
        Err(_) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<AssignmentStatsResponse>::error(
                    "Database error",
                )),
            )
                .into_response();
        }
    };

    // Determine student user_ids in this module
    let student_ids: Vec<i64> = match UMREntity::find()
        .filter(UMRCol::ModuleId.eq(module_id))
        .filter(UMRCol::Role.eq(UMRRole::Student))
        .all(db)
        .await
    {
        Ok(rows) => rows.into_iter().map(|r| r.user_id).collect(),
        Err(_) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<AssignmentStatsResponse>::error(
                    "Database error",
                )),
            )
                .into_response();
        }
    };

    if student_ids.is_empty() {
        let resp = AssignmentStatsResponse {
            total: 0,
            graded: 0,
            pending: 0,
            pass_rate: 0.0,
            avg_mark: 0.0,
            median: 0.0,
            p75: 0.0,
            stddev: 0.0,
            best: 0,
            worst: 0,
            total_marks: 0.0,
            num_students_submitted: 0,
            num_passed: 0,
            num_failed: 0,
            num_full_marks: 0,
            late: 0,
            on_time: 0,
            ignored: 0,
        };
        return (
            axum::http::StatusCode::OK,
            Json(ApiResponse::success(resp, "Submission stats computed")),
        )
            .into_response();
    }

    // ---- Query A: all student submissions (for 'ignored' count visibility only)
    let all_student_rows: Vec<SubmissionModel> = match SubmissionEntity::find()
        .filter(assignment_submission::Column::AssignmentId.eq(assignment_id))
        .filter(assignment_submission::Column::UserId.is_in(student_ids.clone()))
        .order_by(assignment_submission::Column::CreatedAt, Order::Desc)
        .all(db)
        .await
    {
        Ok(r) => r,
        Err(_) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<AssignmentStatsResponse>::error(
                    "Database error",
                )),
            )
                .into_response();
        }
    };

    let ignored = all_student_rows.iter().filter(|s| s.ignored).count();

    // ---- Query B: ONLY counted rows → students & NOT practice & NOT ignored
    let rows: Vec<SubmissionModel> = match SubmissionEntity::find()
        .filter(assignment_submission::Column::AssignmentId.eq(assignment_id))
        .filter(assignment_submission::Column::UserId.is_in(student_ids.clone()))
        .filter(assignment_submission::Column::IsPractice.eq(false))
        .filter(assignment_submission::Column::Ignored.eq(false))
        .order_by(assignment_submission::Column::CreatedAt, Order::Desc)
        .all(db)
        .await
    {
        Ok(r) => r,
        Err(_) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<AssignmentStatsResponse>::error(
                    "Database error",
                )),
            )
                .into_response();
        }
    };

    // Everything below uses ONLY the filtered `rows`
    let total = rows.len();

    // late/on_time for counted rows only
    let mut late = 0usize;
    let mut on_time = 0usize;

    use std::collections::{HashMap, HashSet};
    let mut user_marks: HashMap<i64, Vec<(DateTime<Utc>, i64)>> = HashMap::new();

    for s in &rows {
        if is_late(s.created_at, Some(assignment.due_date)) {
            late += 1;
        } else {
            on_time += 1;
        }

        // read mark from centralized path helper
        let report_path = submission_report_path(module_id, assignment_id, s.user_id, s.attempt);

        if let Ok(content) = std::fs::read_to_string(&report_path) {
            if let Ok(json) = serde_json::from_str::<Value>(&content) {
                let earned = json
                    .get("mark")
                    .and_then(|m| m.get("earned"))
                    .and_then(|v| v.as_i64());
                let total = json
                    .get("mark")
                    .and_then(|m| m.get("total"))
                    .and_then(|v| v.as_i64());
                if let Some(p) = to_pct(earned, total) {
                    user_marks
                        .entry(s.user_id)
                        .or_default()
                        .push((s.created_at, p));
                }
            }
        }
    }

    // Effective mark per student by grading policy
    let cfg = assignment
        .config()
        .unwrap_or_else(ExecutionConfig::default_config);
    let grading_policy = cfg.marking.grading_policy;
    let pass_mark_threshold = cfg.marking.pass_mark as i64;

    let mut effective_marks: Vec<i64> = Vec::new();
    for (_uid, marks) in user_marks {
        let chosen = match grading_policy {
            GradingPolicy::Best => marks.iter().map(|(_, p)| *p).max(),
            GradingPolicy::Last => marks.iter().max_by_key(|(ts, _)| *ts).map(|(_, p)| *p),
        };
        if let Some(p) = chosen {
            effective_marks.push(p);
        }
    }

    let graded = effective_marks.len();
    let pending = total.saturating_sub(graded);

    // pass/fail
    let num_passed = effective_marks
        .iter()
        .filter(|&&p| p >= pass_mark_threshold)
        .count();
    let num_failed = graded.saturating_sub(num_passed);
    let num_full_marks = effective_marks.iter().filter(|&&p| p == 100).count();

    let pass_rate = if graded == 0 {
        0.0
    } else {
        (num_passed as f64 / graded as f64) * 100.0
    };

    // totals across counted rows only
    let total_marks: f32 = rows.iter().map(|s| s.total).sum();
    let num_students_submitted = rows.iter().map(|s| s.user_id).collect::<HashSet<_>>().len();

    // aggregates on effective marks
    let avg_mark = mean(&effective_marks);
    let median_mark = median(&mut effective_marks.clone());
    let p75_mark = percentile(&mut effective_marks.clone(), 0.75);
    let stddev_mark = stddev(&effective_marks);
    let best = effective_marks.iter().copied().max().unwrap_or(0);
    let worst = effective_marks.iter().copied().min().unwrap_or(0);

    let resp = AssignmentStatsResponse {
        total,   // counted attempts only (non-practice, non-ignored)
        graded,  // students with an effective mark among counted rows
        pending, // counted attempts minus graded
        pass_rate: (pass_rate * 10.0).round() / 10.0,
        avg_mark: (avg_mark * 10.0).round() / 10.0,
        median: (median_mark * 10.0).round() / 10.0,
        p75: (p75_mark * 10.0).round() / 10.0,
        stddev: (stddev_mark * 10.0).round() / 10.0,
        best,
        worst,
        total_marks,            // sum over counted rows
        num_students_submitted, // students with at least one counted row
        num_passed,
        num_failed,
        num_full_marks,
        late,    // from counted rows only
        on_time, // from counted rows only
        ignored, // visibility from ALL student rows
    };

    (
        axum::http::StatusCode::OK,
        Json(ApiResponse::success(resp, "Submission stats computed")),
    )
        .into_response()
}
