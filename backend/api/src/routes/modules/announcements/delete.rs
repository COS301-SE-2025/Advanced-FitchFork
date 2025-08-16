use axum::{extract::{Path, State}, http::StatusCode, response::IntoResponse, Json};
use db::models::announcements::Model as AnnouncementModel;
use util::state::AppState;
use crate::response::ApiResponse;

pub async fn delete_announcement(
    State(app_state): State<AppState>,
    Path((_, announcement_id)): Path<(i64, i64)>,
) -> impl IntoResponse {
    let db = app_state.db();
    match AnnouncementModel::delete(db, announcement_id).await {
        Ok(_) => {
            (
                StatusCode::OK,
                Json(ApiResponse::success(
                    (),
                    "Announcement deleted successfully",
                )),
            )
        }
        Err(err) => {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(
                    format!("Failed to delete announcement: {}", err),
                )),
            )
        }
    }
}
