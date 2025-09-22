//! Ticket message routes.
//!
//! Provides all endpoints for managing ticket messages within a module assignment.
//!
//! Endpoints include creating, editing, deleting, and retrieving messages.  
//! Access control is enforced using `is_valid` and author checks for editing/deleting messages.

use axum::{
    Router,
    routing::{delete, get, post, put},
};

pub mod common;
pub mod delete;
pub mod get;
pub mod post;
pub mod put;

use delete::delete_ticket_message;
use get::get_ticket_messages;
use post::create_message;
use put::edit_ticket_message;

/// Returns a `Router` configured with ticket message endpoints.
///
/// ### Routes
/// - `POST /` → Create a new ticket message (`create_message`)
/// - `GET /` → Retrieve all messages for a ticket (`get_ticket_messages`)
/// - `PUT /{message_id}` → Edit an existing ticket message (`edit_ticket_message`)
/// - `DELETE /{message_id}` → Delete a ticket message (`delete_ticket_message`)
///
/// ### Note
/// - Routes expect the `AppState` extractor to provide the database connection.
/// - Authorization is enforced per handler.
pub fn ticket_message_routes() -> Router {
    Router::new()
        .route("/", post(create_message))
        .route("/", get(get_ticket_messages))
        .route("/{message_id}", put(edit_ticket_message))
        .route("/{message_id}", delete(delete_ticket_message))
}
