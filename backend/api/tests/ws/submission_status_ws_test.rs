#[cfg(test)]
mod tests {
    use crate::helpers::app::make_test_app_with_storage;
    use crate::helpers::{connect_ws, spawn_server};

    use api::auth::generate_jwt;
    use chrono::Utc;
    use db::models::{
        assignment::{AssignmentType, Model as AssignmentModel},
        module,
        user::Model as UserModel,
        user_module_role::{Model as UserModuleRoleModel, Role as ModuleRole},
    };
    use futures_util::{SinkExt, StreamExt};
    use sea_orm::{ActiveModelTrait, Set};
    use tokio_tungstenite::tungstenite::protocol::Message;

    // ---------- small helpers for the multiplex protocol ----------
    fn owner_topic_json(assignment_id: i64, user_id: i64) -> serde_json::Value {
        serde_json::json!({
            "kind": "assignment_submissions_owner",
            "assignment_id": assignment_id,
            "user_id": user_id
        })
    }

    fn owner_topic_path(assignment_id: i64, user_id: i64) -> String {
        format!("assignment:{assignment_id}.submissions:user:{user_id}")
    }

    fn subscribe_frame(topics: Vec<serde_json::Value>) -> String {
        serde_json::json!({
            "type": "subscribe",
            "topics": topics,
            "since": serde_json::Value::Null
        })
        .to_string()
    }

    /// seed: one module, one assignment, three users (owner / other / admin)
    async fn seed(
        db: &sea_orm::DatabaseConnection,
    ) -> (
        module::Model,
        db::models::assignment::Model,
        UserModel,
        UserModel,
        UserModel,
    ) {
        let owner = UserModel::create(db, "u-owner", "owner@test.com", "pw", false)
            .await
            .unwrap();
        let other = UserModel::create(db, "u-other", "other@test.com", "pw", false)
            .await
            .unwrap();
        let admin = UserModel::create(db, "u-admin", "admin@test.com", "pw", true)
            .await
            .unwrap();

        let m = module::ActiveModel {
            code: Set("COS999".into()),
            year: Set(2025),
            description: Set(Some("WS Guard Test".into())),
            credits: Set(16),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            ..Default::default()
        }
        .insert(db)
        .await
        .unwrap();

        UserModuleRoleModel::assign_user_to_module(db, owner.id, m.id, ModuleRole::Student)
            .await
            .unwrap();

        let a = AssignmentModel::create(
            db,
            m.id,
            "Submission WS Guard",
            Some("guard tests"),
            AssignmentType::Practical,
            Utc::now(),
            Utc::now(),
        )
        .await
        .unwrap();

        (m, a, owner, other, admin)
    }

    #[tokio::test]
    async fn owner_can_connect_and_ping() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let addr = spawn_server(app).await;
        let (_, a, owner, _other, _admin) = seed(app_state.db()).await;

        let (token, _) = generate_jwt(owner.id, owner.admin);

        // 1) connect to /ws
        let (mut ws, _) = connect_ws(&addr.to_string(), Some(&token))
            .await
            .expect("connect");

        // 2) read initial ready
        if let Some(Ok(Message::Text(txt))) = ws.next().await {
            let v: serde_json::Value = serde_json::from_str(&txt).unwrap();
            assert_eq!(v["type"], "ready");
        } else {
            panic!("expected initial ready");
        }

        // 3) subscribe to the owner's stream
        let sub = subscribe_frame(vec![owner_topic_json(a.id, owner.id)]);
        ws.send(Message::Text(sub.into())).await.unwrap();

        // 4) expect subscribe_ok with accepted path
        let ok = ws.next().await.expect("subscribe_ok msg").expect("ok");
        let Message::Text(txt) = ok else {
            panic!("expected text");
        };
        let v: serde_json::Value = serde_json::from_str(&txt).unwrap();
        assert_eq!(v["type"], "subscribe_ok");
        let accepted = v["accepted"].as_array().cloned().unwrap_or_default();
        let path = owner_topic_path(a.id, owner.id);
        assert!(
            accepted.iter().any(|x| x.as_str() == Some(&path)),
            "owner path not accepted"
        );

        // 5) ping → pong
        let ping = serde_json::json!({ "type": "ping" }).to_string();
        ws.send(Message::Text(ping.into())).await.unwrap();

        loop {
            match ws.next().await {
                Some(Ok(Message::Text(txt))) => {
                    let v: serde_json::Value = serde_json::from_str(&txt).unwrap();
                    if v["type"] == "pong" {
                        break;
                    }
                }
                other => panic!("expected pong, got {:?}", other),
            }
        }

