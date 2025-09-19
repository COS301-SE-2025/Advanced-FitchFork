use axum::{Router, middleware::from_fn, routing::get};
use util::state::AppState;

use crate::auth::guards::require_admin;

pub mod get;
pub mod post;

pub fn system_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/code-manager/max-concurrent",
            get(get::get_max_concurrent_handler).post(post::set_max_concurrent_handler),
        )
        .route("/metrics", get(get::get_metrics))
        .route("/metrics/export", get(get::get_metrics_csv))
        .route("/submissions", get(get::submissions_over_time))
        .route(
            "/submissions/export",
            get(get::submissions_over_time_export),
        )
        .route_layer(from_fn(require_admin))
}
