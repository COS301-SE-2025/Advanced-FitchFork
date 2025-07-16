//! # Modules Routes Module
//!
//! Defines and wires up routes for the `/api/modules` endpoint group.
//!
//! ## Structure
//! - `post.rs` — POST handlers (e.g., create module, assign lecturers)
//! - `get.rs` — GET handlers (e.g., fetch modules, lecturers, students)
//! - `put.rs` — PUT handlers (e.g., edit module, edit lecturers)
//! - `delete.rs` — DELETE handlers (e.g., remove lecturers, students, tutors)
//! - `assignments.rs` — nested assignment routes under modules
//!
//! ## Usage
//! Call `modules_routes()` to get a configured `Router` for `/modules` to be mounted in the main app.

pub mod assignments;
pub mod lecturers;
pub mod tutors;
pub mod students;
pub mod delete;
pub mod get;
pub mod post;
pub mod put;
pub mod common;

use crate::auth::guards::require_admin;
use assignments::assignment_routes;
use lecturers::lecturer_routes;
use tutors::tutor_routes;
use students::student_routes;
use axum::{
    middleware::from_fn,
    routing::{delete, get, post, put},
    Router,
};
use delete::delete_module;
use get::{
    get_eligible_users_for_module,
    get_module,
    get_modules,
    get_my_details,
};
use post::create;
use put::edit_module;

/// Builds and returns the `/modules` route group.
///
/// Routes:
/// - `GET    /modules`                 → list all modules
/// - `POST   /modules`                 → create a new module (admin only)
/// - `GET    /modules/{module_id}`     → get a single module by ID
/// - `PUT    /modules/{module_id}`     → edit module details (admin only)
/// - `DELETE /modules/{module_id}`     → delete a module entirely (admin only)
///
/// - Nested assignments routes under `/modules/{module_id}/assignments`
/// - Nested lecturers routes under `/modules/{module_id}/lecturers`
/// - Nested tutors routes under `/modules/{module_id}/tutors`
/// - Nested students routes under `/modules/{module_id}/students`
///
/// All modifying routes are protected by `require_admin` middleware.
pub fn modules_routes() -> Router {
    Router::new()
        .route(
            "/",
            get(get_modules)
        )
        .route(
            "/me",
            get(get_my_details)
        )
        .route(
            "/{module_id}/eligible-users",
            get(get_eligible_users_for_module).route_layer(from_fn(require_admin)),
        )
        .route(
            "/{module_id}",
            get(get_module)
        )
        .route(
            "/",
            post(create).route_layer(from_fn(require_admin))
        )
        .route(
            "/{module_id}",
            put(edit_module).route_layer(from_fn(require_admin))
        )
        .route(
            "/{module_id}",
            delete(delete_module).route_layer(from_fn(require_admin))
        )
        .nest(
            "/{module_id}/assignments",
            assignment_routes()
        )
        .nest(
            "/{module_id}/lecturers",
            lecturer_routes()
        )
        .nest(
            "/{module_id}/tutors",
            tutor_routes()
        )
        .nest(
            "/{module_id}/students",
            student_routes()
        )
}