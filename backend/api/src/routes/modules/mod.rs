//! # Modules Routes Module
//!
//! Defines and wires up routes for the `/modules` endpoint group.
//!
//! ## Structure
//! - `post.rs` — POST handlers (e.g., create module, assign lecturers)
//! - `get.rs` — GET handlers (e.g., fetch modules, lecturers, students)
//! - `put.rs` — PUT handlers (e.g., edit module, assign lecturers)
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

use crate::auth::guards::{
    require_admin,
    require_lecturer,
    require_student,
    require_tutor,
};
use assignments::assignment_routes;
use axum::{
    middleware::from_fn,
    routing::{delete, get, post, put},
    extract::Path,
    Router,
};
use delete::{delete_module, remove_lecturers, remove_students, remove_tutors};
use get::{
    get_eligible_users_for_module, get_lecturers, get_module, get_modules, get_my_details,
    get_students, get_tutors,
};
use post::{assign_lecturers, assign_students, assign_tutors, create};
use put::{edit_module, edit_lecturers, edit_students};

/// Builds and returns the `/modules` route group.
///
/// Routes:
/// - `GET    /modules`                 → list all modules
/// - `POST   /modules`                 → create a new module (admin only)
/// - `GET    /modules/:module_id`     → get a single module by ID
/// - `PUT    /modules/:module_id`     → edit module details (admin only)
/// - `DELETE /modules/:module_id`     → delete a module entirely (admin only)
///
/// - `POST   /modules/:module_id/lecturers` → assign lecturers (admin only)
/// - `PUT    /modules/:module_id/lecturers` → set lecturers (overwrites existing roles) (admin only)
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
/// All modifying routes are protected by `require_admin` middleware.
pub fn modules_routes() -> Router {
    Router::new()
        // Public or authenticated routes
        .route("/", get(get_modules))
        .route("/me", get(get_my_details))
        .route(
            "/:module_id/eligible-users",
            get(get_eligible_users_for_module).route_layer(from_fn(require_admin)),
        )
        .route("/:module_id", get(get_module))

        // Admin-only routes
        .route("/", post(create).route_layer(from_fn(require_admin)))
        .route("/:module_id", put(edit_module).route_layer(from_fn(require_admin)))
        .route("/:module_id", delete(delete_module).route_layer(from_fn(require_admin)))

        // Assign/remove routes (admin-only)
        .route(
            "/:module_id/lecturers",
            post(assign_lecturers).route_layer(from_fn(require_admin)),
        )
        .route(
            "/:module_id/lecturers",
            put(edit_lecturers).route_layer(from_fn(require_admin)),
        )
        .route(
            "/:module_id/students",
            put(edit_students).route_layer(from_fn(require_admin)),
        )
        .route(
            "/:module_id/students",
            post(assign_students).route_layer(from_fn(require_admin)),
        )
        .route(
            "/:module_id/tutors",
            post(assign_tutors).route_layer(from_fn(require_admin)),
        )
        .route(
            "/:module_id/lecturers",
            delete(remove_lecturers).route_layer(from_fn(require_admin)),
        )
        .route(
            "/:module_id/students",
            delete(remove_students).route_layer(from_fn(require_admin)),
        )
        .route(
            "/:module_id/tutors",
            delete(remove_tutors).route_layer(from_fn(require_admin)),
        )

        // Per-role view access
        .route(
            "/:module_id/lecturers",
            get(get_lecturers).route_layer(from_fn(|Path(params): axum::extract::Path<(i64,)>, req, next| {
                require_lecturer(axum::extract::Path(params), req, next)
            })),
        )
        .route(
            "/:module_id/students",
            get(get_students).route_layer(from_fn(|Path(params): axum::extract::Path<(i64,)>, req, next| {
                require_student(axum::extract::Path(params), req, next)
            })),
        )
        .route(
            "/:module_id/tutors",
            get(get_tutors).route_layer(from_fn(|Path(params): axum::extract::Path<(i64,)>, req, next| {
                require_tutor(axum::extract::Path(params), req, next)
            })),
        )

        // Nested assignments
        .nest("/:module_id/assignments", assignment_routes())
}