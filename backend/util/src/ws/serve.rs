use axum::extract::ws::{Message, WebSocket};
use bytes::Bytes;
use chrono::Utc;
use futures::{SinkExt, StreamExt};
use serde_json::Value;
use std::sync::Arc;
use tokio::{sync::mpsc, time};

use super::WebSocketManager;
use super::handler_trait::WsHandler;
use super::runtime::WsContext;

pub struct WsServerOptions {
    pub ws_ping_sec: u64,
    pub enable_app_ping: bool,
    pub _unused: (),
}

impl Default for WsServerOptions {
    fn default() -> Self {
        Self {
            ws_ping_sec: 30,
            enable_app_ping: true,
            _unused: (),
        }
    }
}

pub async fn serve_topic<H: WsHandler>(
    socket: WebSocket,
    manager: WebSocketManager,
    topic: String,
    user_id: Option<i64>,
    handler: Arc<H>,
    opts: WsServerOptions,
) {
    let mut rx = manager.subscribe(&topic).await;
    if let Some(uid) = user_id {
        manager.register(&topic, uid).await;
    }

    // Split the socket according to *your* axum version.
    // If your version has inherent split:
    let (mut sink, mut socket_rx) = axum::extract::ws::WebSocket::split(socket);
    // If you need the trait split instead, the rest still works because we write via Sink<Message>.

    // Outbound queue and writer task
    let (out_tx, mut out_rx) = mpsc::channel::<Message>(64);
    let writer_task = tokio::spawn(async move {
        while let Some(frame) = out_rx.recv().await {
            if sink.send(frame).await.is_err() {
                break;
            }
        }
    });

    let ctx = WsContext::new(topic.clone(), manager.clone(), out_tx.clone());

    // S→C: forward broadcasts on this topic
    let forward_task = {
        let out_tx = out_tx.clone();
        let topic = topic.clone();
        tokio::spawn(async move {
            while let Ok(msg) = rx.recv().await {
                // `msg` is String; Text wants Utf8Bytes but `.into()` handles it
                if out_tx.send(Message::Text(msg.into())).await.is_err() {
                    tracing::info!("Client disconnected while sending to '{topic}'");
                    break;
                }
            }
        })
    };

    // WS-level periodic ping
    let ping_task = {
        let out_tx = out_tx.clone();
        tokio::spawn(async move {
            loop {
                time::sleep(std::time::Duration::from_secs(opts.ws_ping_sec)).await;
                if out_tx.send(Message::Ping(Bytes::new())).await.is_err() {
                    break;
                }
            }
        })
    };

    // Let feature handler know we're live
    handler.on_open(&ctx).await;

    // C→S: parse & dispatch
    let receive_task = {
        let handler = Arc::clone(&handler);
        let ctx = ctx;
        tokio::spawn(async move {
            while let Some(Ok(msg)) = socket_rx.next().await {
                match msg {
                    Message::Text(text) => {
                        let raw = text.as_str();
                        if opts.enable_app_ping && is_app_ping(raw) {
                            let _ = ctx
                                .reply_text(
                                    serde_json::json!({
                                        "event": "pong",
                                        "topic": ctx.topic,
                                        "payload": {},
                                        "ts": Utc::now().to_rfc3339(),
                                    })
                                    .to_string(),
                                )
                                .await;
                            continue;
                        }
                        match serde_json::from_str::<H::In>(raw) {
                            Ok(parsed) => handler.on_message(&ctx, parsed).await,
                            Err(e) => tracing::warn!(
                                "WS invalid message on '{}': {e}; raw={raw}",
                                ctx.topic
                            ),
                        }
                    }
                    Message::Ping(payload) => {
                        let _ = ctx.reply_pong(payload).await;
                    }
                    Message::Pong(_) => {}
                    Message::Binary(_) => {
                        tracing::warn!("Ignoring binary on topic '{}'", ctx.topic);
                    }
                    Message::Close(_) => {
                        handler.on_close(&ctx).await;
                        break;
                    }
                }
            }
        })
    };

    let _ = tokio::join!(forward_task, receive_task, ping_task, writer_task);
    if let Some(uid) = user_id {
        manager.unregister(&topic, uid).await;
    }
    tracing::info!("WS session ended for topic '{topic}'");
}

fn is_app_ping(raw: &str) -> bool {
    if let Ok(Value::Object(map)) = serde_json::from_str::<Value>(raw) {
        if let Some(Value::String(t)) = map.get("type") {
            return t == "ping";
        }
    }
    false
}
