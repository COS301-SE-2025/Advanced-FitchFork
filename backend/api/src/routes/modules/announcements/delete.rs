use axum::{extract::Path, http::StatusCode, response::IntoResponse, Json};
use db::models::announcements::Model as AnnouncementModel;
use crate::response::ApiResponse;

pub async fn delete_announcement(
    Path((_, announcement_id)): Path<(i64, i64)>,
) -> impl IntoResponse {
    let db = db::get_connection().await;
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
