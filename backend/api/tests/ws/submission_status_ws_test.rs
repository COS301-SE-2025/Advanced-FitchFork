#[cfg(test)]
mod tests {
    use crate::helpers::app::make_test_app_with_storage;
    use crate::helpers::{connect_ws, spawn_server};

    use api::auth::generate_jwt;
    // NOTE: don't use submission_topic(); we build the path explicitly to match the router.

    use chrono::Utc;
    use db::models::{
        assignment::{AssignmentType, Model as AssignmentModel},
        module,
        user::Model as UserModel,
        user_module_role::{Model as UserModuleRoleModel, Role as ModuleRole},
    };
    use futures_util::{SinkExt, StreamExt};
    use sea_orm::{ActiveModelTrait, Set};
    use tokio_tungstenite::tungstenite::{protocol::Message, Error};

    /// seed: one module, one assignment, three users (owner / other / admin)
    async fn seed(
        db: &sea_orm::DatabaseConnection,
    ) -> (module::Model, db::models::assignment::Model, UserModel, UserModel, UserModel) {
        let owner = UserModel::create(db, "u-owner", "owner@test.com", "pw", false).await.unwrap();
        let other = UserModel::create(db, "u-other", "other@test.com", "pw", false).await.unwrap();
        let admin = UserModel::create(db, "u-admin", "admin@test.com", "pw", true).await.unwrap();

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

    /// Helper to build the WS topic path that matches the nested router:
    fn topic_for(module_id: i64, assignment_id: i64, user_id: i64) -> String {
        format!(
            "modules/{}/assignments/{}/submissions/{}",
            module_id, assignment_id, user_id
        )
    }

    #[tokio::test]
    async fn owner_can_connect_and_ping() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let addr = spawn_server(app).await;
        let (m, a, owner, _other, _admin) = seed(app_state.db()).await;

        let (token, _) = generate_jwt(owner.id, owner.admin);
        let topic = format!(
            "modules/{}/assignments/{}/submissions/{}",
            m.id, a.id, owner.id
        );

        let (mut ws, _) =
            connect_ws(&addr.to_string(), &topic, Some(&token)).await.expect("owner connect failed");

        // Some handlers emit an initial "ready" event as soon as the socket opens.
        // If we get it, just ignore and proceed.
        if let Some(Ok(Message::Text(received))) = ws.next().await {
            let v: serde_json::Value = serde_json::from_str(&received).unwrap();
            if v["event"] != "ready" {
                // If it's already a "pong" (or something else), keep it around by asserting here.
                // Most likely it's "ready", but accept "pong" too.
                assert_eq!(v["event"], "pong", "unexpected first event: {v}");
                // if it was "pong", we're done
                ws.close(None).await.unwrap();
                return;
            }
        }

        // Now ping -> expect a subsequent pong (skip any non-pong noise just in case)
        let ping = serde_json::json!({ "type": "ping" });
        ws.send(Message::Text(ping.to_string().into())).await.unwrap();

        loop {
            match ws.next().await {
                Some(Ok(Message::Text(received))) => {
                    let v: serde_json::Value = serde_json::from_str(&received).unwrap();
                    if v["event"] == "pong" {
                        break;
                    }
                    // ignore other events (e.g., periodic heartbeats)
                }
                other => panic!("expected pong, got {:?}", other),
            }
        }

        ws.close(None).await.unwrap();
    }


    #[tokio::test]
    async fn admin_cannot_connect_to_owners_stream() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let addr = spawn_server(app).await;
        let (m, a, owner, _other, admin) = seed(app_state.db()).await;

        let (token, _) = generate_jwt(admin.id, admin.admin);
        let topic = topic_for(m.id, a.id, owner.id);

