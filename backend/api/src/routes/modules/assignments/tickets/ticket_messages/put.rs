use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use db::models::ticket_messages::Model as TicketMessageModel;
use util::state::AppState;

use crate::{
    auth::AuthUser,
    response::ApiResponse,
    routes::modules::assignments::tickets::ticket_messages::common::MessageResponse, ws::tickets::topics::ticket_chat_topic,
};

/// PUT /api/modules/{module_id}/assignments/{assignment_id}/tickets/{ticket_id}/messages/{message_id}
///
/// Update (edit) a **ticket message**. Only the **author** of the message may edit it.
///
/// ### Path Parameters
/// - `module_id` (i64): Module ID (for scoping/authorization)
/// - `assignment_id` (i64): Assignment ID (for scoping/authorization)
/// - `ticket_id` (i64): Ticket ID (for scoping/authorization)
/// - `message_id` (i64): The message to update
///
/// ### Authorization
/// - Requires a valid bearer token
/// - Caller must be the **author** of the message; otherwise `403 Forbidden` is returned
///
/// ### Request Body
/// ```json
/// { "content": "Updated message text" }
/// ```
/// - `content` (string, required): New message content (non-empty after trimming)
///
/// ### WebSocket Broadcast
/// On success, the server broadcasts to:
/// `ws/tickets/{ticket_id}`
///
/// Event payload:
/// ```json
/// {
///   "event": "message_updated",
///   "payload": {
///     "id": 123,
///     "ticket_id": 99,
///     "content": "Updated message text",
///     "created_at": "2025-02-01T12:00:00Z",
///     "updated_at": "2025-02-01T12:05:00Z",
///     "user": null
///   }
/// }
/// ```
///
/// ### Responses
///
/// - `200 OK` — Message updated
/// ```json
/// {
///   "success": true,
///   "message": "Message updated successfully",
///   "data": {
///     "id": 123,
///     "ticket_id": 99,
///     "content": "Updated message text",
///     "created_at": "2025-02-01T12:00:00Z",
///     "updated_at": "2025-02-01T12:05:00Z",
///     "user": null
///   }
/// }
/// ```
///
/// - `400 Bad Request` — Missing/empty `content`
/// ```json
/// { "success": false, "message": "Content is required" }
/// ```
///
/// - `403 Forbidden` — Caller is not the author
/// ```json
/// { "success": false, "message": "Forbidden" }
/// ```
///
/// - `500 Internal Server Error` — Database error while updating
/// ```json
/// { "success": false, "message": "Failed to update message" }
/// ```
///
/// ### Example Request
/// ```http
/// PUT /api/modules/42/assignments/7/tickets/99/messages/123
/// Authorization: Bearer <token>
/// Content-Type: application/json
///
/// { "content": "Updated message text" }
/// ```
pub async fn edit_ticket_message(
    // NOTE: we now extract module_id, assignment_id, and ticket_id so we can build the WS topic
    Path((_, _, ticket_id, message_id)): Path<(i64, i64, i64, i64)>,
    State(app_state): State<AppState>,
    Extension(AuthUser(claims)): Extension<AuthUser>,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    let db = app_state.db();
    let user_id = claims.sub;

    // Only the author can edit
    let is_author = TicketMessageModel::is_author(message_id, user_id, db).await;
    if !is_author {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<()>::error("Forbidden")),
        )
            .into_response();
    }

    // Validate content
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

    // Update in DB
    let message = match TicketMessageModel::update(db, message_id, &content).await {
        Ok(msg) => msg,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("Failed to update message")),
            )
                .into_response();
        }
    };

    // Prepare REST response shape
    let response = MessageResponse {
        id: message.id,
        ticket_id: message.ticket_id,
        content: message.content.clone(),
        created_at: message.created_at.to_rfc3339(),
        updated_at: message.updated_at.to_rfc3339(),
        user: None, // optional; clients preserve existing sender
    };

    // --- WebSocket broadcast: message_updated ---
    let topic = ticket_chat_topic(ticket_id);
    let payload = serde_json::json!({
        "event": "message_updated",
        "payload": {
            "id": message.id,
            "ticket_id": message.ticket_id,
            "content": message.content,
            "created_at": message.created_at.to_rfc3339(),
            "updated_at": message.updated_at.to_rfc3339(),
            "user": null
        }
    });
    app_state.ws_clone().broadcast(&topic, payload.to_string()).await;

    (
        StatusCode::OK,
        Json(ApiResponse::success(
            response,
            "Message updated successfully",
        )),
    )
        .into_response()
}
