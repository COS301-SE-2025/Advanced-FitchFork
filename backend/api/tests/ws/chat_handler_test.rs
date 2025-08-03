#[cfg(test)]
mod tests {
    use crate::helpers::{connect_ws, make_test_app, spawn_server};
    use api::auth::generate_jwt;
    use db::models::user::Model as UserModel;
    use tokio_tungstenite::connect_async;
    use tokio_tungstenite::tungstenite::client::IntoClientRequest;
    use tokio_tungstenite::{tungstenite::{Error}};
    use tokio_tungstenite::tungstenite::protocol::Message;
    use serde_json::json;
    use futures_util::sink::SinkExt;
    use futures_util::stream::StreamExt;

    pub struct TestData {
        pub user1: UserModel,
        pub user2: UserModel,
    }

    pub async fn setup_test_data(db: &sea_orm::DatabaseConnection) -> TestData {
        let user1 = UserModel::create(db, "chat1", "chat1@test.com", "password123", false).await.unwrap();
        let user2 = UserModel::create(db, "chat2", "chat2@test.com", "password123", false).await.unwrap();
        TestData {
            user1,
            user2,
        }
    }

    #[tokio::test]
    async fn authenticated_user_can_connect_to_chat() {
        let (app, state) = make_test_app().await;
        let data = setup_test_data(state.db()).await;
        let addr = spawn_server(app).await;
        let (token, _) = generate_jwt(data.user1.id, data.user1.admin);

        let (mut ws, _) = connect_ws(&addr.to_string(), "chat", &token).await.unwrap();
        ws.close(None).await.unwrap();
    }

    #[tokio::test]
    async fn unauthenticated_user_cannot_connect_to_chat() {
        let (app, _) = make_test_app().await;
        let addr = spawn_server(app).await;
        let url = format!("ws://{}/ws/chat", addr);

        let req = url.clone().into_client_request().unwrap();
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
    async fn users_can_exchange_chat_messages() {
        let (app, state) = make_test_app().await;
        let data = setup_test_data(state.db()).await;
        let addr = spawn_server(app).await;

        let (token1, _) = generate_jwt(data.user1.id, data.user1.admin);
        let (token2, _) = generate_jwt(data.user2.id, data.user2.admin);

        let (mut ws1, _) = connect_ws(&addr.to_string(), "chat", &token1).await.unwrap();
        let (mut ws2, _) = connect_ws(&addr.to_string(), "chat", &token2).await.unwrap();

        let msg = json!({
            "type": "chat",
            "content": "Hello from user1",
            "sender": "chat1"
        });

        ws1.send(Message::Text(msg.to_string().into())).await.unwrap();

        if let Some(Ok(Message::Text(received))) = ws2.next().await {
            let parsed: serde_json::Value = serde_json::from_str(&received).unwrap();
            assert_eq!(parsed["event"], "chat");
            assert_eq!(parsed["payload"]["content"], "Hello from user1");
            assert_eq!(parsed["payload"]["sender"], "chat1");
        } else {
            panic!("Expected text message on ws2");
        }

        ws1.close(None).await.unwrap();
        ws2.close(None).await.unwrap();
    }

    #[tokio::test]
    async fn server_responds_to_ping_with_pong() {
        let (app, state) = make_test_app().await;
        let data = setup_test_data(state.db()).await;
        let addr = spawn_server(app).await;
        let (token, _) = generate_jwt(data.user1.id, data.user1.admin);
        let (mut ws, _) = connect_ws(&addr.to_string(), "chat", &token).await.unwrap();

        let ping = json!({ "type": "ping" });
        ws.send(Message::Text(ping.to_string().into())).await.unwrap();

        if let Some(Ok(Message::Text(received))) = ws.next().await {
            let parsed: serde_json::Value = serde_json::from_str(&received).unwrap();
            assert_eq!(parsed["event"], "pong");
        } else {
            panic!("Expected pong response");
        }

        ws.close(None).await.unwrap();
    }
}
