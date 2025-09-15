// api/src/routes/modules/attendance/get.rs

//! Attendance module: read-only routes (list sessions, get session,
//! fetch current code, list records, export records as CSV).

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    Extension, Json,
};
use chrono::{SecondsFormat, Utc};
use sea_orm::{ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder};
use util::state::AppState;

use crate::{auth::AuthUser, response::ApiResponse};

use super::common::{AttendanceSessionResponse, ListQuery, ListResponse};
use db::models::attendance_session::{
    Column as SessionCol, Entity as SessionEntity, Model as Session,
};
use db::models::{
    attendance_record::Entity as RecordEntity,
    attendance_record::Column as RecordCol,
    user::{Entity as UserEntity, Column as UserCol},
};

/// GET `/api/modules/{module_id}/attendance/sessions`
///
/// List attendance sessions for a module.
/// 
/// **Auth**: Any user **assigned to the module** (Lecturer/AssistantLecturer/Tutor/Student).
/// Enforced by router (`require_assigned_to_module`).
///
/// **Query**:
/// - `q` *(optional)*: fuzzy match on title
/// - `active` *(optional bool)*
/// - `sort` *(optional)*: `created_at` | `title` | `active` (prefix `-` for desc)
/// - `page` *(default 1)*
/// - `per_page` *(default 20, max 100)*
///
/// **Response**: `ListResponse` with `attended_count` and `student_count` per session.
pub async fn list_sessions(
    State(state): State<AppState>,
    Path(module_id): Path<i64>,
    Query(q): Query<ListQuery>,
) -> (StatusCode, Json<ApiResponse<ListResponse>>) {
    let db = state.db();
    let page = q.page.unwrap_or(1).max(1) as u64;
    let per_page = q.per_page.unwrap_or(20).clamp(1, 100) as u64;

    // Base select
    let mut sel = SessionEntity::find().filter(SessionCol::ModuleId.eq(module_id));
    if let Some(s) = q.q.as_ref().filter(|s| !s.trim().is_empty()) {
        sel = sel.filter(SessionCol::Title.contains(s));
    }
    if let Some(a) = q.active {
        sel = sel.filter(SessionCol::Active.eq(a));
    }
    sel = match q.sort.as_deref() {
        Some(sort) if sort.starts_with('-') => match &sort[1..] {
            "created_at" => sel.order_by_desc(SessionCol::CreatedAt),
            "title" => sel.order_by_desc(SessionCol::Title),
            "active" => sel.order_by_desc(SessionCol::Active),
            _ => sel.order_by_desc(SessionCol::CreatedAt),
        },
        Some("created_at") => sel.order_by_asc(SessionCol::CreatedAt),
        Some("title") => sel.order_by_asc(SessionCol::Title),
        Some("active") => sel.order_by_asc(SessionCol::Active),
        _ => sel.order_by_desc(SessionCol::CreatedAt),
    };

    let paginator = sel.paginate(db, per_page);
    let total = paginator.num_items().await.unwrap_or(0) as i32;
    let rows: Vec<Session> = paginator
        .fetch_page(page.saturating_sub(1))
        .await
        .unwrap_or_default();

    // Counts
    let student_count = Session::student_count_for_module(db, module_id)
        .await
        .unwrap_or(0);

    let session_ids: Vec<i64> = rows.iter().map(|r| r.id).collect();
    let attended_map = Session::attended_student_counts_for(db, module_id, &session_ids)
        .await
        .unwrap_or_default();

    let resp = ListResponse {
        sessions: rows
            .into_iter()
            .map(|s| {
                let attended = *attended_map.get(&s.id).unwrap_or(&0);
                AttendanceSessionResponse::from_with_counts(s, attended, student_count)
            })
            .collect(),
        page: page as i32,
        per_page: per_page as i32,
        total,
    };

    (
        StatusCode::OK,
        Json(ApiResponse::success(
            resp,
            "Attendance sessions retrieved",
        )),
    )
}

/// GET `/api/modules/{module_id}/attendance/sessions/{session_id}`
///
/// Fetch a single attendance session with counts.
/// 
/// **Auth**: Any user assigned to the module.
///
/// **Response**: `AttendanceSessionResponse`.
pub async fn get_session(
    State(state): State<AppState>,
    Path((module_id, session_id)): Path<(i64, i64)>,
) -> (StatusCode, Json<ApiResponse<AttendanceSessionResponse>>) {
    let db = state.db();

    let m = SessionEntity::find()
        .filter(
            Condition::all()
                .add(SessionCol::Id.eq(session_id))
                .add(SessionCol::ModuleId.eq(module_id)),
        )
        .one(db)
        .await;

    match m {
        Ok(Some(row)) => {
            let student_count = Session::student_count_for_module(db, module_id)
                .await
                .unwrap_or(0);

            let attended_count = Session::attended_student_count(db, module_id, session_id)
                .await
                .unwrap_or(0);

            let resp =
                AttendanceSessionResponse::from_with_counts(row, attended_count, student_count);

            (
                StatusCode::OK,
                Json(ApiResponse::success(resp, "Attendance session retrieved")),
            )
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("Attendance session not found")),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(
                "Database error retrieving attendance session",
            )),
        ),
    }
}

