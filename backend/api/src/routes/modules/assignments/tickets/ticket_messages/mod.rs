use axum::{Router, routing::{post, put, delete}};
use util::state::AppState;
pub mod post;
pub mod put;
pub mod delete;
pub mod common;
use post::create_message;
use put::edit_ticket_message;
use delete::delete_ticket_message;
pub fn ticket_message_routes(_app_state: AppState) -> Router<AppState> {
	Router::new()
	.route("/", post(create_message))
	.route("/{message_id}", put(edit_ticket_message))
	.route("/{message_id}", delete(delete_ticket_message))
}