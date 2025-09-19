use axum::{
    Router,
    middleware::from_fn_with_state,
    routing::{delete, get, post, put},
};
use util::state::AppState;

mod common;
mod delete;
mod get;
mod post;
mod put;

pub use delete::delete_session;
pub use get::{
    export_session_records_csv, get_session, get_session_code, list_session_records, list_sessions,
};
pub use post::{create_session, mark_attendance};
pub use put::edit_session;

use crate::{
    auth::guards::require_lecturer_or_assistant_lecturer,
    routes::modules::attendance::post::mark_attendance_by_username,
};

pub fn attendance_routes(app_state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            "/sessions",
            get(list_sessions).route_layer(from_fn_with_state(
                app_state.clone(),
                require_lecturer_or_assistant_lecturer,
            )),
        )
        .route(
            "/sessions",
            post(create_session).route_layer(from_fn_with_state(
                app_state.clone(),
                require_lecturer_or_assistant_lecturer,
            )),
        )
        .route(
            "/sessions/{session_id}",
            get(get_session).route_layer(from_fn_with_state(
                app_state.clone(),
                require_lecturer_or_assistant_lecturer,
            )),
        )
        .route(
            "/sessions/{session_id}",
            put(edit_session).route_layer(from_fn_with_state(
                app_state.clone(),
                require_lecturer_or_assistant_lecturer,
            )),
        )
        .route(
            "/sessions/{session_id}",
            delete(delete_session).route_layer(from_fn_with_state(
                app_state.clone(),
                require_lecturer_or_assistant_lecturer,
            )),
        )
        .route(
            "/sessions/{session_id}/code",
            get(get_session_code).route_layer(from_fn_with_state(
                app_state.clone(),
                require_lecturer_or_assistant_lecturer,
            )),
        )
        .route("/sessions/{session_id}/mark", post(mark_attendance))
        .route(
            "/sessions/{session_id}/mark/by-username",
            post(mark_attendance_by_username).route_layer(from_fn_with_state(
                app_state.clone(),
                require_lecturer_or_assistant_lecturer,
            )),
        )
        .route(
            "/sessions/{session_id}/records",
            get(list_session_records).route_layer(from_fn_with_state(
                app_state.clone(),
                require_lecturer_or_assistant_lecturer,
            )),
        )
        .route(
            "/sessions/{session_id}/records/export",
            get(export_session_records_csv).route_layer(from_fn_with_state(
                app_state.clone(),
                require_lecturer_or_assistant_lecturer,
            )),
        )
        .with_state(app_state)
}
