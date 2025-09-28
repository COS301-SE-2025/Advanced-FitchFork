#[cfg(test)]
mod tests {
    use crate::helpers::app::make_test_app_with_storage;
    use crate::helpers::{make_test_app, spawn_server};
    use api::auth::generate_jwt;
    use db::models::user::Model as UserModel;
    use futures_util::{sink::SinkExt, stream::StreamExt};
    use serde_json::json;
    use tokio_tungstenite::connect_async;
    use tokio_tungstenite::tungstenite::Error;
    use tokio_tungstenite::tungstenite::client::IntoClientRequest;
    use tokio_tungstenite::tungstenite::http::HeaderValue;
    use tokio_tungstenite::tungstenite::http::header::AUTHORIZATION;
    use tokio_tungstenite::tungstenite::protocol::Message;

    pub struct TestData {
        pub user1: UserModel,
        pub user2: UserModel,
    }

    pub async fn setup_test_data(db: &sea_orm::DatabaseConnection) -> TestData {
        let user1 = UserModel::create(db, "chat1", "chat1@test.com", "password123", false)
            .await
            .unwrap();
        let user2 = UserModel::create(db, "chat2", "chat2@test.com", "password123", false)
            .await
            .unwrap();
        TestData { user1, user2 }
    }

    /// Build a websocket upgrade request to `/ws` and (optionally) add Authorization header.
    fn ws_request(
        addr: &str,
        bearer_token: Option<&str>,
    ) -> tokio_tungstenite::tungstenite::http::Request<()> {
        let url = format!("ws://{}/ws", addr);
        let mut req = url.into_client_request().unwrap();
        if let Some(tok) = bearer_token {
            let hv = HeaderValue::from_str(&format!("Bearer {}", tok)).unwrap();
            req.headers_mut().insert(AUTHORIZATION, hv);
        }
        req
    }

    #[tokio::test]
    async fn authenticated_user_can_connect_to_ws_mux() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;
        let addr = spawn_server(app).await;

        let (token, _) = generate_jwt(data.user1.id, data.user1.admin);
        let req = ws_request(&addr.to_string(), Some(&token));

        let (mut ws, _) = connect_async(req).await.unwrap();

        // Expect initial READY frame
        if let Some(Ok(Message::Text(txt))) = ws.next().await {
            let v: serde_json::Value = serde_json::from_str(&txt).unwrap();
            assert_eq!(v["type"], "ready");
            assert!(v.get("policy_version").is_some());
        } else {
            panic!("expected ready frame");
        }

        ws.close(None).await.unwrap();
    }

    #[tokio::test]
    async fn unauthenticated_user_cannot_connect_to_ws_mux() {
        let (app, _) = make_test_app().await;
        let addr = spawn_server(app).await;

        let req = ws_request(&addr.to_string(), None);
        let result = connect_async(req).await;

        match result {
            Ok(_) => panic!("Unauthenticated user should not connect"),
            Err(Error::Http(resp)) => {
                assert_eq!(resp.status(), 401);
                let body = std::str::from_utf8(resp.body().as_ref().unwrap()).unwrap();
                let json: serde_json::Value = serde_json::from_str(body).unwrap();
                assert_eq!(json["message"], "Authentication required");
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn subscribers_receive_broadcast_on_system_topic() {
        use chrono::Utc;
        use tokio::time::{Duration, timeout};

        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;
        let addr = spawn_server(app).await;

        let (token1, _) = generate_jwt(data.user1.id, data.user1.admin);
        let (token2, _) = generate_jwt(data.user2.id, data.user2.admin);

        let req1 = ws_request(&addr.to_string(), Some(&token1));
        let req2 = ws_request(&addr.to_string(), Some(&token2));

        let (mut ws1, _) = connect_async(req1).await.unwrap();
        let (mut ws2, _) = connect_async(req2).await.unwrap();

        // consume READY frames
        let _ = ws1.next().await;
        let _ = ws2.next().await;

        // subscribe both to "system"
        let sub = json!({ "type":"subscribe", "topics":[{"kind":"system"}] });
        ws1.send(Message::Text(sub.to_string().into()))
            .await
            .unwrap();
        ws2.send(Message::Text(sub.to_string().into()))
            .await
            .unwrap();

        // expect subscribe_ok for each
        let _ = ws1.next().await;
        let _ = ws2.next().await;

        // Emit a synthetic event onto the "system" topic using the manager
        let topic = "system".to_string();
        let payload = json!({ "content": "Hello from user1", "sender": "chat1" });
        let event = json!({
            "type": "event",
            "event": "system.health",
            "topic": topic,
            "payload": payload,
            "ts": Utc::now().to_rfc3339(),
            "v": null
        })
        .to_string();

        app_state.ws().broadcast(&topic, event.clone()).await;

        // ws2 should receive the event
        let msg = timeout(Duration::from_millis(500), ws2.next())
            .await
            .expect("not timed out")
            .expect("stream closed")
            .expect("ws error");

        let Message::Text(txt) = msg else {
            panic!("expected text");
        };
        let v: serde_json::Value = serde_json::from_str(&txt).unwrap();
        assert_eq!(v["type"], "event");
        assert_eq!(v["event"], "system.health");
        assert_eq!(v["topic"], "system");
        assert_eq!(v["payload"]["content"], "Hello from user1");
        assert_eq!(v["payload"]["sender"], "chat1");

        ws1.close(None).await.unwrap();
        ws2.close(None).await.unwrap();
    }

    #[tokio::test]
    async fn server_responds_to_ping_with_pong() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;
        let addr = spawn_server(app).await;

        let (token, _) = generate_jwt(data.user1.id, data.user1.admin);
        let req = ws_request(&addr.to_string(), Some(&token));
        let (mut ws, _) = connect_async(req).await.unwrap();

        // consume READY
        let _ = ws.next().await;

        ws.send(Message::Text(json!({"type":"ping"}).to_string().into()))
            .await
            .unwrap();

        if let Some(Ok(Message::Text(received))) = ws.next().await {
            let parsed: serde_json::Value = serde_json::from_str(&received).unwrap();
            assert_eq!(parsed["type"], "pong");
        } else {
            panic!("Expected pong response");
        }

        ws.close(None).await.unwrap();
    }
}
