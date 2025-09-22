use axum::{
    extract::{Path, State, ConnectInfo},
    http::StatusCode,
    Extension, Json,
};
use chrono::Utc;
use serde::Deserialize;
use std::net::SocketAddr;

use crate::{auth::AuthUser, response::ApiResponse};
use util::state::AppState;

use super::common::{AttendanceSessionResponse, CreateSessionReq};
use db::models::attendance_session as Sess;
use sea_orm::PaginatorTrait;

pub async fn create_session(
    State(state): State<AppState>,
    Path(module_id): Path<i64>,
    Extension(AuthUser(claims)): Extension<AuthUser>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(body): Json<CreateSessionReq>,
) -> (StatusCode, Json<ApiResponse<AttendanceSessionResponse>>) {
    let db = state.db();

    let active   = body.active.unwrap_or(false);
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

use db::models::{
    attendance_session::{Entity as SessionEntity, Column as SessionCol},
    attendance_record,
};
use sea_orm::{ColumnTrait, DbErr, EntityTrait, QueryFilter};

#[derive(Deserialize)]
pub struct MarkAttendanceReq {
    pub code: String,
}

use crate::ws::attendance::topics::attendance_session_topic;
use serde_json::json;

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
            Json(ApiResponse::error("Only students are allowed to mark attendance")),
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
            // Broadcast to this session’s topic
            let ws = state.ws_clone();
            let topic = attendance_session_topic(session_id);

            let count = attendance_record::Entity::find()
                .filter(attendance_record::Column::SessionId.eq(session_id))
                .count(db)
                .await
                .unwrap_or(0);

            let event = json!({
                "event": "attendance_marked",
                "payload": {
                    "session_id": session_id,
                    "user_id": claims.sub,
                    "taken_at": now.to_rfc3339(),
                    "count": count
                }
            });
            ws.broadcast(&topic, event.to_string()).await;

            (StatusCode::OK, Json(ApiResponse::success((), "Attendance recorded")))
        }
        Err(DbErr::Custom(m)) => (StatusCode::BAD_REQUEST, Json(ApiResponse::error(m))),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error("Failed to record attendance")),
        ),
    }
}