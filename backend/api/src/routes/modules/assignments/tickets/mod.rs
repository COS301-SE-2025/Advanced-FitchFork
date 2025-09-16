//! Ticket routes module.
//!
//! Provides the `/tickets` route group with full CRUD and nested message functionality.
//!
//! Routes include:
//! - Create, open, close, delete, and get tickets
//! - List all tickets
//! - Nested routes for ticket messages
//!
//! Access control is enforced via middleware guards for ticket ownership, lecturer, or admin roles.

use axum::{Router, routing::{post, put, delete, get}};
pub mod post;
pub mod put;
pub mod delete;
pub mod get;
pub mod ticket_messages;
pub mod common;
use post::create_ticket;
use put::{open_ticket, close_ticket};
use delete::delete_ticket;
use get::{get_ticket, get_tickets};
use ticket_messages::ticket_message_routes;

/// Builds and returns the `/tickets` route group for a given ticket context.
///
/// Routes:
/// - `POST   /tickets`                  → Create a new ticket
/// - `PUT    /tickets/{ticket_id}/open` → Reopen a closed ticket
/// - `PUT    /tickets/{ticket_id}/close`→ Close an open ticket
/// - `DELETE /tickets/{ticket_id}`      → Delete a ticket
/// - `GET    /tickets/{ticket_id}`      → Get details of a ticket
/// - `GET    /tickets`                  → List all tickets
///
/// Nested routes:
/// - Ticket messages routes → `/{ticket_id}/messages` handled by `ticket_message_routes`
pub fn ticket_routes() -> Router {
    Router::new()
        .route("/", post(create_ticket))
        .route("/{ticket_id}/close", put(close_ticket))
        .route("/{ticket_id}/open", put(open_ticket))
        .route("/{ticket_id}", delete(delete_ticket))
        .route("/{ticket_id}", get(get_ticket))
        .route("/", get(get_tickets))
        .nest("/{ticket_id}/messages", ticket_message_routes())
}
