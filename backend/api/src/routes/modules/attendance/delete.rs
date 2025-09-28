use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use util::state::AppState;

use crate::response::ApiResponse;
use crate::ws::attendance::emit;
use crate::ws::attendance::payload as ap;

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
            emit::session_deleted(&ws, ap::SessionDeleted { session_id }).await;

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
