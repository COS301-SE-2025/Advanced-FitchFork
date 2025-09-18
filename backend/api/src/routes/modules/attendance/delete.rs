use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use util::state::AppState;

use crate::response::ApiResponse;
use crate::ws::attendance::topics::attendance_session_topic;
use serde_json::json;

use db::models::attendance_session::{Column as SessionCol, Entity as SessionEntity};

pub async fn delete_session(
    State(state): State<AppState>,
    Path((module_id, session_id)): Path<(i64, i64)>,
) -> (StatusCode, Json<ApiResponse<()>>) {
    let db = state.db();

    let res = SessionEntity::delete_many()
        .filter(SessionCol::Id.eq(session_id))
        .filter(SessionCol::ModuleId.eq(module_id))
        .exec(db)
        .await;

    match res {
        Ok(dr) if dr.rows_affected > 0 => {
            let ws = state.ws_clone();
            let topic = attendance_session_topic(session_id);
            let event = json!({
                "event": "session_deleted",
                "payload": { "session_id": session_id }
            });
            ws.broadcast(&topic, event.to_string()).await;

            (
                StatusCode::OK,
                Json(ApiResponse::success((), "Attendance session deleted")),
            )
        }
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("Attendance session not found")),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error("Failed to delete attendance session")),
        ),
    }
}