/// GET `/api/modules/{module_id}/attendance/sessions/{session_id}/code`
///
/// Get the **current rotating code** for an active session.
/// 
/// **Auth**: **Lecturer or AssistantLecturer**.
///
/// **Notes**:
/// - Returns `400` if the session is not active.
pub async fn get_session_code(
    State(state): State<AppState>,
    Path((module_id, session_id)): Path<(i64, i64)>,
    Extension(_user): Extension<AuthUser>,
) -> (StatusCode, Json<ApiResponse<String>>) {
    let db = state.db();

    let Some(sess) = SessionEntity::find()
        .filter(SessionCol::Id.eq(session_id))
        .filter(SessionCol::ModuleId.eq(module_id))
        .one(db)
        .await
        .ok()
        .flatten()
    else {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("Attendance session not found")),
        );
    };

    if !sess.active {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error("Session is not currently active")),
        );
    }

    let now = Utc::now();
    let code = sess.current_code(now);
    (StatusCode::OK, Json(ApiResponse::success(code, "Current code")))
}

/// A single attendance record (DTO) for API responses.
#[derive(serde::Serialize)]
pub struct AttendanceRecordDto {
    pub session_id: i64,
    pub user_id: i64,
    pub username: Option<String>,
    pub taken_at: String,       // ISO-8601 (UTC)
    pub ip_address: Option<String>,
    pub token_window: i64,
}

/// Query params for listing session records.
#[derive(serde::Deserialize)]
pub struct RecordsListQuery {
    /// Free-text search:
    /// - numeric → matches `user_id`
    /// - text   → matches `username` (contains) OR `ip_address` (contains)
    pub q: Option<String>,
    /// Sort by: `taken_at` | `user_id` (prefix with `-` for desc). Default `-taken_at`.
    pub sort: Option<String>,
    /// 1-based page index (default 1).
    pub page: Option<i32>,
    /// Items per page (default 20, max 200).
    pub per_page: Option<i32>,
}

/// Paged response for records list.
#[derive(serde::Serialize)]
pub struct RecordsListResponse {
    pub records: Vec<AttendanceRecordDto>,
    pub page: i32,
    pub per_page: i32,
    pub total: i32,
}

/// GET `/api/modules/{module_id}/attendance/sessions/{session_id}/records`
///
/// List **attendance records** for a session with **pagination, sorting, and search**.
/// 
/// **Auth**: Any user assigned to the module (router layer).
///
/// **Query**:
/// - `q` *(optional)*:
///   - If numeric, matches `user_id`.
///   - Otherwise, matches `username` (contains) OR `ip_address` (contains).
/// - `sort` *(optional)*: `taken_at` | `user_id` (prefix with `-` for desc). Default `-taken_at`.
/// - `page` *(default 1)*
/// - `per_page` *(default 20, max 200)*
///
/// **Response**: `RecordsListResponse`
pub async fn list_session_records(
    State(state): State<AppState>,
    Path((_, session_id)): Path<(i64, i64)>,
    Query(q): Query<RecordsListQuery>,
) -> (StatusCode, Json<ApiResponse<RecordsListResponse>>) {
    let db = state.db();

    let page = q.page.unwrap_or(1).max(1) as u64;
    let per_page = q.per_page.unwrap_or(20).clamp(1, 200) as u64;

    // ----- Base selector
    let mut sel = RecordEntity::find().filter(RecordCol::SessionId.eq(session_id));

    // ----- Search (q)
    if let Some(ref raw) = q.q.as_ref().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()) {
        let mut cond = Condition::any();
        // ip contains
        cond = cond.add(RecordCol::IpAddress.contains(raw));

        // numeric → user_id
        if let Ok(uid) = raw.parse::<i64>() {
            cond = cond.add(RecordCol::UserId.eq(uid));
        } else {
            // username contains → resolve user_ids and filter
            let name_ids: Vec<i64> = UserEntity::find()
                .filter(UserCol::Username.contains(raw))
                .all(db)
                .await
                .unwrap_or_default()
                .into_iter()
                .map(|u| u.id)
                .collect();
            if !name_ids.is_empty() {
                cond = cond.add(RecordCol::UserId.is_in(name_ids));
            }
        }

        sel = sel.filter(cond);
    }

    // ----- Sorting
    sel = match q.sort.as_deref() {
        Some(sort) if sort.starts_with('-') => match &sort[1..] {
            "taken_at" => sel.order_by_desc(RecordCol::TakenAt),
            "user_id"  => sel.order_by_desc(RecordCol::UserId),
            _          => sel.order_by_desc(RecordCol::TakenAt),
        },
        Some("taken_at") => sel.order_by_asc(RecordCol::TakenAt),
        Some("user_id")  => sel.order_by_asc(RecordCol::UserId),
        _                => sel.order_by_desc(RecordCol::TakenAt), // default newest first
    };

    // ----- Pagination
    let paginator = sel.paginate(db, per_page);
    let total = paginator.num_items().await.unwrap_or(0) as i32;
    let rows = paginator
        .fetch_page(page.saturating_sub(1))
        .await
        .unwrap_or_default();

    // Resolve usernames only for the page results
    let user_ids: Vec<i64> = rows.iter().map(|r| r.user_id).collect();
    let mut uname_map = std::collections::HashMap::<i64, String>::new();
    if !user_ids.is_empty() {
        let users = UserEntity::find()
            .filter(UserCol::Id.is_in(user_ids.clone()))
            .all(db)
            .await
            .unwrap_or_default();
        for u in users {
            uname_map.insert(u.id, u.username);
        }
    }

    let records = rows
        .into_iter()
        .map(|r| AttendanceRecordDto {
            session_id,
            user_id: r.user_id,
            username: uname_map.get(&r.user_id).cloned(),
            taken_at: r
                .taken_at
                .with_timezone(&Utc)
                .to_rfc3339_opts(SecondsFormat::Secs, true),
            ip_address: r.ip_address,
            token_window: r.token_window,
        })
        .collect::<Vec<_>>();

    let resp = RecordsListResponse {
        records,
        page: page as i32,
        per_page: per_page as i32,
        total,
    };

    (
        StatusCode::OK,
        Json(ApiResponse::success(
            resp,
            "Attendance records retrieved",
        )),
    )
}

