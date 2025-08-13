use axum::{Router, routing::post};
use util::state::AppState;
pub mod post;
pub mod common;
use post::create_message;
pub fn ticket_message_routes(_app_state: AppState) -> Router<AppState> {
	Router::new()
	.route("/", post(create_message))
}