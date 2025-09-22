//! # Module Personnel Routes
//!
//! Defines and wires up routes for the `/modules/{module_id}/personnel` endpoint group.
//!
//! ## Structure
//! - `post.rs` — POST handlers (e.g., assign users to a role in a module)
//! - `get.rs` — GET handlers (e.g., fetch assigned or eligible users)
//! - `delete.rs` — DELETE handlers (e.g., remove users from a module role)
//!
//! ## Usage
//! Called via `modules_routes()` as a nested router mounted under `/modules/{module_id}/personnel`.
//! This route group is protected by `require_lecturer` middleware in the parent router.

use axum::{
    Router,
    routing::{delete, get, post},
};

mod delete;
mod get;
mod post;

/// Builds and returns the `/modules/{module_id}/personnel` route group.
///
/// Routes:
/// - `GET    /personnel`           → get all assigned users grouped by role
/// - `POST   /personnel`           → assign one or more users to a role
/// - `DELETE /personnel`           → remove one or more users from a role
/// - `GET    /personnel/eligible`  → fetch users not assigned to any role in the module
pub fn personnel_routes() -> Router {
    Router::new()
        .route("/", get(get::get_personnel))
        .route("/", post(post::assign_personnel))
        .route("/", delete(delete::remove_personnel))
        .route("/eligible", get(get::get_eligible_users_for_module))
}
