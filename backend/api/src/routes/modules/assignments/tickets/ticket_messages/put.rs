//! Ticket message edit handler.
//!
//! Provides an endpoint to edit an existing message on a ticket.
//!
//! Only the author of the message can update it. The endpoint validates
//! that the `content` field is provided and not empty.

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
    routes::modules::assignments::tickets::ticket_messages::common::MessageResponse,
};

/// Edits an existing ticket message.
///
/// **Endpoint:** `PUT /modules/{module_id}/assignments/{assignment_id}/tickets/{ticket_id}/messages/{message_id}`  
/// **Permissions:** Only the author of the message can edit it.
///
/// ### Path parameters
/// - `module_id`       → ID of the module (unused in handler, for route consistency)
/// - `assignment_id`   → ID of the assignment (unused in handler, for route consistency)
/// - `ticket_id`       → ID of the ticket (unused in handler, for route consistency)
/// - `message_id`      → ID of the message to edit
///
/// ### Request body
/// ```json
/// {
///   "content": "Updated message content"
/// }
/// ```
///
/// ### Responses
/// - `200 OK` → Message updated successfully
/// ```json
/// {
///   "success": true,
///   "data": {
///       "id": 123,
///       "ticket_id": 456,
///       "content": "Updated message content",
///       "created_at": "2025-08-18T10:00:00Z",
///       "updated_at": "2025-08-18T10:15:00Z",
///       "user": null
///   },
///   "message": "Message updated successfully"
/// }
/// ```
/// - `400 Bad Request` → Content is missing or empty
/// ```json
/// {
///   "success": false,
///   "data": null,
///   "message": "Content is required"
/// }
/// ```
/// - `403 Forbidden` → User is not the author of the message
/// ```json
/// {
///   "success": false,
///   "data": null,
///   "message": "Forbidden"
/// }
/// ```
/// - `500 Internal Server Error` → Failed to update the message
/// ```json
/// {
///   "success": false,
///   "data": null,
///   "message": "Failed to update message"
/// }
/// ```
pub async fn edit_ticket_message(
    Path((_, _, _, message_id)): Path<(i64, i64, i64, i64)>,
    State(app_state): State<AppState>,
    Extension(AuthUser(claims)): Extension<AuthUser>,
    Json(req): Json<serde_json::Value>,
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

    let response = MessageResponse {
        id: message.id,
        ticket_id: message.ticket_id,
        content: message.content,
        created_at: message.created_at.to_rfc3339(),
        updated_at: message.updated_at.to_rfc3339(),
        user: None,
    };

    (
        StatusCode::OK,
        Json(ApiResponse::success(
            response,
            "Message updated successfully",
        )),
    )
        .into_response()
}
