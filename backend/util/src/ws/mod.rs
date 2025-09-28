// util/src/ws/mod.rs
pub mod manager;
pub use manager::WebSocketManager;

use chrono::Utc;
use serde::Serialize;

/// Standard event envelope sent over WebSocket topics.
#[derive(Serialize)]
pub struct EventEnvelope<'a, T> {
    #[serde(rename = "type")]
    pub r#type: &'static str,
    pub event: &'a str,
    pub topic: &'a str,
    pub payload: T,
    pub ts: String,
}

/// Broadcast a JSON-serialized `EventEnvelope` on `topic`.
pub async fn emit<T: Serialize>(ws: &WebSocketManager, topic: &str, event: &str, payload: &T) {
    let env = EventEnvelope {
        r#type: "event",
        event,
        topic,
        payload,
        ts: Utc::now().to_rfc3339(),
    };
    if let Ok(json) = serde_json::to_string(&env) {
        ws.broadcast(topic, json).await;
    }
}
