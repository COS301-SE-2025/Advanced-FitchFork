//! Ticket status handlers.
//!
//! Provides endpoints to open or close a ticket in a module.
//!
//! Access control is enforced via the `is_valid` function, which ensures the
//! user has permission to modify the ticket. Only the ticket owner or
//! authorized users can perform these actions.

use crate::{
    auth::AuthUser,
    response::ApiResponse,
    routes::modules::assignments::tickets::common::is_valid,
};
use axum::{
    Extension,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use db::models::tickets::Model as TicketModel;
use serde::Serialize;
use util::state::AppState;

/// Response payload for ticket status updates.
#[derive(Serialize)]
struct TicketStatusResponse {
    /// Ticket ID
    id: i64,
    /// Current status of the ticket ("open" or "closed")
    status: &'static str,
}

/// Opens a ticket.
///
/// **Endpoint:** `PUT /modules/{module_id}/assignments/{assignment_id}/tickets/{ticket_id}/open`  
/// **Permissions:** Must be the ticket owner or an authorized user.
///
/// ### Path parameters
/// - `module_id`       → ID of the module containing the ticket
/// - `assignment_id`   → ID of the assignment (unused in this handler, kept for path consistency)
/// - `ticket_id`       → ID of the ticket to open
///
/// ### Responses
/// - `200 OK` → Ticket opened successfully
/// ```json
/// {
///   "success": true,
///   "data": { "id": 123, "status": "open" },
///   "message": "Ticket opened successfully"
/// }
/// ```
/// - `403 Forbidden` → User is not authorized to open this ticket
/// ```json
/// {
///   "success": false,
///   "data": null,
///   "message": "User is not authorized to open this ticket"
/// }
/// ```
/// - `500 Internal Server Error` → Failed to update ticket status
/// ```json
/// {
///   "success": false,
///   "data": null,
///   "message": "Failed to open ticket"
/// }
/// ```

pub async fn open_ticket(
    State(app_state): State<AppState>,
    Path((module_id, _, ticket_id)): Path<(i64, i64, i64)>,
    Extension(AuthUser(claims)): Extension<AuthUser>,
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

    let data = TicketStatusResponse {
        id: ticket_id,
        status: "open",
    };

    match TicketModel::set_open(db, ticket_id).await {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse::<TicketStatusResponse>::success(
                data,
                "Ticket opened successfully",
            )),
        )
            .into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error("Failed to open ticket")),
        )
            .into_response(),
    }
}

/// Closes a ticket.
///
/// **Endpoint:** `PUT /modules/{module_id}/assignments/{assignment_id}/tickets/{ticket_id}/close`  
/// **Permissions:** Must be the ticket owner or an authorized user.
///
/// ### Path parameters
/// - `module_id`       → ID of the module containing the ticket
/// - `assignment_id`   → ID of the assignment (unused in this handler, kept for path consistency)
/// - `ticket_id`       → ID of the ticket to close
///
/// ### Responses
/// - `200 OK` → Ticket closed successfully
/// ```json
/// {
///   "success": true,
///   "data": { "id": 123, "status": "closed" },
///   "message": "Ticket closed successfully"
/// }
/// ```
/// - `403 Forbidden` → User is not authorized to close this ticket
/// ```json
/// {
///   "success": false,
///   "data": null,
///   "message": "User is not authorized to close this ticket"
/// }
/// ```
/// - `500 Internal Server Error` → Failed to update ticket status
/// ```json
/// {
///   "success": false,
///   "data": null,
///   "message": "Failed to close ticket"
/// }
/// ```

pub async fn close_ticket(
    State(app_state): State<AppState>,
    Path((module_id, _, ticket_id)): Path<(i64, i64, i64)>,
    Extension(AuthUser(claims)): Extension<AuthUser>,
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

    let data = TicketStatusResponse {
        id: ticket_id,
        status: "closed",
    };

    match TicketModel::set_closed(db, ticket_id).await {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse::<TicketStatusResponse>::success(
                data,
                "Ticket closed successfully",
            )),
        )
            .into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error("Failed to close ticket")),
        )
            .into_response(),
    }
}
