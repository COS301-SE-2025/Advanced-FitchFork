use axum::{Router, routing::{post, put}};
use util::state::AppState;
pub mod post;
pub mod put;

use post::create_ticket;
use put::{open_ticket, close_ticket};
pub fn ticket_routes(app_state: AppState) -> Router<AppState> {
	Router::new().route("/", post(create_ticket))
	.route("/{ticket_id}/close", put(close_ticket))
	.route("/{ticket_id}/open", put(open_ticket))
}