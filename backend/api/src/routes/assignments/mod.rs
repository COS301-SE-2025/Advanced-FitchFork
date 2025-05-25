use axum::{
    Router, routing::{post}
};
use post::create;
use crate::auth::guards::require_authenticated;
pub mod post;


pub fn assignment_routes() -> Router {
    Router::new().route("/create", post(create)).route_layer(axum::middleware::from_fn(require_authenticated))
}