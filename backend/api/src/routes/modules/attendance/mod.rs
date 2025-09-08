use axum::{
    middleware::from_fn_with_state,
    routing::{get, post, put, delete},
    Router,
};
use util::state::AppState;

mod common;
mod get;
mod post;
mod put;
mod delete;

pub use get::{list_sessions, get_session, get_session_code, list_session_records, export_session_records_csv};
pub use post::{create_session, mark_attendance};
pub use put::edit_session;
pub use delete::delete_session;

use crate::auth::guards::{
    require_lecturer_or_assistant_lecturer,
};

pub fn attendance_routes(app_state: AppState) -> Router<AppState> {
    Router::new()
        .route("/sessions", get(list_sessions).route_layer(from_fn_with_state(app_state.clone(), require_lecturer_or_assistant_lecturer)))
        .route("/sessions", post(create_session).route_layer(from_fn_with_state(app_state.clone(), require_lecturer_or_assistant_lecturer)))
        .route("/sessions/{session_id}", get(get_session).route_layer(from_fn_with_state(app_state.clone(), require_lecturer_or_assistant_lecturer)))
        .route("/sessions/{session_id}", put(edit_session).route_layer(from_fn_with_state(app_state.clone(), require_lecturer_or_assistant_lecturer)))
        .route("/sessions/{session_id}", delete(delete_session).route_layer(from_fn_with_state(app_state.clone(), require_lecturer_or_assistant_lecturer)))
        .route("/sessions/{session_id}/code", get(get_session_code).route_layer(from_fn_with_state(app_state.clone(), require_lecturer_or_assistant_lecturer)))
        .route("/sessions/{session_id}/mark", post(mark_attendance))
        .route("/sessions/{session_id}/records", get(list_session_records).route_layer(from_fn_with_state(app_state.clone(), require_lecturer_or_assistant_lecturer)))
        .route("/sessions/{session_id}/records/export", get(export_session_records_csv).route_layer(from_fn_with_state(app_state.clone(), require_lecturer_or_assistant_lecturer)))
        .with_state(app_state)
}
