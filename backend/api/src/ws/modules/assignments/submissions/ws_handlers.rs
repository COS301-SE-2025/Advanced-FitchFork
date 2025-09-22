use super::common::SubmissionIncoming;
use serde_json::json;
use util::ws::handler_trait::{async_trait, WsHandler};
use util::ws::runtime::WsContext;

pub struct SubmissionWsHandler;

#[inline]
fn now_ts() -> String {
    chrono::Utc::now().to_rfc3339()
}

#[async_trait]
impl WsHandler for SubmissionWsHandler {
    type In = SubmissionIncoming;

    async fn on_open(&self, ctx: &WsContext) {
        // minimal "I'm ready" signal
        let _ = ctx.emit("ready", &json!({
            "event": "ready",
            "ts": now_ts(),
        })).await;
    }

    async fn on_message(&self, ctx: &WsContext, msg: Self::In) {
        match msg {
            SubmissionIncoming::Ping => {
                // reply to the sender only with a tiny pong
                let _ = ctx
                    .reply_text(
                        json!({
                            "event": "pong",
                            "ts": now_ts(),
                        })
                        .to_string(),
                    )
                    .await;
            }
        }
    }

    async fn on_close(&self, _ctx: &WsContext) {
        // no-op
    }
}
