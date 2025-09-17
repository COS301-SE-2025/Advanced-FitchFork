use super::common::AttendanceIncoming;
use serde_json::json;
use util::ws::handler_trait::{WsHandler, async_trait};
use util::ws::runtime::WsContext;

pub struct AttendanceWsHandler;

#[async_trait]
impl WsHandler for AttendanceWsHandler {
    type In = AttendanceIncoming;

    async fn on_open(&self, _ctx: &WsContext) {}

    async fn on_message(&self, ctx: &WsContext, _msg: Self::In) {
        // Explicit app-level pong (framework auto-pongs to {"type":"ping"} too)
        let _ = ctx
            .reply_text(
                json!({
                    "event": "pong",
                    "topic": ctx.topic,
                    "payload": {},
                    "ts": chrono::Utc::now().to_rfc3339(),
                })
                .to_string(),
            )
            .await;
    }

    async fn on_close(&self, _ctx: &WsContext) {}
}
