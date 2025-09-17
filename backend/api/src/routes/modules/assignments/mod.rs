//! Assignment routes module.
//!
//! Provides the `/assignments` route group with full CRUD and nested functionality.
//!
//! Routes include:
//! - Create, read, update, delete assignments (single and bulk)
//! - Open/close assignments
//! - Assignment stats and readiness checks
//! - Nested routes for tasks, config, memo output, mark allocation, submissions, files, interpreter, tickets, plagiarism, grades, and starter packs
//!
//! Access control is enforced via middleware guards for lecturers, assistants, and assigned users.

use crate::{
    auth::guards::{
        require_assigned_to_module, require_assignment_access,
        require_lecturer_or_assistant_lecturer, require_ready_assignment,
    },
    routes::modules::assignments::{post::verify_assignment_pin, statistics::statistics_routes},
};
use axum::{
    Router,
    middleware::from_fn_with_state,
    routing::{delete, get, post, put},
};
use config::config_routes;
use delete::{bulk_delete_assignments, delete_assignment};
use files::files_routes;
use get::{get_assignment, get_assignment_readiness, get_assignments};
use grades::grade_routes;
use interpreter::interpreter_routes;
use mark_allocator::mark_allocator_routes;
use memo_output::memo_output_routes;
use overwrite_files::overwrite_file_routes;
use plagiarism::plagiarism_routes;
use post::create_assignment;
use put::{bulk_update_assignments, close_assignment, edit_assignment, open_assignment};
use submissions::submission_routes;
use tasks::tasks_routes;
use tickets::ticket_routes;
use util::state::AppState;

