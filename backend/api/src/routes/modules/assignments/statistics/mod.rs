//! Assignment statistics routes.
//!
//! Base path (mounted by parent): /api/modules/{module_id}/assignments/{assignment_id}/stats
//!
//! Routes:
//! - GET /summary  — Aggregated submission summary (lecturer/assistant lecturer/admin)

use axum::{
    middleware::from_fn_with_state,
    routing::get,
    Router,
};

use util::state::AppState;
use crate::{auth::guards::allow_lecturer_or_assistant_lecturer};
use get::get_assignment_stats;

pub mod get;

pub fn statistics_routes(app_state: AppState) -> Router<AppState> {
    Router::new()
    .route("/",get(get_assignment_stats).route_layer(from_fn_with_state(app_state.clone(),allow_lecturer_or_assistant_lecturer)))
}
