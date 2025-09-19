use super::common::TicketIncoming;
use serde_json::json;
use util::ws::handler_trait::WsHandler;
use util::ws::handler_trait::async_trait;
use util::ws::runtime::WsContext;

pub struct TicketWsHandler;

#[async_trait]
impl WsHandler for TicketWsHandler {
    type In = TicketIncoming;

    async fn on_open(&self, _ctx: &WsContext) {
        // Optional: announce join, preload state, etc.
        // _ctx.emit("joined", &json!({ "ok": true })).await;
    }

    async fn on_message(&self, ctx: &WsContext, msg: Self::In) {
        match msg {
            TicketIncoming::Typing { sender } => {
                // Broadcast "typing" on the topic using your standard envelope
                ctx.emit("typing", &json!({ "sender": sender })).await;
            }
            TicketIncoming::Ping => {
                // App-level pong to THIS client (framework also auto-responds to {"type":"ping"})
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
        }
    }

    async fn on_close(&self, _ctx: &WsContext) {
        // Optional: cleanup
    }
}
