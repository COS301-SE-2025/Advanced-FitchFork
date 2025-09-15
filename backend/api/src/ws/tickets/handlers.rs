use std::sync::Arc;
use axum::{
    extract::{Path, WebSocketUpgrade},
    response::IntoResponse,
    Extension,
};
use util::ws::serve::WsServerOptions;
use util::ws::axum_adapter::ws_route;
use crate::auth::AuthUser;
use super::topics::ticket_chat_topic;
use super::ws_handlers::TicketWsHandler;

pub async fn ticket_chat_handler(
    ws: WebSocketUpgrade,
    Extension(AuthUser(claims)): Extension<AuthUser>,
    Path(ticket_id): Path<i64>,
) -> impl IntoResponse {
    let uid_opt = Some(claims.sub);              // presence enabled
    let handler = Arc::new(TicketWsHandler);
    let topic = move || ticket_chat_topic(ticket_id);
    let opts = WsServerOptions::default();       // 30s WS ping; auto app-ping → pong enabled

    // Adapter expects Extension<Uid> where Uid: Into<Option<i64>>
    ws_route(ws, Extension(uid_opt), topic, handler, opts).await
}
