use crate::{auth::AuthUser, response::ApiResponse, routes::modules::assignments::tickets::common::is_valid};
use axum::{
    extract::{Path, State}, http::StatusCode, response::{IntoResponse, Json}, Extension
};
use db::models::tickets::Model as TicketModel;
use serde::Serialize;
use util::state::AppState;


#[derive(Serialize)]
struct TicketStatusResponse {
    id: i64,
    status: &'static str,
}

pub async fn open_ticket(
    State(app_state): State<AppState>,
    Path((_, _, ticket_id)): Path<(i64, i64, i64)>,
    Extension(AuthUser(claims)): Extension<AuthUser>,
) -> impl IntoResponse {
    let db = app_state.db();
    let user_id = claims.sub;

    if !is_valid(user_id, ticket_id, db).await {
        return (StatusCode::FORBIDDEN, Json(ApiResponse::<()>::error("Forbidden"))).into_response();
    }

    let data = TicketStatusResponse {
        id: ticket_id,
        status: "open",
    };

    match TicketModel::set_open(db, ticket_id).await {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse::<TicketStatusResponse>::success(data, "Ticket opened successfully")),
        ).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error("Failed to open ticket")),
        ).into_response(),
    }
}

pub async fn close_ticket(
    State(app_state): State<AppState>,
    Path((_, _, ticket_id)): Path<(i64, i64, i64)>,
    Extension(AuthUser(claims)): Extension<AuthUser>,
) -> impl IntoResponse {
    let db = app_state.db();
    let user_id = claims.sub;

    if !is_valid(user_id, ticket_id, db).await {
        return (StatusCode::FORBIDDEN, Json(ApiResponse::<()>::error("Forbidden"))).into_response();
    }

    let data = TicketStatusResponse {
        id: ticket_id,
        status: "closed",
    };

    match TicketModel::set_closed(db, ticket_id).await {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse::<TicketStatusResponse>::success(data, "Ticket closed successfully")),
        ).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error("Failed to close ticket")),
        ).into_response(),
    }
}