use axum::{extract::Path, http::StatusCode, response::IntoResponse, Json};
use db::models::announcements::Model as AnnouncementModel;
use crate::{response::ApiResponse, routes::modules::announcements::common::AnnouncementRequest};

pub async fn edit_announcement(
    Path((_, announcement_id)): Path<(i64, i64)>,
    Json(req): Json<AnnouncementRequest>,
) -> impl IntoResponse {
    let db = db::get_connection().await;
    match AnnouncementModel::update(
        db,
        announcement_id,
        &req.title,
        &req.body,
        req.pinned,
    )
    .await
    {
        Ok(updated_announcement) => {
            (
                StatusCode::OK,
                Json(ApiResponse::success(
                    updated_announcement,
                    "Announcement updated successfully",
                )),
            )
        }
        Err(_) => {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(
                    format!("Failed to update announcement"),
                )),
            )
        }
    }
}