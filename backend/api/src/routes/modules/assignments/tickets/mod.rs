use axum::{Router, routing::{post, put, delete, get}};
use util::state::AppState;
pub mod post;
pub mod put;
pub mod common;
pub mod delete;
pub mod get;
pub mod ticket_messages;
use post::create_ticket;
use put::{open_ticket, close_ticket};
use delete::delete_ticket;
use get::{get_ticket, get_tickets};
use ticket_messages::ticket_message_routes;

pub fn ticket_routes(_app_state: AppState) -> Router<AppState> {
	Router::new()
	.route("/", post(create_ticket))
	.route("/{ticket_id}/close", put(close_ticket))
	.route("/{ticket_id}/open", put(open_ticket))
	.route("/{ticket_id}", delete(delete_ticket))
	.route("/{ticket_id}", get(get_ticket))
	.route("/",get(get_tickets))
	.nest("/{ticket_id}/messages", ticket_message_routes(_app_state.clone()))
}