/// GET `/api/modules/{module_id}/attendance/sessions/{session_id}/records.csv`
///
/// Export all **attendance records** for a session as a CSV file.
/// 
/// **Auth**: Any user assigned to the module.
///
/// **Response**: `text/csv` attachment with columns:
/// `session_id,user_id,username,taken_at,ip_address,token_window`
pub async fn export_session_records_csv(
    State(state): State<AppState>,
    Path((_, session_id)): Path<(i64, i64)>,
) -> (StatusCode, (HeaderMap, String)) {
    let db = state.db();

    let records = match RecordEntity::find()
        .filter(RecordCol::SessionId.eq(session_id))
        .all(db)
        .await
    {
        Ok(v) => v,
        Err(_) => {
            let mut headers = HeaderMap::new();
            headers.insert(
                axum::http::header::CONTENT_TYPE,
                HeaderValue::from_static("text/plain; charset=utf-8"),
            );
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                (headers, "error".to_string()),
            );
        }
    };

    let user_ids: Vec<i64> = records.iter().map(|r| r.user_id).collect();
    let users = UserEntity::find()
        .filter(UserCol::Id.is_in(user_ids.clone()))
        .all(db)
        .await
        .unwrap_or_default();

    let mut uname_map = std::collections::HashMap::<i64, String>::new();
    for u in users { uname_map.insert(u.id, u.username); }

    // CSV header
    let mut csv =
        String::from("session_id,user_id,username,taken_at,ip_address,token_window\n");

    fn esc(s: &str) -> String {
        if s.contains(',') || s.contains('"') || s.contains('\n') {
            format!("\"{}\"", s.replace('"', "\"\""))
        } else {
            s.to_string()
        }
    }

    for r in records {
        let uname = uname_map.get(&r.user_id).map(|s| s.as_str()).unwrap_or("");
        let taken_at_iso = r
            .taken_at
            .with_timezone(&Utc)
            .to_rfc3339_opts(SecondsFormat::Secs, true);

        let row = format!(
            "{},{},{},{},{},{}\n",
            r.session_id,
            r.user_id,
            esc(uname),
            esc(&taken_at_iso),
            esc(&r.ip_address.clone().unwrap_or_default()),
            r.token_window
        );
        csv.push_str(&row);
    }

    let filename = format!("attendance_session_{}.csv", session_id);

    let mut headers = HeaderMap::new();
    headers.insert(
        axum::http::header::CONTENT_TYPE,
        HeaderValue::from_static("text/csv; charset=utf-8"),
    );
    headers.insert(
        axum::http::header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!("attachment; filename=\"{}\"", filename))
            .unwrap_or(HeaderValue::from_static("attachment")),
    );

    (StatusCode::OK, (headers, csv))
}
