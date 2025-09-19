use crate::ws::WebSocketManager;
use axum::extract::ws::{Message, Utf8Bytes};
use serde::Serialize;
use tokio::sync::mpsc;

pub struct WsContext {
    pub topic: String,
    pub ws: WebSocketManager,
    // enqueue frames for the writer task
    out_tx: mpsc::Sender<Message>,
}

impl WsContext {
    pub fn new(topic: String, ws: WebSocketManager, out_tx: mpsc::Sender<Message>) -> Self {
        Self { topic, ws, out_tx }
    }

    /// Send a *single* text frame to this client only
    pub async fn reply_text(&self, text: impl Into<Utf8Bytes>) -> Result<(), ()> {
        self.out_tx
            .send(Message::Text(text.into()))
            .await
            .map_err(|_| ())
    }

    /// Send a WS-level pong to this client
    pub async fn reply_pong(&self, payload: bytes::Bytes) -> Result<(), ()> {
        self.out_tx
            .send(Message::Pong(payload))
            .await
            .map_err(|_| ())
    }

    /// Send any raw WS frame to this client
    pub async fn send(&self, msg: Message) -> Result<(), ()> {
        self.out_tx.send(msg).await.map_err(|_| ())
    }

    /// Broadcast a JSON envelope on this topic
    pub async fn emit<T: Serialize>(&self, event: &str, payload: &T) {
        crate::ws::emit(&self.ws, &self.topic, event, payload).await;
    }

    /// Broadcast raw text on this topic
    pub async fn broadcast_text(&self, text: impl Into<String>) {
        self.ws.broadcast(&self.topic, text).await;
    }
}
