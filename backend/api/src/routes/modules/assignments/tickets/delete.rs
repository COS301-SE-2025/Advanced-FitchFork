use crate::{auth::AuthUser, response::ApiResponse};
use axum::{
    extract::Path, http::StatusCode, response::{IntoResponse, Json}, Extension
};
use db::models::tickets::Model as TicketModel;
use crate::routes::modules::assignments::tickets::common::is_valid;

pub async fn delete_ticket(
    Path((_, _, ticket_id)): Path<(i64, i64, i64)>,
    Extension(AuthUser(claims)): Extension<AuthUser>,
) -> impl IntoResponse {
    let db = db::get_connection().await;
    let user_id = claims.sub;

    if !is_valid(user_id, ticket_id, db).await {
        return (StatusCode::FORBIDDEN, Json(ApiResponse::<()>::error("Forbidden"))).into_response();
    }

    match TicketModel::delete(db, ticket_id).await {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse::<()>::success((), "Ticket deleted successfully")),
        ).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error("Failed to delete ticket")),
        ).into_response(),
    }
}