use std::sync::Arc;
use axum::{
    extract::WebSocketUpgrade,
    response::IntoResponse,
};
use crate::state::AppState;
use crate::ws::axum_adapter::ws_route;
use crate::ws::serve::WsServerOptions;

use crate::ws::default::ws_handler::DefaultWsHandler;

pub async fn default_websocket_handler(
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    // Fixed topic; clients don’t specify it
    let topic = || "__default".to_string();

    let handler = Arc::new(DefaultWsHandler);
    let opts = WsServerOptions::default(); // 30s WS pings; app-level {"type":"ping"} auto-pong

    // Unauthenticated default channel → no presence tracking
    ws_route(ws, State(state), axum::Extension(None::<i64>), topic, handler, opts).await
}