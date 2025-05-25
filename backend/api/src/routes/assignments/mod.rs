use axum::{
    Router, routing::{post}
};
use axum::routing::delete;
use post::{create, upload_files};
use crate::auth::guards::require_authenticated;
pub mod post;


pub fn assignment_routes() -> Router {
    Router::new()
        .route("/create", post(create))
        .route("/:module_id/files", post(upload_files))
        .route_layer(axum::middleware::from_fn(require_authenticated))
}