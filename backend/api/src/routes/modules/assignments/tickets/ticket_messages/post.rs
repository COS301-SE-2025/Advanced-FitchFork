use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use db::models::{
    ticket_messages::Model as TicketMessageModel,
    user::{Column as UserColumn, Entity as UserEntity, Model as UserModel},
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use util::state::AppState;

use crate::{
    auth::AuthUser, response::ApiResponse, routes::modules::assignments::tickets::{common::is_valid, ticket_messages::common::{MessageResponse, UserResponse}},
};

pub async fn create_message(
    Path((module_id, _assignment_id, ticket_id)): Path<(i64, i64, i64)>,
    State(app_state): State<AppState>,
    Extension(AuthUser(claims)): Extension<AuthUser>,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    let db = app_state.db();
    let user_id = claims.sub;

    if !is_valid(user_id, ticket_id, module_id, db).await {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<()>::error("Forbidden")),
        )
            .into_response();
    }

    let content = match req.get("content").and_then(|v| v.as_str()) {
        Some(c) if !c.trim().is_empty() => c.trim().to_string(),
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<()>::error("Content is required")),
            )
                .into_response();
        }
    };

    let user: Option<UserModel> = UserEntity::find()
        .filter(UserColumn::Id.eq(user_id))
        .one(db)
        .await
        .unwrap_or(None);

    let user = match user {
        Some(u) => u,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<()>::error("User not found")),
            )
                .into_response();
        }
    };

    let message = match TicketMessageModel::create(db, ticket_id, user_id, &content).await {
        Ok(msg) => msg,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("Failed to create message")),
            )
                .into_response();
        }
    };

    let response = MessageResponse {
        id: message.id,
        ticket_id: message.ticket_id,
        content: message.content,
        created_at: message.created_at.to_rfc3339(),
        updated_at: message.updated_at.to_rfc3339(),
        user: UserResponse {
            id: user.id,
            username: user.username,
        },
    };

    (
        StatusCode::OK,
        Json(ApiResponse::success(
            response,
            "Ticket created successfully",
        )),
    )
        .into_response()
}
