use axum::{
    extract::{State, WebSocketUpgrade},
    response::IntoResponse,
};
use axum::extract::ws::{Message, WebSocket};
use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::state::AppState;

pub async fn default_websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    // Fixed topic used internally, but clients don’t see it or set it
    let topic = "__default".to_string();
    let manager = state.ws_clone();

    ws.on_upgrade(move |socket: WebSocket| async move {
        let mut rx = manager.subscribe(&topic).await;
        let manager_clone = manager.clone();

        let (socket_tx_raw, mut socket_rx) = socket.split();
        let socket_tx = Arc::new(Mutex::new(socket_tx_raw));

        // Server → Client
        let forward_socket_tx = Arc::clone(&socket_tx);
        let forward_task = tokio::spawn(async move {
            while let Ok(msg) = rx.recv().await {
                let mut tx = forward_socket_tx.lock().await;
                if tx.send(Message::Text(msg.into())).await.is_err() {
                    tracing::info!("Client disconnected from default topic");
                    break;
                }
            }
        });

        // Client → Server
        let receive_socket_tx = Arc::clone(&socket_tx);
        let receive_task = tokio::spawn(async move {
            while let Some(Ok(msg)) = socket_rx.next().await {
                match msg {
                    Message::Text(txt) => {
                        manager_clone.broadcast(&topic, txt.to_string()).await;
                    }
                    Message::Binary(_) => {
                        tracing::warn!("Binary messages not supported on default handler");
                    }
                    Message::Ping(payload) => {
                        let mut tx = receive_socket_tx.lock().await;
                        if tx.send(Message::Pong(payload)).await.is_err() {
                            break;
                        }
                    }
                    Message::Close(_) => {
                        tracing::info!("Client closed connection on default topic");
                        break;
                    }
                    _ => {}
                }
            }
        });

        // Periodic Ping
        let ping_socket_tx = Arc::clone(&socket_tx);
        let ping_task = tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                let mut tx = ping_socket_tx.lock().await;
                if tx.send(Message::Ping(Bytes::new())).await.is_err() {
                    break;
                }
            }
        });

        let _ = tokio::join!(forward_task, receive_task, ping_task);
        tracing::info!("Default WebSocket session fully closed");
    })
}
