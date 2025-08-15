use axum::{extract::Path, http::StatusCode, response::IntoResponse, Extension, Json};
use crate::{auth::AuthUser, response::ApiResponse, routes::modules::announcements::common::AnnouncementRequest};
use db::models::announcements::Model as AnnouncementModel;

pub async fn create_announcement(
    Path(module_id): Path<i64>,
    Extension(AuthUser(claims)): Extension<AuthUser>,
    Json(req): Json<AnnouncementRequest>,
) -> impl IntoResponse {
    let db = db::get_connection().await;
    let user_id = claims.sub;
    match AnnouncementModel::create(db, module_id, user_id, &req.title, &req.body, req.pinned).await {
        Ok(announcement) => {
            (
                StatusCode::OK,
                Json(ApiResponse::success(
                    announcement,
                    "Announcement created successfully",
                )),
            )
        }
        Err(err) => {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(
                    format!("Failed to create announcement: {}", err),
                )),
            )
        }
    }
}
