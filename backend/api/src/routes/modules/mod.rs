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

use crate::{
    auth::guards::{require_admin, require_assigned_to_module, require_lecturer},
    routes::modules::{
        announcements::announcement_routes, attendance::attendance_routes,
        personnel::personnel_routes,
    },
};
use assignments::assignment_routes;
use axum::{
    Router,
    middleware::{from_fn, from_fn_with_state},
    routing::{delete, get, post, put},
};
use delete::{bulk_delete_modules, delete_module};
use get::{get_module, get_modules, get_my_details};
use post::create;
use put::{bulk_edit_modules, edit_module};
use util::state::AppState;

pub mod announcements;
pub mod assignments;
pub mod attendance;
pub mod common;
pub mod delete;
pub mod get;
pub mod personnel;
pub mod post;
pub mod put;

/// Builds and returns the `/modules` route group.
///
/// Routes:
/// - `GET    /modules`                 → list all modules
/// - `POST   /modules`                 → create a new module (admin only)
/// - `GET    /modules/{module_id}`     → get a single module by ID
/// - `PUT    /modules/{module_id}`     → edit module details (admin only)
/// - `DELETE /modules/{module_id}`     → delete a module entirely (admin only)
///
/// - Nested students routes under `/modules/{module_id}/students`
/// - Nested personnel routes under `/modules/{module_id}/personnel`
///
/// All modifying routes are protected by `require_admin` middleware.
pub fn modules_routes(app_state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(get_modules))
        .route("/me", get(get_my_details))
        .route(
            "/{module_id}",
            get(get_module).route_layer(from_fn_with_state(
                app_state.clone(),
                require_assigned_to_module,
            )),
        )
        .route("/", post(create).route_layer(from_fn(require_admin)))
        .route(
            "/{module_id}",
            put(edit_module).route_layer(from_fn(require_admin)),
        )
        .route(
            "/{module_id}",
            delete(delete_module).route_layer(from_fn(require_admin)),
        )
        .route(
            "/bulk",
            delete(bulk_delete_modules).route_layer(from_fn(require_admin)),
        )
        .route(
            "/bulk",
            put(bulk_edit_modules).route_layer(from_fn(require_admin)),
        )
        .nest(
            "/{module_id}/assignments",
            assignment_routes(app_state.clone()),
        )
        .nest(
            "/{module_id}/personnel",
            personnel_routes().route_layer(from_fn_with_state(app_state.clone(), require_lecturer)),
        )
        .nest(
            "/{module_id}/announcements",
            announcement_routes(app_state.clone()).route_layer(from_fn_with_state(
                app_state.clone(),
                require_assigned_to_module,
            )),
        )
        .nest(
            "/{module_id}/attendance",
            attendance_routes(app_state.clone()).route_layer(from_fn_with_state(
                app_state.clone(),
                require_assigned_to_module,
            )),
        )
}
