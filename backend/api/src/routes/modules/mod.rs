//! # Modules Routes Module
//!
//! Defines and wires up routes for the `/modules` endpoint group.
//!
//! ## Structure
//! - `post.rs` — POST handlers (e.g., create module, assign lecturers)
//! - `get.rs` — GET handlers (e.g., fetch modules, lecturers, students)
//! - `put.rs` — PUT handlers (e.g., edit module)
//! - `delete.rs` — DELETE handlers (e.g., remove lecturers, students, tutors)
//! - `assignments.rs` — nested assignment routes under modules
//!
//! ## Usage
//! Call `modules_routes()` to get a configured `Router` for `/modules` to be mounted in the main app.

pub mod assignments;
pub mod delete;
pub mod get;
pub mod post;
pub mod put;

use crate::auth::guards::require_admin;
use assignments::assignment_routes;
use axum::{
    routing::{delete, get, post, put},
    Router,
};
use delete::{remove_lecturers, remove_students, remove_tutors};
use get::{get_lecturers, get_module, get_modules, get_students, get_tutors, get_my_details};
use post::{assign_lecturers, assign_students, assign_tutors, create};
use put::edit_module;

/// Builds and returns the `/modules` route group.
///
/// Routes:
/// - `GET    /modules`                 → list all modules
/// - `POST   /modules`                 → create a new module (admin only)
/// - `GET    /modules/:module_id`     → get a single module by ID
/// - `PUT    /modules/:module_id`     → edit module details (admin only)
///
/// - `POST   /modules/:module_id/lecturers` → assign lecturers (admin only)
/// - `POST   /modules/:module_id/students`  → assign students (admin only)
/// - `POST   /modules/:module_id/tutors`    → assign tutors (admin only)
///
/// - `GET    /modules/:module_id/lecturers` → get lecturers assigned to module
/// - `GET    /modules/:module_id/students`  → get students assigned to module
/// - `GET    /modules/:module_id/tutors`    → get tutors assigned to module
///
/// - `DELETE /modules/:module_id/lecturers` → remove lecturers from module (admin only)
/// - `DELETE /modules/:module_id/students`  → remove students from module (admin only)
/// - `DELETE /modules/:module_id/tutors`    → remove tutors from module (admin only)
///
/// - Nested assignments routes under `/modules/:module_id/assignments`
///
/// All routes are protected by `require_admin` middleware.
pub fn modules_routes() -> Router {
    Router::new()
        .route("/", get(get_modules))        // Public: list modules
        .route("/", post(create))             // Admin: create module
        .route("/me", get(get_my_details))
        .route("/:module_id/lecturers", post(assign_lecturers))
        .route("/:module_id/students", post(assign_students))
        .route("/:module_id/tutors", post(assign_tutors))
        .route("/:module_id/lecturers", delete(remove_lecturers))
        .route("/:module_id/students", delete(remove_students))
        .route("/:module_id/tutors", delete(remove_tutors))
        .route("/:module_id/lecturers", get(get_lecturers))
        .route("/:module_id/students", get(get_students))
        .route("/:module_id/tutors", get(get_tutors))
        .route("/:module_id", put(edit_module))
        .route("/:module_id", get(get_module))
        .nest("/:module_id/assignments", assignment_routes())
        .route_layer(axum::middleware::from_fn(require_admin))
}
