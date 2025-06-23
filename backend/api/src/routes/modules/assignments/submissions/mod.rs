use axum::middleware::from_fn;
use axum::{extract::Path, Router};

pub mod get;
pub mod post;
pub mod delete;
use axum::routing::{get, delete};

use get::{list_submissions};
use delete::{delete_submissions};
use crate::auth::guards::{require_lecturer_or_admin};

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
        ).route(
            "/",
            delete(delete_submissions).layer(from_fn(
                |Path((module_id, assignment_id)): Path<(i64, i64)>, req, next| {
                    require_lecturer_or_admin(Path((module_id, assignment_id)), req, next)
                }
            )),
        )
}
