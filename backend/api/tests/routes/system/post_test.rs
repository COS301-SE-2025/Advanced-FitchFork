#[cfg(test)]
mod tests {
    use std::sync::Once;

    use api::auth::generate_jwt;
    use axum::body::Body as AxumBody;
    use axum::http::{Request, StatusCode};
    use db::models::{
        module::Model as ModuleModel,
        user::Model as UserModel,
        user_module_role::{Model as UserModuleRoleModel, Role},
    };
    use serde_json::{Value, json};
    use serial_test::serial;
    use tower::ServiceExt;

    use crate::helpers::app::make_test_app_with_storage;

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

    fn authed_post_json(uri: &str, token: &str, json_body: Value) -> Request<AxumBody> {
        Request::builder()
            .method("POST")
            .uri(uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(AxumBody::from(json_body.to_string()))
            .unwrap()
    }

    // --- Tests ---

    /// Happy path: admin updates max_concurrent.
    /// This calls the **real** Code Manager configured in util::config.
    #[serial]
    #[tokio::test]
    async fn test_admin_can_update_max_concurrent() {
        ensure_jwt_env();
        let (app, app_state, _tmp) = make_test_app_with_storage().await;

        let admin = UserModel::create(
            app_state.db(),
            "admin_post",
            "admin_post@test.com",
            "password",
            true,
        )
        .await
        .unwrap();
        let (token, _) = generate_jwt(admin.id, admin.admin);

        // Use a conservative positive value (>= 1).
        let payload = json!({ "max_concurrent": 2 });
        let req = authed_post_json("/api/system/code-manager/max-concurrent", &token, payload);

        let response = app.clone().oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["data"], 2);
        assert_eq!(json["message"], "Updated");
    }

    /// Validation: zero is rejected before contacting Code Manager.
    #[serial]
    #[tokio::test]
    async fn test_validation_rejects_zero_max_concurrent() {
        ensure_jwt_env();
        let (app, app_state, _tmp) = make_test_app_with_storage().await;

        let admin = UserModel::create(
            app_state.db(),
            "admin_zero",
            "admin_zero@test.com",
            "password",
            true,
        )
        .await
        .unwrap();
        let (token, _) = generate_jwt(admin.id, admin.admin);

        let payload = json!({ "max_concurrent": 0 });
        let req = authed_post_json("/api/system/code-manager/max-concurrent", &token, payload);
        let response = app.clone().oneshot(req).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "max_concurrent must be >= 1");
    }

    /// Permissions: any non-admin gets 403 (middleware blocks before external call).
    #[serial]
    #[tokio::test]
    async fn test_non_admin_roles_cannot_update_max_concurrent() {
        ensure_jwt_env();
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let db = app_state.db();

        let module = ModuleModel::create(db, "SYS201", 2024, Some("Sys Module"), 16)
            .await
            .unwrap();

        let roles = [
            (Role::Lecturer, "lecturer_post"),
            (Role::AssistantLecturer, "assistant_post"),
            (Role::Tutor, "tutor_post"),
            (Role::Student, "student_post"),
        ];

        for (role, username) in roles {
            let user = UserModel::create(
                db,
                username,
                &format!("{username}@test.com"),
                "password",
                false,
            )
            .await
            .unwrap();
            UserModuleRoleModel::assign_user_to_module(db, user.id, module.id, role)
                .await
                .unwrap();

            let (token, _) = generate_jwt(user.id, user.admin);
            let payload = json!({ "max_concurrent": 3 });

            let req = authed_post_json("/api/system/code-manager/max-concurrent", &token, payload);
            let resp = app.clone().oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::FORBIDDEN);
        }
    }

    /// Auth: missing or invalid token → 401 (no external call made).
    #[serial]
    #[tokio::test]
    async fn test_missing_or_invalid_token_yields_unauthorized() {
        ensure_jwt_env();
        let (app, _app_state, _tmp) = make_test_app_with_storage().await;

        // Missing token
        let req = Request::builder()
            .method("POST")
            .uri("/api/system/code-manager/max-concurrent")
            .header("Content-Type", "application/json")
            .body(AxumBody::from(json!({"max_concurrent": 2}).to_string()))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

        // Invalid token
        let req = Request::builder()
            .method("POST")
            .uri("/api/system/code-manager/max-concurrent")
            .header("Authorization", "Bearer not.a.valid.jwt")
            .header("Content-Type", "application/json")
            .body(AxumBody::from(json!({"max_concurrent": 2}).to_string()))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }
}
