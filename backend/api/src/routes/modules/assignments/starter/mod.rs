use axum::{
    Router,
    routing::{get, post},
};
use util::state::AppState;

pub mod common;
pub mod get;
pub mod post;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get::list)) // ← list packs for dropdown
        .route("/", post(post::create)) // existing create endpoint
}
