use axum::middleware::from_fn;
use axum::{extract::Path, Router};

pub mod get;
pub mod post;
use axum::routing::get;
use get::get_user_submissions;

use crate::auth::guards::require_assigned_to_module;

/// Defines routes related to assignment submissions.
///
/// # Routes
/// - `GET  /submissions/:user_id`  
///   → Retrieve submissions by a specific user (Returns a users submissions, denies access to any other persons submissions if user is not lecturer/tuor)

pub fn submission_routes() -> Router {
    Router::new().route(
        "/:user_id",
        get(get_user_submissions).layer(from_fn(|Path(params): Path<(i64,)>, req, next| {
            require_assigned_to_module(Path(params), req, next)
        })),
    )
}
