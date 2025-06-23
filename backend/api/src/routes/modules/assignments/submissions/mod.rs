use axum::middleware::from_fn;
use axum::{extract::Path, Router};

pub mod get;
pub mod post;
use axum::routing::get;
use get::{list_submissions};

use crate::auth::guards::{require_assigned_to_module};

/// Defines routes related to assignment submissions.
///
/// # Routes
/// - `GET  /submissions`  
///   → List all submissions for the assignment (lecturer or tutor access only)

pub fn submission_routes() -> Router {
    Router::new()
        .route(
            "/",
            get(list_submissions).layer(from_fn(|Path(params): Path<(i64,)>, req, next| {
                require_assigned_to_module(Path(params), req, next)
            })),
        )
}
