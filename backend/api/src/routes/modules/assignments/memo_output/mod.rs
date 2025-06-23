pub mod get;
pub mod post;

use axum::Router;

pub fn assignment_routes() -> Router {
    Router::new()
}