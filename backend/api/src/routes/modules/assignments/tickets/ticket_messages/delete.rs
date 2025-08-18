//! Ticket message deletion handler.
//!
//! Provides an endpoint to delete an existing message within a ticket.
//!
//! Only the author of the message can delete it. The endpoint validates
//! that the user is the author before performing the deletion.

use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use db::models::ticket_messages::Model as TicketMessageModel;
use util::state::AppState;

use crate::{auth::AuthUser, response::ApiResponse};

/// Deletes a ticket message.
///
/// **Endpoint:** `DELETE /modules/{module_id}/assignments/{assignment_id}/tickets/{ticket_id}/messages/{message_id}`  
/// **Permissions:** Only the message author can delete their message.
///
/// ### Path parameters
/// - `module_id`       → ID of the module (unused in handler, kept for route consistency)
/// - `assignment_id`   → ID of the assignment (unused in handler, kept for route consistency)
/// - `ticket_id`       → ID of the ticket (unused in handler, kept for route consistency)
/// - `message_id`      → ID of the message to be deleted
///
/// ### Responses
/// - `200 OK` → Message deleted successfully
/// ```json
/// {
///   "success": true,
///   "data": { "id": 123 },
///   "message": "Message deleted successfully"
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
/// - `500 Internal Server Error` → Failed to delete the message
/// ```json
/// {
///   "success": false,
///   "data": null,
///   "message": "Failed to delete message"
/// }
/// ```
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
