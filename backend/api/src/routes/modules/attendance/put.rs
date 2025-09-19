use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use util::state::AppState;

use crate::response::ApiResponse;
use crate::ws::attendance::topics::attendance_session_topic; // NEW
use serde_json::json; // NEW

use super::common::{AttendanceSessionResponse, EditSessionReq};
use db::models::attendance_session::{
    ActiveModel as SessionAM, Column as SessionCol, Entity as SessionEntity,
};

/// PUT /api/modules/{module_id}/attendance/sessions/{session_id}
///
/// Edit an existing attendance session’s settings.
/// Fields `code_length` and `allow_manual_entry` have been removed and can no longer be changed.
pub async fn edit_session(
    State(state): State<AppState>,
    Path((module_id, session_id)): Path<(i64, i64)>,
    Json(body): Json<EditSessionReq>,
) -> (StatusCode, Json<ApiResponse<AttendanceSessionResponse>>) {
    let db = state.db();

    // Load existing session
    let Some(existing) = SessionEntity::find()
        .filter(SessionCol::Id.eq(session_id))
        .filter(SessionCol::ModuleId.eq(module_id))
        .one(db)
        .await
        .unwrap_or(None)
    else {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("Attendance session not found")),
        );
    };

    let mut am: SessionAM = existing.clone().into();

    // Apply allowed updates
    if let Some(t) = body.title {
        am.title = Set(t);
    }
    if let Some(a) = body.active {
        am.active = Set(a);
    }
    if let Some(r) = body.rotation_seconds {
        let r = r.clamp(5, 300);
        am.rotation_seconds = Set(r);
    }
    if let Some(b) = body.restrict_by_ip {
        am.restrict_by_ip = Set(b);
    }
    if let Some(cidr) = body.allowed_ip_cidr {
        am.allowed_ip_cidr = Set(Some(cidr));
    }
    if let Some(ip) = body.created_from_ip {
        am.created_from_ip = Set(Some(ip));
    }
    // Removed: code_length and allow_manual_entry

    // Persist update
    match am.update(db).await {
        Ok(updated) => {
            // --- NEW: broadcast session_updated so other tabs sync immediately
            let ws = state.ws_clone();
            let topic = attendance_session_topic(updated.id);
            let event = json!({
                "event": "session_updated",
                "payload": {
                    "session_id": updated.id,
                    "active": updated.active,
                    "rotation_seconds": updated.rotation_seconds,
                    "title": updated.title,
                    "restrict_by_ip": updated.restrict_by_ip,
                    "allowed_ip_cidr": updated.allowed_ip_cidr,
                    "created_from_ip": updated.created_from_ip,
                }
            });
            ws.broadcast(&topic, event.to_string()).await;

            (
                StatusCode::OK,
                Json(ApiResponse::success(
                    AttendanceSessionResponse::from(updated),
                    "Attendance session updated",
                )),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(format!(
                "Failed to update attendance session: {e}"
            ))),
        ),
    }
}
