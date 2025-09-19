pub mod axum_adapter;
pub mod default;
pub mod handler;
pub mod handler_trait;
pub mod manager;
pub mod runtime;
pub mod serve;

use chrono::Utc;
use serde::Serialize;

pub use handler::default_websocket_handler;
pub use manager::WebSocketManager;

/// Standard event envelope sent over WebSocket topics.
#[derive(Serialize)]
pub struct EventEnvelope<'a, T> {
    /// Semantic event name, e.g. "message_created"
    pub event: &'a str,
    /// The topic this envelope is published on (useful for muxing & client routing)
    pub topic: &'a str,
    /// The domain payload (e.g., MessageResponse)
    pub payload: T,
    /// ISO-8601 timestamp (UTC)
    pub ts: String,
}

/// Broadcast a JSON-serialized `EventEnvelope` on `topic`.
pub async fn emit<T: Serialize>(ws: &WebSocketManager, topic: &str, event: &str, payload: &T) {
    let env = EventEnvelope {
        event,
        topic,
        payload,
        ts: Utc::now().to_rfc3339(),
    };
    let json = serde_json::to_string(&env).expect("serialize event");
    ws.broadcast(topic, json).await;
}
