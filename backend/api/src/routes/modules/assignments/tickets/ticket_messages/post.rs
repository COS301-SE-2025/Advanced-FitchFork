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
    auth::AuthUser,
    response::ApiResponse,
    routes::modules::assignments::tickets::{
        common::is_valid,
        ticket_messages::common::{MessageResponse, UserResponse},
    }, ws::tickets::topics::ticket_chat_topic,
};

/// POST /api/modules/{module_id}/assignments/{assignment_id}/tickets/{ticket_id}/messages
///
/// Create a **new message** in a ticket and broadcast a WebSocket event on:
/// `ws/tickets/{ticket_id}`
///
/// ### Path Parameters
/// - `module_id` (i64)
/// - `assignment_id` (i64)
/// - `ticket_id` (i64)
///
/// ### Request Body (JSON)
/// - `content` (string, required)
///
/// ### Responses
/// - `200 OK` — returns the created message; also emits a WS event:
///   ```json
///   {
///     "event": "message_created",
///     "payload": {
///       "id": 123,
///       "ticket_id": 99,
///       "content": "Hey, I'm blocked on step 3.",
///       "created_at": "2025-02-18T09:12:33Z",
///       "updated_at": "2025-02-18T09:12:33Z",
///       "user": { "id": 42, "username": "alice" }
///     }
///   }
///   ```
/// - `400 Bad Request` — `{ "success": false, "message": "Content is required" }`
/// - `403 Forbidden` — `{ "success": false, "message": "Forbidden" }`
/// - `404 Not Found` — `{ "success": false, "message": "User not found" }`
/// - `500 Internal Server Error`
///
/// ### Example Request
/// ```http
/// POST /api/modules/42/assignments/7/tickets/99/messages
/// Authorization: Bearer <token>
/// Content-Type: application/json
///
/// { "content": "Can someone review my latest attempt?" }
/// ```
pub async fn create_message(
    Path((module_id, _, ticket_id)): Path<(i64, i64, i64)>,
    State(app_state): State<AppState>,
    Extension(AuthUser(claims)): Extension<AuthUser>,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    let db = app_state.db();
    let user_id = claims.sub;

    if !is_valid(user_id, ticket_id, module_id, claims.admin, db).await {
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
        user: Some(UserResponse {
            id: user.id,
            username: user.username,
        }),
    };

    // ---- WebSocket broadcast: notify subscribers on this ticket's topic ----
    // Topic: ws/tickets/{ticket_id}
    let topic = ticket_chat_topic(ticket_id);
    let ws = app_state.ws_clone();
    let event_json = serde_json::json!({
        "event": "message_created",
        "payload": &response
    });
    ws.broadcast(&topic, event_json.to_string()).await;

    (
        StatusCode::OK,
        Json(ApiResponse::success(
            response,
            "Message created successfully",
        )),
    )
        .into_response()
}