        match connect_ws(&addr.to_string(), &topic, Some(&token)).await {
            Ok(_) => panic!("admin should be forbidden"),
            Err(Error::Http(resp)) => {
                assert_eq!(resp.status(), 403);
                let body = std::str::from_utf8(resp.body().as_ref().unwrap()).unwrap();
                let json: serde_json::Value = serde_json::from_str(body).unwrap();
                assert_eq!(
                    json["message"],
                    "Not allowed to access this submission websocket"
                );
            }
            Err(e) => panic!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn lecturer_cannot_connect_to_owners_stream() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let db = app_state.db();
        let addr = spawn_server(app).await;
        let (m, a, owner, lecturer, _admin) = seed(db).await;

        // give lecturer a staff role anyway (should still be forbidden)
        UserModuleRoleModel::assign_user_to_module(db, lecturer.id, m.id, ModuleRole::Lecturer)
            .await
            .expect("assign lecturer");

        let (token, _) = generate_jwt(lecturer.id, lecturer.admin);
        let topic = topic_for(m.id, a.id, owner.id);

        match connect_ws(&addr.to_string(), &topic, Some(&token)).await {
            Ok(_) => panic!("lecturer should be forbidden"),
            Err(Error::Http(resp)) => {
                assert_eq!(resp.status(), 403);
                let body = std::str::from_utf8(resp.body().as_ref().unwrap()).unwrap();
                let json: serde_json::Value = serde_json::from_str(body).unwrap();
                assert_eq!(
                    json["message"],
                    "Not allowed to access this submission websocket"
                );
            }
            Err(e) => panic!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn random_other_user_is_forbidden() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let addr = spawn_server(app).await;
        let (m, a, owner, other, _admin) = seed(app_state.db()).await;

        let (token, _) = generate_jwt(other.id, other.admin);
        let topic = topic_for(m.id, a.id, owner.id);

        match connect_ws(&addr.to_string(), &topic, Some(&token)).await {
            Ok(_) => panic!("non-owner should be forbidden"),
            Err(Error::Http(resp)) => {
                assert_eq!(resp.status(), 403);
                let body = std::str::from_utf8(resp.body().as_ref().unwrap()).unwrap();
                let json: serde_json::Value = serde_json::from_str(body).unwrap();
                assert_eq!(
                    json["message"],
                    "Not allowed to access this submission websocket"
                );
            }
            Err(e) => panic!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn unauthenticated_is_401() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let addr = spawn_server(app).await;
        let (m, a, owner, _other, _admin) = seed(app_state.db()).await;

        let topic = topic_for(m.id, a.id, owner.id);

        match connect_ws(&addr.to_string(), &topic, None).await {
            Ok(_) => panic!("unauthenticated user should not connect"),
            Err(Error::Http(resp)) => {
                assert_eq!(resp.status(), 401);
                let body = std::str::from_utf8(resp.body().as_ref().unwrap()).unwrap();
                let json: serde_json::Value = serde_json::from_str(body).unwrap();
                assert_eq!(json["message"], "Authentication required");
            }
            Err(e) => panic!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn assignment_not_in_module_is_404() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let db = app_state.db();
        let addr = spawn_server(app).await;
        let (_, a1, owner, _other, _admin) = seed(db).await;

        // mismatch module (create another)
        let m2 = module::ActiveModel {
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
        let topic = topic_for(m2.id, a1.id, owner.id); // mismatched

        match connect_ws(&addr.to_string(), &topic, Some(&token)).await {
            Ok(_) => panic!("should not connect when assignment not in module"),
            Err(Error::Http(resp)) => {
                assert_eq!(resp.status(), 404);
                let body = std::str::from_utf8(resp.body().as_ref().unwrap()).unwrap();
                let json: serde_json::Value = serde_json::from_str(body).unwrap();
                assert_eq!(json["message"], "Assignment not found in module");
            }
            Err(e) => panic!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn bad_ids_are_400() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let addr = spawn_server(app).await;

        // send a valid token so we pass the top-level /ws auth layer and hit ID validation
        let u = UserModel::create(app_state.db(), "u-any", "any@test.com", "pw", false)
            .await
            .unwrap();
        let (token, _) = generate_jwt(u.id, u.admin);

        // non-numeric params (this must still go through the same nested route)
        let topic = "modules/abc/assignments/def/submissions/ghi";

        match connect_ws(&addr.to_string(), topic, Some(&token)).await {
            Ok(_) => panic!("should not connect with bad IDs"),
            Err(Error::Http(resp)) => {
                assert_eq!(resp.status(), 400);
                let body = std::str::from_utf8(resp.body().as_ref().unwrap()).unwrap();
                let json: serde_json::Value = serde_json::from_str(body).unwrap();
                assert!(json["message"]
                    .as_str()
                    .unwrap()
                    .starts_with("Missing or invalid"));
            }
            Err(e) => panic!("unexpected error: {:?}", e),
        }
    }
}
