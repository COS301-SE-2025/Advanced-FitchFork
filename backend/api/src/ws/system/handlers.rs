use axum::{
    Extension,
    extract::{State, WebSocketUpgrade},
    response::IntoResponse,
};
use std::sync::Arc;
use util::state::AppState;
use util::ws::axum_adapter::ws_route;
use util::ws::default::ws_handler::DefaultWsHandler;
use util::ws::serve::WsServerOptions;

use super::topics::{system_health_admin_topic, system_health_topic};
use crate::auth::AuthUser;

pub async fn system_health_ws_handler(
    ws: WebSocketUpgrade,
    State(app_state): State<AppState>,
    Extension(AuthUser(claims)): Extension<AuthUser>,
) -> impl IntoResponse {
    let uid_opt = Some(claims.sub);
    let handler = Arc::new(DefaultWsHandler);
    let topic = move || system_health_topic();
    let opts = WsServerOptions::default();
    ws_route(
        ws,
        State(app_state),
        Extension(uid_opt),
        topic,
        handler,
        opts,
    )
    .await
}

pub async fn system_health_admin_ws_handler(
    ws: WebSocketUpgrade,
    State(app_state): State<AppState>,
    Extension(AuthUser(claims)): Extension<AuthUser>,
) -> impl IntoResponse {
    let uid_opt = Some(claims.sub);
    let handler = Arc::new(DefaultWsHandler);
    let topic = move || system_health_admin_topic();
    let opts = WsServerOptions::default();
    ws_route(
        ws,
        State(app_state),
        Extension(uid_opt),
        topic,
        handler,
        opts,
    )
    .await
}
