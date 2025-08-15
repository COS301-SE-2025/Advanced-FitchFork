use axum::{
    Extension, Json,
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
};
use db::models::tickets::Model as TicketModel;
use serde::Deserialize;

use crate::{auth::AuthUser, response::ApiResponse, routes::modules::assignments::tickets::common::TicketResponse};

#[derive(Debug, Deserialize)]
pub struct TicketRequest {
    pub title: String,
    pub description: String,
}

pub async fn create_ticket(
    Path((_, assignment_id)): Path<(i64, i64)>,
    Extension(AuthUser(claims)): Extension<AuthUser>,
    Json(req): Json<TicketRequest>,
) -> impl IntoResponse {
    let db = db::get_connection().await;
    let user_id = claims.sub;
    match TicketModel::create(db, assignment_id, user_id, &req.title, &req.description).await {
        Ok(ticket) => {
            let response = TicketResponse::from(ticket);
            (
                StatusCode::OK,
                Json(ApiResponse::success(
                    response,
                    "Ticket created successfully",
                )),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<TicketResponse>::error(e.to_string())),
        ),
    }
}
