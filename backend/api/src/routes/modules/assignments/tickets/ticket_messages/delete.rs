use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use db::models::ticket_messages::Model as TicketMessageModel;
use util::state::AppState;

use crate::{auth::AuthUser, response::ApiResponse};

pub async fn delete_ticket_message(
    Path((_, _, _, message_id)): Path<(i64, i64, i64, i64)>,
    State(app_state): State<AppState>,
    Extension(AuthUser(claims)): Extension<AuthUser>,
) -> impl IntoResponse {
    let db = app_state.db();
    let user_id = claims.sub;
    let is_author = TicketMessageModel::is_author(message_id, user_id, db).await;

    if !is_author {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<()>::error("Forbidden")),
        )
            .into_response();
    }

    let res = TicketMessageModel::delete(db, message_id).await;

    if res.is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error("Failed to delete message")),
        )
            .into_response();
    }

    (
        StatusCode::OK,
        Json(ApiResponse::success(
            serde_json::json!({ "id": message_id }),
            "Message deleted successfully",
        )),
    )
        .into_response()
}
