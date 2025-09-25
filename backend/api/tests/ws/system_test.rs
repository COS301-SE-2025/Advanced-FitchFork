#[cfg(test)]
mod tests {
    use api::auth::generate_jwt;
    use db::models::{
        module::Model as ModuleModel,
        user::Model as UserModel,
        user_module_role::{Model as UserModuleRoleModel, Role},
    };
    use futures_util::{SinkExt, StreamExt};
    use serial_test::serial;
    use std::sync::Once;
    use tokio_tungstenite::tungstenite::Error;
    use tokio_tungstenite::tungstenite::protocol::Message;

    use crate::helpers::{connect_ws, make_test_app_with_storage, spawn_server};

    fn ensure_jwt_env() {
        static INIT: Once = Once::new();
        INIT.call_once(|| {
            if std::env::var("JWT_SECRET").is_err() {
                unsafe {
                    std::env::set_var("JWT_SECRET", "test-secret");
                }
            }
            if std::env::var("JWT_DURATION_MINUTES").is_err() {
                unsafe {
                    std::env::set_var("JWT_DURATION_MINUTES", "60");
                }
            }
            if std::env::var("APP_ENV").is_err() {
                unsafe {
                    std::env::set_var("APP_ENV", "test");
                }
            }
        });
    }

    // Small helpers for the multiplex frames
    fn subscribe_frame(topics: Vec<serde_json::Value>) -> String {
        serde_json::json!({
            "type": "subscribe",
            "topics": topics,
            "since": serde_json::Value::Null
        })
        .to_string()
    }

    fn sys_topic_json() -> serde_json::Value {
        serde_json::json!({ "kind": "system" })
    }

    fn sys_admin_topic_json() -> serde_json::Value {
        serde_json::json!({ "kind": "system_admin" })
    }

    #[serial]
    #[tokio::test]
    async fn admin_can_subscribe_to_system_admin_topic() {
        ensure_jwt_env();
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let addr = spawn_server(app).await;

        let admin = UserModel::create(
            app_state.db(),
            "ws_admin",
            "ws_admin@test.com",
            "password",
            true,
        )
        .await
        .unwrap();
        let (token, _) = generate_jwt(admin.id, admin.admin);

        // Connect to /ws
        let (mut ws, _) = connect_ws(&addr.to_string(), Some(token.as_str()))
            .await
            .unwrap();

        // Expect initial "ready"
        if let Some(Ok(Message::Text(txt))) = ws.next().await {
            let v: serde_json::Value = serde_json::from_str(&txt).unwrap();
            assert_eq!(v["type"], "ready");
        } else {
            panic!("expected initial ready");
        }

        // Subscribe to system_admin
        let sub = subscribe_frame(vec![sys_admin_topic_json()]);
        ws.send(Message::Text(sub.into())).await.unwrap();

        // Should be accepted
        let msg = ws.next().await.expect("message").expect("ok");
        let Message::Text(body) = msg else {
            panic!("expected text");
        };
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(v["type"], "subscribe_ok");

        let accepted = v["accepted"].as_array().cloned().unwrap_or_default();
        // Path is "system:admin"
        assert!(
            accepted.iter().any(|x| x.as_str() == Some("system:admin")),
            "system:admin not accepted for admin"
        );

        ws.close(None).await.unwrap();
    }

