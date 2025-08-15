use axum::{middleware::from_fn, Router};
use axum::routing::{post, delete, put, get};
use crate::auth::guards::require_lecturer_or_assistant_lecturer;
use post::create_announcement;
use delete::delete_announcement;
use put::edit_announcement;
use get::get_announcements;

pub mod post;
pub mod get;
pub mod delete;
pub mod put;
pub mod common;

pub fn announcement_routes() -> Router {
    Router::new()
    .route("/", post(create_announcement).route_layer(from_fn(require_lecturer_or_assistant_lecturer)))
    .route("/{announcement_id}", delete(delete_announcement).route_layer(from_fn(require_lecturer_or_assistant_lecturer)))
    .route("/{announcement_id}", put(edit_announcement).route_layer(from_fn(require_lecturer_or_assistant_lecturer)))
    .route("/", get(get_announcements))
}