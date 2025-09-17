//! Ticket creation handler.
//!
//! Provides an endpoint to create a new ticket for an assignment.
//!
//! Only authenticated users can create tickets, and each ticket is linked
//! to the assignment and the user who created it.

use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use db::models::tickets::Model as TicketModel;
use serde::Deserialize;
use util::state::AppState;

use crate::{
    auth::AuthUser, response::ApiResponse,
    routes::modules::assignments::tickets::common::TicketResponse,
};

/// Request payload for creating a ticket.
#[derive(Debug, Deserialize)]
pub struct TicketRequest {
    /// Title of the ticket
    pub title: String,
    /// Detailed description of the issue or request
    pub description: String,
}

/// Creates a new ticket.
///
/// **Endpoint:** `POST /modules/{module_id}/assignments/{assignment_id}/tickets`  
/// **Permissions:** Authenticated users can create tickets for the assignment.
///
/// ### Path parameters
/// - `module_id`       → ID of the module (unused in handler, kept for route consistency)
/// - `assignment_id`   → ID of the assignment for which the ticket is created
///
/// ### Request body
/// ```json
/// {
///   "title": "Issue with submission",
///   "description": "I am unable to submit my assignment due to..."
/// }
/// ```
///
/// ### Responses
/// - `200 OK` → Ticket created successfully
/// ```json
/// {
///   "success": true,
///   "data": {
///       "id": 123,
///       "title": "Issue with submission",
///       "description": "I am unable to submit my assignment due to...",
///       "status": "open",
///       "user_id": 456
///   },
///   "message": "Ticket created successfully"
/// }
/// ```
/// - `500 Internal Server Error` → Failed to create the ticket
/// ```json
/// {
///   "success": false,
///   "data": null,
///   "message": "Failed to create ticket: <error message>"
/// }
/// ```
pub async fn create_ticket(
    State(app_state): State<AppState>,
    Path((_, assignment_id)): Path<(i64, i64)>,
    Extension(AuthUser(claims)): Extension<AuthUser>,
    Json(req): Json<TicketRequest>,
) -> impl IntoResponse {
    let db = app_state.db();
    let user_id = claims.sub;

    match TicketModel::create(db, assignment_id, user_id, &req.title, &req.description).await {
        Ok(ticket) => {
            let response = TicketResponse::from(ticket);
            (
                StatusCode::OK,
                Json(ApiResponse::success(
                    response,
                    "Ticket created successfully",
                )),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<TicketResponse>::error(format!(
                "Failed to create ticket: {}",
                e
            ))),
        ),
    }
}