    #[serial]
    #[tokio::test]
    async fn non_admin_roles_cannot_subscribe_to_system_admin_topic() {
        ensure_jwt_env();
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let addr = spawn_server(app).await;
        let db = app_state.db();

        // Create module + users with roles
        let module = ModuleModel::create(db, "MOD101", 2025, Some("Test"), 16)
            .await
            .unwrap();
        let lecturer = UserModel::create(db, "ws_lect", "ws_lect@test.com", "pw", false)
            .await
            .unwrap();
        let tutor = UserModel::create(db, "ws_tutor", "ws_tutor@test.com", "pw", false)
            .await
            .unwrap();
        let student = UserModel::create(db, "ws_stud", "ws_stud@test.com", "pw", false)
            .await
            .unwrap();

        UserModuleRoleModel::assign_user_to_module(db, lecturer.id, module.id, Role::Lecturer)
            .await
            .unwrap();
        UserModuleRoleModel::assign_user_to_module(db, tutor.id, module.id, Role::Tutor)
            .await
            .unwrap();
        UserModuleRoleModel::assign_user_to_module(db, student.id, module.id, Role::Student)
            .await
            .unwrap();

        for u in [lecturer, tutor, student] {
            let (tok, _) = generate_jwt(u.id, u.admin);

            // Connect
            let (mut ws, _) = connect_ws(&addr.to_string(), Some(tok.as_str()))
                .await
                .unwrap();
            // consume ready
            let _ = ws.next().await;

            // Subscribe to system_admin → expect rejection with code "admin_only"
            let sub = subscribe_frame(vec![sys_admin_topic_json()]);
            ws.send(Message::Text(sub.into())).await.unwrap();

            let msg = ws.next().await.expect("message").expect("ok");
            let Message::Text(body) = msg else {
                panic!("expected text");
            };
            let v: serde_json::Value = serde_json::from_str(&body).unwrap();
            assert_eq!(v["type"], "subscribe_ok");

            let rejected = v["rejected"].as_array().cloned().unwrap_or_default();
            let has_admin_only = rejected.iter().any(|r| {
                r.get(0).and_then(|s| s.as_str()) == Some("system:admin")
                    && r.get(1).and_then(|s| s.as_str()) == Some("admin_only")
            });
            assert!(has_admin_only, "expected admin_only rejection");

            ws.close(None).await.unwrap();
        }
    }

    #[serial]
    #[tokio::test]
    async fn authenticated_user_can_subscribe_to_system_topic() {
        ensure_jwt_env();
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let addr = spawn_server(app).await;

        let user = UserModel::create(
            app_state.db(),
            "ws_user",
            "ws_user@test.com",
            "password",
            false,
        )
        .await
        .unwrap();
        let (token, _) = generate_jwt(user.id, user.admin);

        // Any authenticated user
        let (mut ws, _) = connect_ws(&addr.to_string(), Some(token.as_str()))
            .await
            .unwrap();
        // consume ready
        let _ = ws.next().await;

        // Subscribe to system → must be accepted
        let sub = subscribe_frame(vec![sys_topic_json()]);
        ws.send(Message::Text(sub.into())).await.unwrap();

        let msg = ws.next().await.expect("message").expect("ok");
        let Message::Text(body) = msg else {
            panic!("expected text");
        };
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(v["type"], "subscribe_ok");

        let accepted = v["accepted"].as_array().cloned().unwrap_or_default();
        assert!(
            accepted.iter().any(|x| x.as_str() == Some("system")),
            "system not accepted for authenticated user"
        );

        ws.close(None).await.unwrap();
    }

    #[serial]
    #[tokio::test]
    async fn unauthenticated_user_cannot_connect() {
        ensure_jwt_env();
        let (app, _app_state, _tmp) = make_test_app_with_storage().await;
        let addr = spawn_server(app).await;

        // No token → HTTP 401 on upgrade
        let res = connect_ws(&addr.to_string(), None).await;
        match res {
            Ok(_) => panic!("Unauthenticated client should not connect"),
            Err(Error::Http(resp)) => assert_eq!(resp.status(), 401),
            Err(e) => panic!("Unexpected websocket error: {e:?}"),
        }
    }

    #[serial]
    #[tokio::test]
    async fn invalid_token_cannot_connect() {
        ensure_jwt_env();
        let (app, _app_state, _tmp) = make_test_app_with_storage().await;
        let addr = spawn_server(app).await;

        let res = connect_ws(&addr.to_string(), Some("not.a.valid.jwt")).await;
        match res {
            Ok(_) => panic!("Invalid token should not connect"),
            Err(Error::Http(resp)) => assert_eq!(resp.status(), 401),
            Err(e) => panic!("Unexpected websocket error: {e:?}"),
        }
    }
}
