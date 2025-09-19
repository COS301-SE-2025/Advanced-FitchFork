#[cfg(test)]
mod tests {
    use api::auth::generate_jwt;
    use db::models::{
        module::Model as ModuleModel,
        user::Model as UserModel,
        user_module_role::{Model as UserModuleRoleModel, Role},
    };
    use serial_test::serial;
    use std::sync::Once;
    use tokio_tungstenite::tungstenite::Error;

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

    #[serial]
    #[tokio::test]
    async fn admin_can_connect_to_system_health_admin_topic() {
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

        let (mut ws, _) = connect_ws(
            &addr.to_string(),
            "system/health/admin",
            Some(token.as_str()),
        )
        .await
        .unwrap();
        ws.close(None).await.unwrap();
    }

    #[serial]
    #[tokio::test]
    async fn non_admin_roles_cannot_connect_to_system_health_admin_topic() {
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

        let (lect_tok, _) = generate_jwt(lecturer.id, lecturer.admin);
        let (tutor_tok, _) = generate_jwt(tutor.id, tutor.admin);
        let (stud_tok, _) = generate_jwt(student.id, student.admin);

        for tok in [lect_tok, tutor_tok, stud_tok] {
            let res =
                connect_ws(&addr.to_string(), "system/health/admin", Some(tok.as_str())).await;
            match res {
                Ok(_) => panic!("Non-admin role should not connect to /system/health/admin"),
                Err(Error::Http(resp)) => assert_eq!(resp.status(), 403),
                Err(e) => panic!("Unexpected websocket error: {e:?}"),
            }
        }
    }

    #[serial]
    #[tokio::test]
    async fn authenticated_user_can_connect_to_system_health_topic() {
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

        // any authenticated user (no role required)
        let (mut ws, _) = connect_ws(&addr.to_string(), "system/health", Some(token.as_str()))
            .await
            .unwrap();
        ws.close(None).await.unwrap();
    }

    #[serial]
    #[tokio::test]
    async fn unauthenticated_user_cannot_connect_to_system_health_topic() {
        ensure_jwt_env();
        let (app, _app_state, _tmp) = make_test_app_with_storage().await;
        let addr = spawn_server(app).await;

        let res = connect_ws(&addr.to_string(), "system/health", None).await;
        match res {
            Ok(_) => panic!("Unauthenticated client should not connect"),
            Err(Error::Http(resp)) => assert_eq!(resp.status(), 401),
            Err(e) => panic!("Unexpected websocket error: {e:?}"),
        }
    }

    #[serial]
    #[tokio::test]
    async fn invalid_token_cannot_connect_to_system_health_topic() {
        ensure_jwt_env();
        let (app, _app_state, _tmp) = make_test_app_with_storage().await;
        let addr = spawn_server(app).await;

        let res = connect_ws(&addr.to_string(), "system/health", Some("not.a.valid.jwt")).await;
        match res {
            Ok(_) => panic!("Invalid token should not connect"),
            Err(Error::Http(resp)) => assert_eq!(resp.status(), 401),
            Err(e) => panic!("Unexpected websocket error: {e:?}"),
        }
    }
}
