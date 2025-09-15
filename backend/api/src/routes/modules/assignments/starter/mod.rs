use axum::{routing::{get, post}, Router};
use util::state::AppState;

pub mod get;
pub mod post;
pub mod common;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get::list))   // ← list packs for dropdown
        .route("/", post(post::create))  // existing create endpoint
}
