use crate::{auth::AuthUser, response::ApiResponse};
use axum::{
    extract::{Path, State}, http::StatusCode, response::{IntoResponse, Json}, Extension
};
use db::models::tickets::Model as TicketModel;
use db::models::user_module_role::Role;
use db::models::{user, user_module_role};
use sea_orm::entity::prelude::*;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, JoinType, QueryFilter, QuerySelect};
use serde::Serialize;
use util::state::AppState;

async fn is_valid(module_id: i64, user_id: i64, ticket_id: i64, db: &DatabaseConnection) -> bool {
    let is_student = user_module_role::Entity::find()
        .filter(user_module_role::Column::UserId.eq(user_id))
        .filter(user_module_role::Column::ModuleId.eq(module_id))
        .filter(user_module_role::Column::Role.eq(Role::Student))
        .join(JoinType::InnerJoin, user_module_role::Relation::User.def())
        .filter(user::Column::Admin.eq(false))
        .one(db)
        .await
        .map(|opt| opt.is_some())
        .unwrap_or(false);

    let is_author = TicketModel::is_author(ticket_id, user_id, db).await;

    !is_student && is_author
}

#[derive(Serialize)]
struct TicketStatusResponse {
    id: i64,
    status: &'static str,
}

pub async fn open_ticket(
    State(app_state): State<AppState>,
    Path((module_id, _, ticket_id)): Path<(i64, i64, i64)>,
    Extension(AuthUser(claims)): Extension<AuthUser>,
) -> impl IntoResponse {
    let db = app_state.db();
    let user_id = claims.sub;

    if !is_valid(module_id, user_id, ticket_id, db).await {
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
    Path((module_id, _, ticket_id)): Path<(i64, i64, i64)>,
    Extension(AuthUser(claims)): Extension<AuthUser>,
) -> impl IntoResponse {
    let db = app_state.db();
    let user_id = claims.sub;

    if !is_valid(module_id, user_id, ticket_id, db).await {
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