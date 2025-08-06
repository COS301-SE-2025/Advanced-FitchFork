use axum::{middleware::from_fn_with_state, Router, routing::{get, post, put, delete}};
use delete::{delete_assignment, bulk_delete_assignments};
use get::{get_assignment, get_assignments, get_assignment_stats, get_assignment_readiness};
use post::create_assignment;
use put::{edit_assignment, bulk_update_assignments, open_assignment, close_assignment};
use mark_allocator::mark_allocator_routes;
use config::config_routes;
use memo_output::memo_output_routes;
use submissions::submission_routes;
use files::files_routes;
use tasks::tasks_routes;
use tickets::ticket_routes;
use plagiarism::plagiarism_routes;
use util::state::AppState;
use crate::auth::guards::{require_assigned_to_module, require_lecturer, require_lecturer_or_assistant_lecturer};

pub mod config;
pub mod delete;
pub mod get;
pub mod post;
pub mod put;
pub mod mark_allocator;
pub mod submissions;
pub mod files;
pub mod memo_output;
pub mod tasks;
pub mod common;
pub mod tickets;
pub mod plagiarism;

/// Expects a module ID.
/// If an assignment ID is included it will be modified or deleted.
///
/// Builds and returns the `/assignments` route group.
///
/// Routes:
/// - `POST   /assignments`                               → Create a new assignment (requires lecturer)
/// - `GET    /assignments`                               → List assignments
/// - `DELETE /assignments/bulk`                          → Bulk delete assignments (requires lecturer)
/// - `PUT    /assignments/bulk`                          → Bulk edit assignments (requires lecturer, cannot edit status)
/// - `GET    /assignments/:assignment_id`                → Get assignment details
/// - `PUT    /assignments/:assignment_id`                → Edit assignment (requires lecturer, cannot edit status)
/// - `PUT    /assignments/:assignment_id/open`           → Open assignment (requires lecturer, only if currently Ready, Closed, or Archived)
/// - `PUT    /assignments/:assignment_id/close`          → Close assignment (requires lecturer, only if currently Open)
/// - `DELETE /assignments/:assignment_id`                → Delete assignment (requires lecturer)
/// - `GET    /assignments/:assignment_id/stats`          → Assignment statistics (lecturer only)
/// - `GET    /assignments/:assignment_id/readiness`      → Assignment readiness (lecturer or admin only)
///
/// Nested routes:
/// - Tasks routes                  → `tasks_routes`
/// - Config routes                 → `config_routes`
/// - Memo output routes            → `memo_output_routes`
/// - Mark allocator routes         → `mark_allocator_routes`
/// - Submissions routes            → `submission_routes`
/// - Files routes                  → `files_routes`
pub fn assignment_routes(app_state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", post(create_assignment).route_layer(from_fn_with_state(app_state.clone(), require_lecturer)))
        .route("/", get(get_assignments).route_layer(from_fn_with_state(app_state.clone(), require_assigned_to_module)))
        .route("/{assignment_id}", get(get_assignment).route_layer(from_fn_with_state(app_state.clone(), require_assigned_to_module)))
        .route("/{assignment_id}", put(edit_assignment).route_layer(from_fn_with_state(app_state.clone(), require_lecturer)))
        .route("/{assignment_id}", delete(delete_assignment).route_layer(from_fn_with_state(app_state.clone(), require_lecturer)))
        .route("/{assignment_id}/open", put(open_assignment).route_layer(from_fn_with_state(app_state.clone(), require_lecturer)))
        .route("/{assignment_id}/close", put(close_assignment).layer(from_fn_with_state(app_state.clone(), require_lecturer)))
        .route("/bulk", delete(bulk_delete_assignments).layer(from_fn_with_state(app_state.clone(), require_lecturer)))
        .route("/bulk", put(bulk_update_assignments).layer(from_fn_with_state(app_state.clone(), require_lecturer)))
        .route("/{assignment_id}/stats", get(get_assignment_stats).route_layer(from_fn_with_state(app_state.clone(), require_lecturer)))
        .route("/{assignment_id}/readiness", get(get_assignment_readiness).route_layer(from_fn_with_state(app_state.clone(), require_lecturer)))
        .nest("/{assignment_id}/tasks", tasks_routes().route_layer(from_fn_with_state(app_state.clone(), require_lecturer)))
        .nest("/{assignment_id}/config", config_routes().layer(from_fn_with_state(app_state.clone(), require_lecturer)))
        .nest("/{assignment_id}/memo_output", memo_output_routes().layer(from_fn_with_state(app_state.clone(), require_lecturer)))
        .nest("/{assignment_id}/mark_allocator", mark_allocator_routes().route_layer(from_fn_with_state(app_state.clone(), require_lecturer)))
        .nest( "/{assignment_id}/submissions", submission_routes(app_state.clone()).route_layer(from_fn_with_state(app_state.clone(), require_assigned_to_module)))
        .nest("/{assignment_id}/files", files_routes(app_state.clone()))
        .nest("/{assignment_id}/tickets", ticket_routes(app_state.clone()).route_layer(from_fn_with_state(app_state.clone(), require_assigned_to_module)))
        .nest("/{assignment_id}/plagiarism", plagiarism_routes().route_layer(from_fn_with_state(app_state.clone(), require_lecturer_or_assistant_lecturer)))
}