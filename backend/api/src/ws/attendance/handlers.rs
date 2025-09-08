use std::sync::Arc;
use axum::{
    extract::{Path, State, WebSocketUpgrade},
    response::IntoResponse,
    Extension,
};
use util::state::AppState;
use util::ws::axum_adapter::ws_route;
use util::ws::serve::WsServerOptions;

use crate::auth::AuthUser;
use super::topics::attendance_session_topic;
use super::ws_handlers::AttendanceWsHandler;

pub async fn attendance_session_ws_handler(
    ws: WebSocketUpgrade,
    State(app_state): State<AppState>,
    Extension(AuthUser(claims)): Extension<AuthUser>,
    Path(session_id): Path<i64>,
) -> impl IntoResponse {
    // Presence-enabled: pass the authenticated user id (Some(uid))
    let uid_opt = Some(claims.sub);

    // Per-connection handler
    let handler = Arc::new(AttendanceWsHandler);

    // Topic factory (captures `session_id`)
    let topic = move || attendance_session_topic(session_id);

    // Optional server opts (defaults: ~30s ping, app ping/pong enabled)
    let opts = WsServerOptions::default();

    // Hand off to the adapter (same as tickets)
    ws_route(ws, State(app_state), Extension(uid_opt), topic, handler, opts).await
}