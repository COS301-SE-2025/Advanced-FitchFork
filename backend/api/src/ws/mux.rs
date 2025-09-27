//! Single-socket multiplexer (structured topics only).

use axum::{
    Extension,
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, Utf8Bytes, WebSocket},
    },
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use std::collections::{HashMap, HashSet};
use util::state::AppState;

use super::{
    auth::authorize_topic,
    types::{WsIn, WsOut},
};
use crate::{auth::claims::AuthUser, ws::auth::TopicAuth};

pub async fn ws_multiplex_entry(
    ws: WebSocketUpgrade,
    State(app): State<AppState>,
    Extension(user): Extension<AuthUser>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| serve(socket, app, user))
}

struct Subs {
    pumps: HashMap<String, tokio::task::JoinHandle<()>>,
}
impl Subs {
    fn new() -> Self {
        Self {
            pumps: HashMap::new(),
        }
    }
}

async fn serve(socket: WebSocket, app: AppState, user: AuthUser) {
    let (mut sink, mut rx) = socket.split();

    let (tx_out, mut rx_out) = tokio::sync::mpsc::channel::<Message>(256);
    let writer = tokio::spawn(async move {
        while let Some(m) = rx_out.recv().await {
            if sink.send(m).await.is_err() {
                break;
            }
        }
    });

    // READY
    let ready = serde_json::to_string(&WsOut::<()>::Ready {
        policy_version: 1,
        exp: None,
    })
    .unwrap();
    let _ = tx_out.send(Message::Text(Utf8Bytes::from(ready))).await;

    let ws = app.ws_clone();
    let db = app.db_clone();

    let mut subs = Subs::new();
    let mut presence_paths: HashSet<String> = HashSet::new();

    let spawn_pump = |path: String| {
        let ws = ws.clone();
        let tx_out = tx_out.clone();
        async move {
            let mut brx = ws.subscribe(&path).await;
            tokio::spawn(async move {
                while let Ok(text) = brx.recv().await {
                    // manager broadcasts String; convert to Utf8Bytes
                    if tx_out.send(Message::Text(text.into())).await.is_err() {
                        break;
                    }
                }
            })
        }
    };

    while let Some(Ok(frame)) = rx.next().await {
        match frame {
            Message::Text(txt) => {
                let parsed = serde_json::from_str::<WsIn>(txt.as_str());
                match parsed {
                    Ok(WsIn::Ping) => {
                        let pong = serde_json::to_string(&WsOut::<()>::Pong).unwrap();
                        let _ = tx_out.send(Message::Text(pong.into())).await;
                    }

                    Ok(WsIn::Subscribe { topics, .. }) => {
                        let mut accepted = Vec::new();
                        let mut rejected = Vec::new();

                        for ct in topics {
                            let auth = authorize_topic(&db, &user, &ct).await;
                            let path = ct.path();

                            match auth {
                                TopicAuth::Allowed => {
                                    if !subs.pumps.contains_key(&path) {
                                        let pump = spawn_pump(path.clone()).await;
                                        subs.pumps.insert(path.clone(), pump);
                                        app.ws().register(&path, user.0.sub).await;
                                        presence_paths.insert(path.clone());
                                    }
                                    accepted.push(path);
                                }
                                TopicAuth::Denied(code) => {
                                    rejected.push((path, code.to_string()));
                                    continue;
                                }
                            }
                        }

                        let msg =
                            serde_json::to_string(&WsOut::<()>::SubscribeOk { accepted, rejected })
                                .unwrap();
                        let _ = tx_out.send(Message::Text(msg.into())).await;
                    }

                    Ok(WsIn::Unsubscribe { topics }) => {
                        let paths: Vec<String> = topics.into_iter().map(|t| t.path()).collect();

                        for p in paths.iter() {
                            if let Some(h) = subs.pumps.remove(p) {
                                h.abort();
                            }
                            if presence_paths.remove(p) {
                                app.ws().unregister(p, user.0.sub).await;
                            }
                        }
                        let msg =
                            serde_json::to_string(&WsOut::<()>::UnsubscribeOk { topics: paths })
                                .unwrap();
                        let _ = tx_out.send(Message::Text(msg.into())).await;
                    }

                    Ok(WsIn::Command { .. }) => {
                        let msg = serde_json::to_string(&WsOut::<()>::Error {
                            code: "not_implemented",
                            message: "WS commands are not implemented".into(),
                            meta: None,
                        })
                        .unwrap();
                        let _ = tx_out.send(Message::Text(msg.into())).await;
                    }

                    Ok(WsIn::Auth { .. }) | Ok(WsIn::Reauth { .. }) => {
                        let msg = serde_json::to_string(&WsOut::<()>::Error {
                            code: "bad_request",
                            message: "Auth is handled by HTTP guard".into(),
                            meta: None,
                        })
                        .unwrap();
                        let _ = tx_out.send(Message::Text(msg.into())).await;
                    }

                    Err(e) => {
                        let msg = serde_json::to_string(&WsOut::<()>::Error {
                            code: "bad_request",
                            message: format!("invalid frame: {e}"),
                            meta: None,
                        })
                        .unwrap();
                        let _ = tx_out.send(Message::Text(msg.into())).await;
                    }
                }
            }

            Message::Ping(b) => {
                let _ = tx_out.send(Message::Pong(b)).await;
            }
            Message::Close(_) => break,
            Message::Pong(_) | Message::Binary(_) => {}
        }
    }

    // cleanup
    for (p, h) in subs.pumps.into_iter() {
        h.abort();
        if presence_paths.contains(&p) {
            app.ws().unregister(&p, user.0.sub).await;
        }
    }
    let _ = writer.await;
}
