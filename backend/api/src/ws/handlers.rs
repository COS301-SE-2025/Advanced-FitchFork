//! WebSocket handler for the global `chat` topic.
//!
//! This handler supports basic real-time chat functionality with typed JSON messages.
//! It handles four types of incoming messages (`chat`, `typing`, `ping`, and `join`) and
//! broadcasts them to all connected clients using a shared `WebSocketManager`.

use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use util::state::AppState;

/// Incoming WebSocket message types sent by clients.
///
/// This enum is deserialized from the client-provided JSON with a `type` tag.
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ChatMessage {
    /// A chat message with content and sender.
    Chat { content: String, sender: String },
    /// Join event, used to announce that a user has connected.
    Join { sender: String },
    /// Ping message sent by the client to keep the connection alive.
    Ping,
    /// Indicates the user is typing.
    Typing { sender: String },
}

/// Outgoing event wrapper sent to connected WebSocket clients.
///
/// These are broadcasted to all subscribers of the topic.
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
struct ChatEvent {
    /// The event type (e.g., `chat`, `join`, `typing`, etc.)
    event: String,
    /// The payload of the event.
    payload: ChatPayload,
}

/// Inner payload for outgoing chat events.
#[derive(Debug, Serialize)]
struct ChatPayload {
    /// The sender of the message.
    sender: String,
    /// The message content (can be empty for non-chat events).
    content: String,
}

/// Axum handler for WebSocket connections to the global `chat` topic.
///
/// Accepts the WebSocket upgrade, splits the socket into sender/receiver halves, and spawns
/// two concurrent tasks:
///
/// - One that forwards all broadcasted messages to the client.
/// - One that receives incoming client messages, deserializes them, and rebroadcasts appropriately.
///
/// # Arguments
///
/// * `ws` - WebSocket upgrade extractor.
/// * `state` - Shared application state (`AppState`) containing the WebSocket manager.
///
/// # Returns
///
/// An Axum `IntoResponse` that performs the WebSocket upgrade.
pub async fn chat_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let topic = "chat".to_string();
    let manager = state.ws_clone();

    ws.on_upgrade(move |socket: WebSocket| async move {
        let mut rx = manager.subscribe(&topic).await;
        let manager_clone = manager.clone();
        let (socket_tx_raw, mut socket_rx) = socket.split();
        let socket_tx = Arc::new(Mutex::new(socket_tx_raw));

        // Server → Client: forward broadcasted messages to this client
        let forward_socket_tx = Arc::clone(&socket_tx);
        let forward_task = tokio::spawn({
            let topic = topic.clone();
            async move {
                while let Ok(msg) = rx.recv().await {
                    let mut tx = forward_socket_tx.lock().await;
                    if tx.send(Message::Text(msg.into())).await.is_err() {
                        tracing::info!("Client disconnected while sending to topic '{topic}'");
                        break;
                    }
                }
            }
        });

        // Client → Server: process and broadcast client messages
        let receive_socket_tx = Arc::clone(&socket_tx);
        let receive_task = tokio::spawn({
            let topic = topic.clone();
            async move {
                while let Some(Ok(msg)) = socket_rx.next().await {
                    match msg {
                        Message::Text(text) => {
                            match serde_json::from_str::<ChatMessage>(&text) {
                                Ok(ChatMessage::Chat { content, sender }) => {
                                    let event = ChatEvent {
                                        event: "chat".into(),
                                        payload: ChatPayload { sender, content },
                                    };
                                    let serialized = serde_json::to_string(&event).unwrap();
                                    manager_clone.broadcast(&topic, serialized).await;
                                }
                                Ok(ChatMessage::Typing { sender }) => {
                                    let event = ChatEvent {
                                        event: "typing".into(),
                                        payload: ChatPayload {
                                            sender,
                                            content: "".into(),
                                        },
                                    };
                                    let serialized = serde_json::to_string(&event).unwrap();
                                    manager_clone.broadcast(&topic, serialized).await;
                                }
                                Ok(ChatMessage::Ping) => {
                                    let pong = ChatEvent {
                                        event: "pong".into(),
                                        payload: ChatPayload {
                                            sender: "system".into(),
                                            content: "".into(),
                                        },
                                    };
                                    let serialized = serde_json::to_string(&pong).unwrap();
                                    manager_clone.broadcast(&topic, serialized).await;
                                }
                                Ok(ChatMessage::Join { sender }) => {
                                    tracing::info!("Client '{sender}' joined chat topic '{}'", topic);
                                    let join_msg = ChatEvent {
                                        event: "join".into(),
                                        payload: ChatPayload {
                                            sender,
                                            content: "joined the chat".into(),
                                        },
                                    };
                                    let serialized = serde_json::to_string(&join_msg).unwrap();
                                    manager_clone.broadcast(&topic, serialized).await;
                                }
                                Err(e) => {
                                    tracing::warn!("Invalid chat message: {e}");
                                }
                            }
                        }
                        Message::Ping(payload) => {
                            let mut tx = receive_socket_tx.lock().await;
                            if tx.send(Message::Pong(payload)).await.is_err() {
                                break;
                            }
                        }
                        Message::Close(_) => {
                            tracing::info!("Client closed connection on topic '{}'", topic);
                            break;
                        }
                        _ => {} // Ignore binary and other message types
                    }
                }
            }
        });

        // Wait for both tasks to finish
        let _ = tokio::join!(forward_task, receive_task);
        tracing::info!("WebSocket session ended for topic '{}'", topic);
    })
}