pub mod common;
pub mod config;
pub mod delete;
pub mod files;
pub mod get;
pub mod grades;
pub mod interpreter;
pub mod mark_allocator;
pub mod memo_output;
pub mod overwrite_files;
pub mod plagiarism;
pub mod post;
pub mod put;
pub mod starter;
pub mod statistics;
pub mod submissions;
pub mod tasks;
pub mod tickets;

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
/// - `GET    /assignments/:assignment_id/readiness`      → Assignment readiness (lecturer or admin only)
///
/// Nested routes:
/// - Tasks routes                  → `tasks_routes`
/// - Config routes                 → `config_routes`
/// - Memo output routes            → `memo_output_routes`
/// - Mark allocator routes         → `mark_allocator_routes`
/// - Submissions routes            → `submission_routes`
/// - Files routes                  → `files_routes`
/// - Interpreter routes            → `interpreter_routes`
/// - Tickets routes                → `ticket_routes`
/// - Plagiarism routes             → `plagiarism_routes`
/// - Grades routes                 → `grade_routes`
/// - Overwrite files routes        → `overwrite_file_routes`
/// - Statistics routes             → `statistics_routes`
/// - Starter routes                → `starter_routes`
pub fn assignment_routes(app_state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            "/",
            post(create_assignment).route_layer(from_fn_with_state(
                app_state.clone(),
                require_lecturer_or_assistant_lecturer,
            )),
        )
        .route(
            "/",
            get(get_assignments).route_layer(from_fn_with_state(
                app_state.clone(),
                require_assigned_to_module,
            )),
        )
        .route(
            "/{assignment_id}",
            get(get_assignment)
                .route_layer(from_fn_with_state(
                    app_state.clone(),
                    require_assigned_to_module,
                ))
                .route_layer(from_fn_with_state(
                    app_state.clone(),
                    require_assignment_access,
                )),
        )
        .route(
            "/{assignment_id}",
            put(edit_assignment).route_layer(from_fn_with_state(
                app_state.clone(),
                require_lecturer_or_assistant_lecturer,
            )),
        )
        .route(
            "/{assignment_id}",
            delete(delete_assignment).route_layer(from_fn_with_state(
                app_state.clone(),
                require_lecturer_or_assistant_lecturer,
            )),
        )
        .route(
            "/{assignment_id}/open",
            put(open_assignment)
                .route_layer(from_fn_with_state(
                    app_state.clone(),
                    require_lecturer_or_assistant_lecturer,
                ))
                .route_layer(from_fn_with_state(
                    app_state.clone(),
                    require_ready_assignment,
                )),
        )
        .route(
            "/{assignment_id}/close",
            put(close_assignment)
                .layer(from_fn_with_state(
                    app_state.clone(),
                    require_lecturer_or_assistant_lecturer,
                ))
                .route_layer(from_fn_with_state(
                    app_state.clone(),
                    require_ready_assignment,
                )),
        )
        .route(
            "/bulk",
            delete(bulk_delete_assignments).layer(from_fn_with_state(
                app_state.clone(),
                require_lecturer_or_assistant_lecturer,
            )),
        )
        .route(
            "/bulk",
            put(bulk_update_assignments).layer(from_fn_with_state(
                app_state.clone(),
                require_lecturer_or_assistant_lecturer,
            )),
        )
        .route(
            "/{assignment_id}/readiness",
            get(get_assignment_readiness)
                .route_layer(from_fn_with_state(
                    app_state.clone(),
                    require_assigned_to_module,
                ))
                .route_layer(from_fn_with_state(
                    app_state.clone(),
                    require_assignment_access,
                )),
        )
        .nest(
            "/{assignment_id}/tasks",
            tasks_routes().route_layer(from_fn_with_state(
                app_state.clone(),
                require_lecturer_or_assistant_lecturer,
            )),
        )
        .nest(
            "/{assignment_id}/config",
            config_routes().layer(from_fn_with_state(
                app_state.clone(),
                require_lecturer_or_assistant_lecturer,
            )),
        )
        .nest(
            "/{assignment_id}/memo_output",
            memo_output_routes().layer(from_fn_with_state(
                app_state.clone(),
                require_lecturer_or_assistant_lecturer,
            )),
        )
        .nest(
            "/{assignment_id}/mark_allocator",
            mark_allocator_routes().route_layer(from_fn_with_state(
                app_state.clone(),
                require_lecturer_or_assistant_lecturer,
            )),
        )
        .nest(
            "/{assignment_id}/submissions",
            submission_routes(app_state.clone())
                .route_layer(from_fn_with_state(
                    app_state.clone(),
                    require_assigned_to_module,
                ))
                .route_layer(from_fn_with_state(
                    app_state.clone(),
                    require_assignment_access,
                )),
        )
        .nest("/{assignment_id}/files", files_routes(app_state.clone()))
        .nest(
            "/{assignment_id}/interpreter",
            interpreter_routes(app_state.clone()),
        )
        .nest(
            "/{assignment_id}/tickets",
            ticket_routes(app_state.clone())
                .route_layer(from_fn_with_state(
                    app_state.clone(),
                    require_assigned_to_module,
                ))
                .route_layer(from_fn_with_state(
                    app_state.clone(),
                    require_assignment_access,
                )),
        )
        .nest(
            "/{assignment_id}/plagiarism",
            plagiarism_routes()
                .route_layer(from_fn_with_state(
                    app_state.clone(),
                    require_assigned_to_module,
                ))
                .route_layer(from_fn_with_state(
                    app_state.clone(),
                    require_lecturer_or_assistant_lecturer,
                )),
        )
        .nest("/{assignment_id}/grades", grade_routes(app_state.clone()))
        .nest(
            "/{assignment_id}/overwrite_files",
            overwrite_file_routes(app_state.clone()).layer(from_fn_with_state(
                app_state.clone(),
                require_lecturer_or_assistant_lecturer,
            )),
        )
        .nest(
            "/{assignment_id}/stats",
            statistics_routes(app_state.clone()),
        )
        .route("/{assignment_id}/verify", post(verify_assignment_pin))
        .nest(
            "/{assignment_id}/starter",
            starter::routes().route_layer(from_fn_with_state(
                app_state.clone(),
                require_lecturer_or_assistant_lecturer,
            )),
        )
}
