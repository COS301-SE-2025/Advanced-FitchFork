//! Ticket deletion handler.
//!
//! Provides an endpoint to delete an existing ticket for an assignment.
//!
//! Only the user who owns the ticket can delete it. The endpoint validates
//! that the user has permission before performing the deletion.

use crate::routes::modules::assignments::tickets::common::is_valid;
use crate::{auth::AuthUser, response::ApiResponse};
use axum::{
    Extension,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use services::service::Service;
use services::ticket::TicketService;

/// Deletes an existing ticket.
///
/// **Endpoint:** `DELETE /modules/{module_id}/assignments/{assignment_id}/tickets/{ticket_id}`  
/// **Permissions:** Only the ticket owner can delete their ticket.
///
/// ### Path parameters
/// - `module_id`       → ID of the module (used for permission check)
/// - `assignment_id`   → ID of the assignment (unused in handler, kept for route consistency)
/// - `ticket_id`       → ID of the ticket to be deleted
///
/// ### Responses
/// - `200 OK` → Ticket deleted successfully
/// ```json
/// {
///   "success": true,
///   "data": {},
///   "message": "Ticket deleted successfully"
/// }
/// ```
/// - `403 Forbidden` → User does not have permission to delete this ticket
/// ```json
/// {
///   "success": false,
///   "data": null,
///   "message": "Forbidden"
/// }
/// ```
/// - `500 Internal Server Error` → Failed to delete the ticket
/// ```json
/// {
///   "success": false,
///   "data": null,
///   "message": "Failed to delete ticket"
/// }
/// ```
pub async fn delete_ticket(
    Path((module_id, _, ticket_id)): Path<(i64, i64, i64)>,
    Extension(AuthUser(claims)): Extension<AuthUser>,
) -> impl IntoResponse {
    let user_id = claims.sub;

    if !is_valid(user_id, ticket_id, module_id, claims.admin, db).await {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<()>::error("Forbidden")),
        )
            .into_response();
    }

    match TicketService::delete_by_id(ticket_id).await {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse::<()>::success(
                (),
                "Ticket deleted successfully",
            )),
        )
            .into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error("Failed to delete ticket")),
        )
            .into_response(),
    }
}
