use axum::{Router, routing::{post, put, delete, get}};
use util::state::AppState;
pub mod post;
pub mod put;
pub mod common;
pub mod delete;
pub mod get;
use post::create_ticket;
use put::{open_ticket, close_ticket};
use delete::delete_ticket;
use get::{get_ticket, get_tickets};


pub fn ticket_routes(app_state: AppState) -> Router<AppState> {
	Router::new()
	.route("/", post(create_ticket))
	.route("/{ticket_id}/close", put(close_ticket))
	.route("/{ticket_id}/open", put(open_ticket))
	.route("/{ticket_id}", delete(delete_ticket))
	.route("/{ticket_id}", get(get_ticket))
	.route("/",get(get_tickets))
}