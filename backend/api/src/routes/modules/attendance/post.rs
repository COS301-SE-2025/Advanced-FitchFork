use axum::{
    Extension, Json,
    extract::{ConnectInfo, Path, State},
    http::StatusCode,
};
use chrono::Utc;
use serde::Deserialize;
use std::net::SocketAddr;

use crate::{auth::AuthUser, response::ApiResponse};
use util::state::AppState;

use super::common::{AttendanceSessionResponse, CreateSessionReq};
use db::models::attendance_session as Sess;
use sea_orm::PaginatorTrait;

use crate::ws::attendance::emit;
use crate::ws::attendance::payload as ap;
use db::models::{
    attendance_record,
    attendance_session::{Column as SessionCol, Entity as SessionEntity},
    user,
    user::{Column as UserCol, Entity as UserEntity},
};
use sea_orm::{ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, QueryFilter, Set};

pub async fn create_session(
    State(state): State<AppState>,
    Path(module_id): Path<i64>,
    Extension(AuthUser(claims)): Extension<AuthUser>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(body): Json<CreateSessionReq>,
) -> (StatusCode, Json<ApiResponse<AttendanceSessionResponse>>) {
    let db = state.db();

    let active = body.active.unwrap_or(false);
    let rotation = body.rotation_seconds.unwrap_or(30).clamp(5, 300);
    let restrict = body.restrict_by_ip.unwrap_or(false);

    // If requested, pin to the creator's peer IP
    let creator_ip = if body.pin_to_creator_ip.unwrap_or(false) {
        Some(addr.ip().to_string())
    } else {
        None
    };

    // NOTE: Model::create no longer takes code_length/allow_manual_entry
    match Sess::Model::create(
        db,
        module_id,
        claims.sub,
        &body.title,
        active,
        rotation,
        restrict,
        body.allowed_ip_cidr.as_deref(),
        creator_ip.as_deref(),
        None, // generate random secret
    )
    .await
    {
        Ok(row) => (
            StatusCode::CREATED,
            Json(ApiResponse::success(
                AttendanceSessionResponse::from(row),
                "Attendance session created",
            )),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(format!(
                "Failed to create attendance session: {e}"
            ))),
        ),
    }
}

#[derive(Deserialize)]
pub struct MarkAttendanceReq {
    pub code: String,
}

/// POST /api/modules/{module_id}/attendance/sessions/{session_id}/mark
pub async fn mark_attendance(
    State(state): State<AppState>,
    Path((module_id, session_id)): Path<(i64, i64)>,
    Extension(AuthUser(claims)): Extension<AuthUser>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(body): Json<MarkAttendanceReq>,
) -> (StatusCode, Json<ApiResponse<()>>) {
    let db = state.db();
    let now = Utc::now();

    // Only students of this module may mark attendance
    use db::models::user;
    let is_student = user::Model::is_in_role(db, claims.sub, module_id, "Student")
        .await
        .unwrap_or(false);
    if !is_student {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error(
                "Only students are allowed to mark attendance",
            )),
        );
    }

    // Load session (must belong to module)
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

    // -------- NEW: short-circuit if already recorded (before any code/IP checks) --------
    if attendance_record::Entity::find()
        .filter(attendance_record::Column::SessionId.eq(session_id))
        .filter(attendance_record::Column::UserId.eq(claims.sub))
        .one(db)
        .await
        .ok()
        .flatten()
        .is_some()
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error("Attendance already recorded")),
        );
    }
    // ------------------------------------------------------------------------------------

    let ip_txt = Some(addr.ip().to_string());

    // Proceed with normal mark (validates active, IP policy, and rotating code)
    match attendance_record::Model::mark(
        db,
        &sess,
        claims.sub,
        &body.code,
        ip_txt.as_deref(),
        now,
        1, // window tolerance
    )
    .await
    {
        Ok(_rec) => {
            let ws = state.ws_clone();

            let count = attendance_record::Entity::find()
                .filter(attendance_record::Column::SessionId.eq(session_id))
                .count(db)
                .await
                .unwrap_or(0);

            emit::attendance_marked(
                &ws,
                ap::AttendanceMarked {
                    session_id,
                    user_id: claims.sub,
                    taken_at: now.to_rfc3339(),
                    count,
                    method: None,
                },
            )
            .await;

            (
                StatusCode::OK,
                Json(ApiResponse::success((), "Attendance recorded")),
            )
        }
        Err(DbErr::Custom(m)) => (StatusCode::BAD_REQUEST, Json(ApiResponse::error(m))),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error("Failed to record attendance")),
        ),
    }
}

#[derive(Deserialize)]
pub struct ManualMarkByUsernameReq {
    pub username: String,
}

/// POST /api/modules/{module_id}/attendance/sessions/{session_id}/mark/by-username
///
/// Admin-only (Lecturer/AssistantLecturer): mark a student present by username.
/// - No rotating code/IP checks.
/// - Allowed regardless of session.active (lecturer override).
pub async fn mark_attendance_by_username(
    State(state): State<AppState>,
    Path((module_id, session_id)): Path<(i64, i64)>,
    Extension(_auth): Extension<AuthUser>, // route layer already restricts to lecturer/assistant
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(body): Json<ManualMarkByUsernameReq>,
) -> (StatusCode, Json<ApiResponse<()>>) {
    let db = state.db();
    let now = Utc::now();

    // Load session (must belong to module)
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

    // Resolve user by username
    let uname = body.username.trim();
    if uname.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error("Username required")),
        );
    }

    let Some(student) = UserEntity::find()
        .filter(UserCol::Username.eq(uname))
        .one(db)
        .await
        .ok()
        .flatten()
    else {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("User not found")),
        );
    };

    // Ensure this user is a Student of this module
    let is_student = user::Model::is_in_role(db, student.id, module_id, "Student")
        .await
        .unwrap_or(false);
    if !is_student {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error("User is not a student of this module")),
        );
    }

    // Prevent duplicates
    if attendance_record::Entity::find()
        .filter(attendance_record::Column::SessionId.eq(session_id))
        .filter(attendance_record::Column::UserId.eq(student.id))
        .one(db)
        .await
        .ok()
        .flatten()
        .is_some()
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(
                "Attendance already recorded for this user",
            )),
        );
    }

    // Insert manual attendance (lecturer override)
    let token_window = sess.window(now); // keep analytics/export consistent

    let am = attendance_record::ActiveModel {
        session_id: Set(session_id),
        user_id: Set(student.id),
        taken_at: Set(now),
        ip_address: Set(Some(addr.ip().to_string())), // admin IP for audit; set None if undesired
        token_window: Set(token_window),
    };

    match am.insert(db).await {
        Ok(_rec) => {
            let ws = state.ws_clone();

            let count = attendance_record::Entity::find()
                .filter(attendance_record::Column::SessionId.eq(session_id))
                .count(db)
                .await
                .unwrap_or(0);

            emit::attendance_marked(
                &ws,
                ap::AttendanceMarked {
                    session_id,
                    user_id: student.id,
                    taken_at: now.to_rfc3339(),
                    count,
                    method: Some("admin_manual".into()),
                },
            )
            .await;

            (
                StatusCode::OK,
                Json(ApiResponse::success((), "Attendance recorded")),
            )
        }
        Err(DbErr::RecordNotInserted) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error("Failed to insert attendance record")),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error("Failed to record attendance")),
        ),
    }
}
