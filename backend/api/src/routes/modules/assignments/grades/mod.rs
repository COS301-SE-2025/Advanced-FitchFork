//! Assignment grades routes module.
//!
//! Provides the `/grades` route group nested under assignments.
//!
//! Routes include:
//! - List grades for an assignment (with pagination, search, and sorting)
//! - Export grades as CSV
//!
//! Access control:
//! - Students see only their own grade (list endpoint)
//! - Staff-only access for export (enforced upstream as per router setup)

use crate::auth::guards::allow_assistant_lecturer;
use axum::{Router, middleware::from_fn_with_state, routing::get};
use get::{export_grades, list_grades};
use util::state::AppState;

pub mod get;

/// Expects an assignment ID and module ID.
/// Builds and returns the `/grades` route group.
///
/// Routes:
/// - `GET /grades`         → List grades (students restricted to own)
/// - `GET /grades/export`  → Export grades as CSV
// TODO: Write tests for GET /grades
// TODO: Write tests for GET /grades/export
pub fn grade_routes(app_state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            "/",
            get(list_grades).route_layer(from_fn_with_state(
                app_state.clone(),
                allow_assistant_lecturer,
            )),
        )
        .route(
            "/export",
            get(export_grades).route_layer(from_fn_with_state(
                app_state.clone(),
                allow_assistant_lecturer,
            )),
        )
}
