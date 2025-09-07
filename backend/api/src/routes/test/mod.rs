//! Test-only routes (mounted under `/api/test` in non-production envs).
//!
//! These endpoints exist solely for E2E/integration tests (seed, lookup, teardown).
//! They must never be exposed in production.
//!
//! # Endpoints
//! - POST   `/api/test/users`               – Create or update a user (idempotent, returns `id`).
//! - GET    `/api/test/users?username=NAME` – Fetch a user by username.
//! - DELETE `/api/test/users/{id}`          – Delete a user by numeric ID.

use axum::{
    routing::{delete, post},
    Router,
};
use util::state::AppState;

mod common;
mod get;
mod post;
mod delete;

pub use common::{TestUserResponse, UpsertUserRequest};
pub use get::get_user;
pub use post::upsert_user;
pub use delete::delete_user;

/// Registers the test routes on a `Router<AppState>`.
///
/// Why mount here (not in `main`):
/// - Keeps env-conditional test endpoints colocated with handlers and types.
/// - Avoids changing the router type in `main` (prevents trait-bound surprises).
/// - Improves discoverability and maintenance of the test API surface.
///
/// ## Available routes
/// - **POST** `/api/test/users`  
///   Create or update a test user. Idempotent — if `username` exists, updates instead.
///   Always returns the `id` so it can be deleted in teardown.
/// - **GET** `/api/test/users?username=NAME`  
///   Fetch a test user by their username.
/// - **DELETE** `/api/test/users/{id}`  
///   Delete a test user by numeric ID.
pub fn test_routes(app_state: AppState) -> Router<AppState> {
    Router::new()
        .route("/users", post(upsert_user).get(get_user))
        .route("/users/{user_id}", delete(delete_user))
        .with_state(app_state)
}