        ws.close(None).await.unwrap();
    }

    #[tokio::test]
    async fn admin_subscription_is_rejected_not_owner() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let addr = spawn_server(app).await;
        let (_, a, owner, _other, admin) = seed(app_state.db()).await;

        let (token, _) = generate_jwt(admin.id, admin.admin);

        let (mut ws, _) = connect_ws(&addr.to_string(), Some(&token))
            .await
            .expect("connect");
        // consume ready
        let _ = ws.next().await;

        let sub = subscribe_frame(vec![owner_topic_json(a.id, owner.id)]);
        ws.send(Message::Text(sub.into())).await.unwrap();

        let msg = ws.next().await.expect("message").expect("ok");
        let Message::Text(txt) = msg else {
            panic!("expected text");
        };
        let v: serde_json::Value = serde_json::from_str(&txt).unwrap();
        assert_eq!(v["type"], "subscribe_ok");
        let rejected = v["rejected"].as_array().cloned().unwrap_or_default();
        let path = owner_topic_path(a.id, owner.id);

        // rejected is array of [path, code]
        let has_not_owner = rejected.iter().any(|r| {
            r.get(0).and_then(|s| s.as_str()) == Some(&path)
                && r.get(1).and_then(|s| s.as_str()) == Some("not_owner")
        });
        assert!(has_not_owner, "expected not_owner rejection");

        ws.close(None).await.unwrap();
    }

    #[tokio::test]
    async fn lecturer_subscription_is_rejected_not_owner() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let db = app_state.db();
        let addr = spawn_server(app).await;
        let (m, a, owner, lecturer, _admin) = seed(db).await;

        // give lecturer role (still should be rejected to owner stream)
        UserModuleRoleModel::assign_user_to_module(db, lecturer.id, m.id, ModuleRole::Lecturer)
            .await
            .unwrap();

        let (token, _) = generate_jwt(lecturer.id, lecturer.admin);
        let (mut ws, _) = connect_ws(&addr.to_string(), Some(&token))
            .await
            .expect("connect");
        // consume ready
        let _ = ws.next().await;

        let sub = subscribe_frame(vec![owner_topic_json(a.id, owner.id)]);
        ws.send(Message::Text(sub.into())).await.unwrap();

        let msg = ws.next().await.expect("message").expect("ok");
        let Message::Text(txt) = msg else {
            panic!("expected text");
        };
        let v: serde_json::Value = serde_json::from_str(&txt).unwrap();
        assert_eq!(v["type"], "subscribe_ok");
        let rejected = v["rejected"].as_array().cloned().unwrap_or_default();
        let path = owner_topic_path(a.id, owner.id);
        let has_not_owner = rejected.iter().any(|r| {
            r.get(0).and_then(|s| s.as_str()) == Some(&path)
                && r.get(1).and_then(|s| s.as_str()) == Some("not_owner")
        });
        assert!(has_not_owner, "expected not_owner rejection");

        ws.close(None).await.unwrap();
    }

    #[tokio::test]
    async fn unauthenticated_is_401_on_upgrade() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let addr = spawn_server(app).await;
        let (_m, _a, _owner, _other, _admin) = seed(app_state.db()).await;

        // No token → guard rejects at upgrade with 401
        match connect_ws(&addr.to_string(), None).await {
            Ok(_) => panic!("unauthenticated user should not connect"),
            Err(tokio_tungstenite::tungstenite::Error::Http(resp)) => {
                assert_eq!(resp.status(), 401);
                let body = std::str::from_utf8(resp.body().as_ref().unwrap()).unwrap();
                let json: serde_json::Value = serde_json::from_str(body).unwrap();
                assert_eq!(json["message"], "Authentication required");
            }
            Err(e) => panic!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn assignment_not_found_rejected() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let db = app_state.db();
        let addr = spawn_server(app).await;
        let (_m1, a1, owner, _other, _admin) = seed(db).await;

        // create mismatched module m2 (assignment a1 is not in m2)
        let _m2 = module::ActiveModel {
            code: Set("COS000".into()),
            year: Set(2025),
            description: Set(Some("Other Module".into())),
            credits: Set(16),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            ..Default::default()
        }
        .insert(db)
        .await
        .unwrap();

        let (token, _) = generate_jwt(owner.id, owner.admin);
        let (mut ws, _) = connect_ws(&addr.to_string(), Some(&token))
            .await
            .expect("connect");
        // consume ready
        let _ = ws.next().await;

        // even though old router used module path, new protocol only uses assignment_id+user_id.
        // To simulate "not found in module", we can subscribe to an *unknown* assignment id.
        // Use a1.id + large offset to guarantee it doesn't exist.
        let unknown_assignment = a1.id + 1_000_000;
        let sub = subscribe_frame(vec![owner_topic_json(unknown_assignment, owner.id)]);
        ws.send(Message::Text(sub.into())).await.unwrap();

        let msg = ws.next().await.expect("message").expect("ok");
        let Message::Text(txt) = msg else {
            panic!("expected text");
        };
        let v: serde_json::Value = serde_json::from_str(&txt).unwrap();
        assert_eq!(v["type"], "subscribe_ok");
        let rejected = v["rejected"].as_array().cloned().unwrap_or_default();
        let path = owner_topic_path(unknown_assignment, owner.id);
        let has_nf = rejected.iter().any(|r| {
            r.get(0).and_then(|s| s.as_str()) == Some(&path)
                && r.get(1).and_then(|s| s.as_str()) == Some("assignment_not_found")
        });
        assert!(has_nf, "expected assignment_not_found rejection");

        ws.close(None).await.unwrap();
    }

    #[tokio::test]
    async fn malformed_subscribe_yields_bad_request_error() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let addr = spawn_server(app).await;
        let db = app_state.db();

        // make any user for auth
        let u = UserModel::create(db, "u-any", "any@test.com", "pw", false)
            .await
            .unwrap();
        let (token, _) = generate_jwt(u.id, u.admin);

        let (mut ws, _) = connect_ws(&addr.to_string(), Some(&token))
            .await
            .expect("connect");
        // consume ready
        let _ = ws.next().await;

        // send a malformed frame (wrong shape for topics)
        let bad = serde_json::json!({
            "type": "subscribe",
            "topics": [{"kind":"assignment_submissions_owner","assignment_id":"oops","user_id":"nope"}],
            "since": null
        }).to_string();

        ws.send(Message::Text(bad.into())).await.unwrap();

        // expect an error frame (not an HTTP status)
        let msg = ws.next().await.expect("message").expect("ok");
        let Message::Text(txt) = msg else {
            panic!("expected text");
        };
        let v: serde_json::Value = serde_json::from_str(&txt).unwrap();
        assert_eq!(v["type"], "error");
        assert_eq!(v["code"], "bad_request");

        ws.close(None).await.unwrap();
    }
}
