//! Ticket message deletion handler.
//!
//! Provides an endpoint to delete an existing message within a ticket.
//!
//! Only the author of the message can delete it. The endpoint validates
//! that the user is the author before performing the deletion.

use axum::{
    Extension, Json,
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
};
use crate::{auth::AuthUser, response::ApiResponse, ws::tickets::topics::ticket_chat_topic};
use util::state::AppState;
use services::service::Service;
use services::ticket_message::TicketMessageService;

/// DELETE /api/modules/{module_id}/assignments/{assignment_id}/tickets/{ticket_id}/messages/{message_id}
///
/// Delete a **ticket message**. Only the **author** of the message may delete it.
///
/// ### Path Parameters
/// - `module_id` (i64): Module ID (present in the route for authorization scope)
/// - `assignment_id` (i64): Assignment ID (present in the route for authorization scope)
/// - `ticket_id` (i64): Ticket ID (present in the route for authorization scope)
/// - `message_id` (i64): The ID of the message to delete
///
/// ### Authorization
/// - Requires a valid bearer token
/// - Caller must be the **author** of the message; otherwise `403 Forbidden` is returned
///
/// ### WebSocket Broadcast
/// - On success, broadcasts:
/// ```json
/// { "event": "message_deleted", "payload": { "id": <message_id> } }
/// ```
/// to topic:
/// `ws/tickets/{ticket_id}`
///
/// ### Responses
///
/// - `200 OK` — Message deleted
/// ```json
/// {
///   "success": true,
///   "message": "Message deleted successfully",
///   "data": { "id": 123 }
/// }
/// ```
///
/// - `403 Forbidden` — Caller is not the author
/// ```json
/// { "success": false, "message": "Forbidden" }
/// ```
///
/// - `500 Internal Server Error` — Database error while deleting
/// ```json
/// { "success": false, "message": "Failed to delete message" }
/// ```
pub async fn delete_ticket_message(
    Path((_, _, ticket_id, message_id)): Path<(i64, i64, i64, i64)>,
    Extension(AuthUser(claims)): Extension<AuthUser>,
) -> impl IntoResponse {
    let user_id = claims.sub;

    // Author check
    let is_author = TicketMessageService::is_author(message_id, user_id).await;
    if !is_author {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<()>::error("Forbidden")),
        )
            .into_response();
    }

    // Delete
    if let Err(_) = TicketMessageService::delete_by_id(message_id).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error("Failed to delete message")),
        )
            .into_response();
    }

    // Broadcast deletion to the per-ticket chat topic
    let topic = ticket_chat_topic(ticket_id);
    let event = serde_json::json!({
        "event": "message_deleted",
        "payload": { "id": message_id }
    });
    AppState::get().ws().broadcast(&topic, event.to_string()).await;

    // HTTP response
    (
        StatusCode::OK,
        Json(ApiResponse::success(
            serde_json::json!({ "id": message_id }),
            "Message deleted successfully",
        )),
    )
        .into_response()
}
