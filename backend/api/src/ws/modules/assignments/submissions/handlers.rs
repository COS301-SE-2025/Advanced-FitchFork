use super::topics::submission_topic;
use super::ws_handlers::SubmissionWsHandler;
use crate::auth::AuthUser;
use axum::{
    Extension,
    extract::{Path, State, WebSocketUpgrade},
    response::IntoResponse,
};
use std::sync::Arc;
use util::state::AppState;
use util::ws::axum_adapter::ws_route;
use util::ws::serve::WsServerOptions;

pub async fn submission_ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Extension(AuthUser(claims)): Extension<AuthUser>,
    Path((module_id, assignment_id, user_id)): Path<(i64, i64, i64)>,
) -> impl IntoResponse {
    let uid_opt = Some(claims.sub);
    let handler = Arc::new(SubmissionWsHandler);
    let topic = move || submission_topic(module_id, assignment_id, user_id);
    let opts = WsServerOptions::default();

    ws_route(ws, State(state), Extension(uid_opt), topic, handler, opts).await
}
