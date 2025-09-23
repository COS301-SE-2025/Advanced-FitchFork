use super::ws_types::DefaultIncoming;
use crate::ws::handler_trait::WsHandler;
use crate::ws::runtime::WsContext;

pub struct DefaultWsHandler;

impl WsHandler for DefaultWsHandler {
    type In = DefaultIncoming;

    async fn on_open(&self, _ctx: &WsContext) {
        // optional: announce joined, etc.
        // _ctx.emit("joined", &serde_json::json!({ "ok": true })).await;
    }

    async fn on_message(&self, ctx: &WsContext, msg: Self::In) {
        if let DefaultIncoming::Envelope { text } = msg {
            // broadcast the raw text to the default topic
            ctx.broadcast_text(text).await;
        }
        // Other JSON shapes are ignored; {"type":"ping"} is auto-ponged by the server
    }

    async fn on_close(&self, _ctx: &WsContext) {}
}
