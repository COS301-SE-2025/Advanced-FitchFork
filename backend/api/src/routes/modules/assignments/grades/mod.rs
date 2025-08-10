use axum::{middleware::from_fn_with_state, Router, routing::{get, post, delete}};
use util::state::AppState;
use crate::auth::guards::{require_assigned_to_module, require_lecturer};

pub mod post;

pub fn grade_routes(app_state: AppState) -> Router<AppState> {
    Router::new()
}