use axum::{middleware::from_fn_with_state, Router};
use util::state::AppState;

pub mod post;
pub mod get;
pub mod delete;
pub mod put;
pub mod common;
use axum::routing::{post, delete, put, get};
use post::create_announcement;
use delete::delete_announcement;
use put::edit_announcement;
use get::get_announcements;
use crate::auth::guards::require_lecturer_or_assistant_lecturer;
pub fn announcement_routes(app_state: AppState) -> Router<AppState> {
    Router::new()
    .route("/", post(create_announcement).route_layer(from_fn_with_state(app_state.clone(), require_lecturer_or_assistant_lecturer)))
    .route("/{announcement_id}", delete(delete_announcement).route_layer(from_fn_with_state(app_state.clone(), require_lecturer_or_assistant_lecturer)))
    .route("/{announcement_id}", put(edit_announcement).route_layer(from_fn_with_state(app_state.clone(), require_lecturer_or_assistant_lecturer)))
    .route("/", get(get_announcements))
}