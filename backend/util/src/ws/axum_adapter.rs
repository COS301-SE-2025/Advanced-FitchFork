// util/ws/axum_adapter.rs
use super::handler_trait::WsHandler;
use super::serve::{WsServerOptions, serve_topic};
use crate::state::AppState;
use axum::{
    Extension,
    extract::{State, WebSocketUpgrade, ws::WebSocket},
    response::IntoResponse,
};
use std::sync::Arc;

pub async fn ws_route<H, FTopic, Uid>(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Extension(user_id): Extension<Uid>, // could be Option<AuthUser> or AuthUser
    topic_fn: FTopic,
    handler: Arc<H>,
    opts: WsServerOptions,
) -> impl IntoResponse
where
    H: WsHandler,
    FTopic: Fn() -> String + Send + 'static,
    Uid: Into<Option<i64>> + Clone + Send + Sync + 'static,
{
    let ws_manager = AppState::get().ws();
    let uid_opt = user_id.into();

    ws.on_upgrade(move |socket: WebSocket| {
        let topic = topic_fn();
        let handler = Arc::clone(&handler);
        let manager = ws_manager.clone();
        async move {
            serve_topic(socket, manager, topic, uid_opt, handler, opts).await;
        }
    })
}
