#[cfg(test)]
mod tests {
    //! Integration tests for assignment access controls (IP allowlist + PIN) and the verify endpoint.

    #![allow(clippy::unwrap_used)]

    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use chrono::{TimeZone, Utc};
    use serde_json::{Value, json};
    use serial_test::serial;
    use tower::ServiceExt;

    use crate::helpers::app::make_test_app_with_storage;
    use api::auth::generate_jwt;

    use db::models::{
        assignment::{AssignmentType, Model as AssignmentModel},
        assignment_file::{FileType, Model as AssignmentFile},
        module::Model as ModuleModel,
        user::Model as UserModel,
        user_module_role::{Model as UserModuleRoleModel, Role},
    };

    /// Local test fixture
    struct TestData {
        admin_user: UserModel,
        lecturer_user: UserModel,
        tutor_user: UserModel,
        student_user: UserModel,
        forbidden_user: UserModel,
        module: ModuleModel,
        assignments: Vec<AssignmentModel>,
    }

    /// Creates a module + users + roles + two assignments.
    async fn setup_test_data(db: &sea_orm::DatabaseConnection) -> TestData {
        let module = ModuleModel::create(db, "COS101", 2024, Some("Test Module"), 16)
            .await
            .unwrap();

        let admin_user = UserModel::create(db, "admin1", "admin1@test.com", "password", true)
            .await
            .unwrap();
        let lecturer_user =
            UserModel::create(db, "lecturer1", "lecturer1@test.com", "password1", false)
                .await
                .unwrap();
        let tutor_user = UserModel::create(db, "tutor1", "tutor1@test.com", "passwordt", false)
            .await
            .unwrap();
        let student_user =
            UserModel::create(db, "student1", "student1@test.com", "password2", false)
                .await
                .unwrap();
        let forbidden_user =
            UserModel::create(db, "forbidden", "forbidden@test.com", "password3", false)
                .await
                .unwrap();

        UserModuleRoleModel::assign_user_to_module(db, lecturer_user.id, module.id, Role::Lecturer)
            .await
            .unwrap();
        UserModuleRoleModel::assign_user_to_module(db, tutor_user.id, module.id, Role::Tutor)
            .await
            .unwrap();
        UserModuleRoleModel::assign_user_to_module(db, student_user.id, module.id, Role::Student)
            .await
            .unwrap();

        let a1 = AssignmentModel::create(
            db,
            module.id,
            "Assignment 1",
            Some("Desc 1"),
            AssignmentType::Assignment,
            Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 1, 31, 23, 59, 59).unwrap(),
        )
        .await
        .unwrap();

        let a2 = AssignmentModel::create(
            db,
            module.id,
            "Assignment 2",
            Some("Desc 2"),
            AssignmentType::Practical,
            Utc.with_ymd_and_hms(2024, 2, 1, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 2, 28, 23, 59, 59).unwrap(),
        )
        .await
        .unwrap();

        TestData {
            admin_user,
            lecturer_user,
            tutor_user,
            student_user,
            forbidden_user,
            module,
            assignments: vec![a1, a2],
        }
    }

    /// Helper: write a `config.json` with a `security` block for an assignment.
    async fn write_security_config(
        db: &sea_orm::DatabaseConnection,
        module_id: i64,
        assignment_id: i64,
        security: serde_json::Value,
    ) {
        // Minimal config that the backend will read; other fields omitted on purpose
        let config = json!({
            "security": security
        });

        let bytes = serde_json::to_vec_pretty(&config).unwrap();
        AssignmentFile::save_file(
            db,
            assignment_id,
            module_id,
            FileType::Config,
            "config.json",
            &bytes,
        )
        .await
        .unwrap();
    }

    /* ─────────────────────────────────────────────────
     * /verify endpoint
     * ───────────────────────────────────────────────── */

    #[tokio::test]
    #[serial]
    async fn verify_pin_ok_when_enabled() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        // Enable PIN, allow all IPs
        write_security_config(
            app_state.db(),
            data.module.id,
            data.assignments[0].id,
            json!({
                "password_enabled": true,
                "password_pin": "PIN123",
                "cookie_ttl_minutes": 60,
                "bind_cookie_to_user": true,
                "allowed_cidrs": ["0.0.0.0/0"]
            }),
        )
        .await;

        // Any authenticated user is fine (use the student)
        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);

        let uri = format!(
            "/api/modules/{}/assignments/{}/verify",
            data.module.id, data.assignments[0].id
        );
        let body = json!({ "pin": "PIN123" });

        let req = Request::builder()
            .method("POST")
            .uri(uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();

        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        let body = axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);
    }

    #[tokio::test]
    #[serial]
    async fn verify_pin_invalid_when_enabled() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        write_security_config(
            app_state.db(),
            data.module.id,
            data.assignments[0].id,
            json!({
                "password_enabled": true,
                "password_pin": "SECRET",
                "cookie_ttl_minutes": 60,
                "bind_cookie_to_user": true,
                "allowed_cidrs": ["0.0.0.0/0"]
            }),
        )
        .await;

        let uri = format!(
            "/api/modules/{}/assignments/{}/verify",
            data.module.id, data.assignments[0].id
        );
        let body = json!({ "pin": "WRONG" });

        let req = Request::builder()
            .method("POST")
            .uri(uri)
            .header("Content-Type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();

        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

        let body = axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
    }

    #[tokio::test]
    #[serial]
    async fn verify_pin_ignored_when_disabled() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        write_security_config(
            app_state.db(),
            data.module.id,
            data.assignments[0].id,
            json!({
                "password_enabled": false,
                "password_pin": "ANY",
                "cookie_ttl_minutes": 60,
                "bind_cookie_to_user": true,
                "allowed_cidrs": ["0.0.0.0/0"]
            }),
        )
        .await;

        // Any authenticated user is fine (use the student)
        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);

        let uri = format!(
            "/api/modules/{}/assignments/{}/verify",
            data.module.id, data.assignments[0].id
        );
        let body = json!({ "pin": "does-not-matter" });

        let req = Request::builder()
            .method("POST")
            .uri(uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();

        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        let body = axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);
    }

    /* ─────────────────────────────────────────────────
     * Guarded GET /assignments/:id (students)
     * ───────────────────────────────────────────────── */

    #[tokio::test]
    #[serial]
    async fn student_blocked_by_ip_allowlist() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        // Only allow 10.0.0.0/8; test client defaults to 127.0.0.1 (blocked)
        write_security_config(
            app_state.db(),
            data.module.id,
            data.assignments[0].id,
            json!({
                "password_enabled": false,
                "allowed_cidrs": ["10.0.0.0/8"],
                "cookie_ttl_minutes": 60,
                "bind_cookie_to_user": true
            }),
        )
        .await;

        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}",
            data.module.id, data.assignments[0].id
        );
        let req = Request::builder()
            .method("GET")
            .uri(uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    #[serial]
    async fn student_allowed_by_ip_allowlist() {
        let (app, app_state, _) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        // Allow 127.0.0.1
        write_security_config(
            app_state.db(),
            data.module.id,
            data.assignments[0].id,
            json!({
                "password_enabled": false,
                "allowed_cidrs": ["127.0.0.1/32"],
                "cookie_ttl_minutes": 60,
                "bind_cookie_to_user": true
            }),
        )
        .await;

        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}",
            data.module.id, data.assignments[0].id
        );
        let req = Request::builder()
            .method("GET")
            .uri(uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    #[serial]
    async fn student_blocked_without_pin_when_required() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        // PIN required; IP allowed
        write_security_config(
            app_state.db(),
            data.module.id,
            data.assignments[0].id,
            json!({
                "password_enabled": true,
                "password_pin": "123456",
                "allowed_cidrs": ["127.0.0.1/32"],
                "cookie_ttl_minutes": 60,
                "bind_cookie_to_user": true
            }),
        )
        .await;

        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}",
            data.module.id, data.assignments[0].id
        );
        let req = Request::builder()
            .method("GET")
            .uri(uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    #[serial]
    async fn student_allowed_with_correct_pin_and_ip_ok() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        write_security_config(
            app_state.db(),
            data.module.id,
            data.assignments[0].id,
            json!({
                "password_enabled": true,
                "password_pin": "LETMEIN",
                "allowed_cidrs": ["127.0.0.1/32"],
                "cookie_ttl_minutes": 60,
                "bind_cookie_to_user": true
            }),
        )
        .await;

        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}",
            data.module.id, data.assignments[0].id
        );
        let req = Request::builder()
            .method("GET")
            .uri(uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("x-assignment-pin", "LETMEIN")
            .body(Body::empty())
            .unwrap();

        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    #[serial]
    async fn ip_checked_before_pin_even_if_pin_correct() {
        // If IP is not allowed, even a correct PIN should not grant access.
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        write_security_config(
            app_state.db(),
            data.module.id,
            data.assignments[0].id,
            json!({
                "password_enabled": true,
                "password_pin": "ABC",
                "allowed_cidrs": ["10.0.0.0/8"], // excludes 127.0.0.1
                "cookie_ttl_minutes": 60,
                "bind_cookie_to_user": true
            }),
        )
        .await;

        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}",
            data.module.id, data.assignments[0].id
        );
        let req = Request::builder()
            .method("GET")
            .uri(uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("x-assignment-pin", "ABC") // correct, but IP will fail first
            .body(Body::empty())
            .unwrap();

        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::FORBIDDEN);
    }

    /* ─────────────────────────────────────────────────
     * Staff bypass (lecturer, tutor, admin)
     * ───────────────────────────────────────────────── */

    #[tokio::test]
    #[serial]
    async fn lecturer_bypasses_ip_and_pin() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        // Disallow client IP and require PIN; lecturer should still pass
        write_security_config(
            app_state.db(),
            data.module.id,
            data.assignments[0].id,
            json!({
                "password_enabled": true,
                "password_pin": "S",
                "allowed_cidrs": ["10.0.0.0/8"],
                "cookie_ttl_minutes": 60,
                "bind_cookie_to_user": true
            }),
        )
        .await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}",
            data.module.id, data.assignments[0].id
        );
        let req = Request::builder()
            .method("GET")
            .uri(uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    #[serial]
    async fn tutor_bypasses_ip_and_pin() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        write_security_config(
            app_state.db(),
            data.module.id,
            data.assignments[0].id,
            json!({
                "password_enabled": true,
                "password_pin": "S",
                "allowed_cidrs": ["10.0.0.0/8"],
                "cookie_ttl_minutes": 60,
                "bind_cookie_to_user": true
            }),
        )
        .await;

        let (token, _) = generate_jwt(data.tutor_user.id, data.tutor_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}",
            data.module.id, data.assignments[0].id
        );
        let req = Request::builder()
            .method("GET")
            .uri(uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    #[serial]
    async fn admin_bypasses_ip_and_pin() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        write_security_config(
            app_state.db(),
            data.module.id,
            data.assignments[0].id,
            json!({
                "password_enabled": true,
                "password_pin": "S",
                "allowed_cidrs": ["10.0.0.0/8"],
                "cookie_ttl_minutes": 60,
                "bind_cookie_to_user": true
            }),
        )
        .await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}",
            data.module.id, data.assignments[0].id
        );
        let req = Request::builder()
            .method("GET")
            .uri(uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    /* ─────────────────────────────────────────────────
     * Unassigned user rejected by module guard
     * ───────────────────────────────────────────────── */

    #[tokio::test]
    #[serial]
    async fn unassigned_user_forbidden_even_without_security() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        // No special security (allow all IPs, no PIN)
        write_security_config(
            app_state.db(),
            data.module.id,
            data.assignments[0].id,
            json!({
                "password_enabled": false,
                "allowed_cidrs": ["0.0.0.0/0"],
                "cookie_ttl_minutes": 60,
                "bind_cookie_to_user": false
            }),
        )
        .await;

        let (token, _) = generate_jwt(data.forbidden_user.id, data.forbidden_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}",
            data.module.id, data.assignments[0].id
        );
        let req = Request::builder()
            .method("GET")
            .uri(uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let res = app.clone().oneshot(req).await.unwrap();
        // blocked by require_assigned_to_module
        assert_eq!(res.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    #[serial]
    async fn verify_requires_authentication() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        // PIN disabled; still requires auth because of global auth layer.
        write_security_config(
            app_state.db(),
            data.module.id,
            data.assignments[0].id,
            json!({
                "password_enabled": false,
                "allowed_cidrs": ["0.0.0.0/0"],
                "cookie_ttl_minutes": 60,
                "bind_cookie_to_user": false
            }),
        )
        .await;

        let uri = format!(
            "/api/modules/{}/assignments/{}/verify",
            data.module.id, data.assignments[0].id
        );
        let body = json!({ "pin": "anything" });

        let req = Request::builder()
            .method("POST")
            .uri(uri)
            .header("Content-Type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();

        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    #[serial]
    async fn pin_required_error_message_when_missing() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        write_security_config(
            app_state.db(),
            data.module.id,
            data.assignments[0].id,
            json!({
                "password_enabled": true,
                "password_pin": "SECRET",
                "allowed_cidrs": ["127.0.0.1/32"],
                "cookie_ttl_minutes": 60,
                "bind_cookie_to_user": true
            }),
        )
        .await;

        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}",
            data.module.id, data.assignments[0].id
        );
        let req = Request::builder()
            .method("GET")
            .uri(uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::FORBIDDEN);

        let body = axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "PIN required");
    }

    #[tokio::test]
    #[serial]
    async fn ip_not_allowed_error_message_even_with_correct_pin() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        write_security_config(
            app_state.db(),
            data.module.id,
            data.assignments[0].id,
            json!({
                "password_enabled": true,
                "password_pin": "OKPIN",
                "allowed_cidrs": ["10.0.0.0/8"], // excludes 127.0.0.1
                "cookie_ttl_minutes": 60,
                "bind_cookie_to_user": true
            }),
        )
        .await;

        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}",
            data.module.id, data.assignments[0].id
        );
        let req = Request::builder()
            .method("GET")
            .uri(uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("x-assignment-pin", "OKPIN")
            .body(Body::empty())
            .unwrap();

        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::FORBIDDEN);

        let body = axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "IP not allowed");
    }

    #[tokio::test]
    #[serial]
    async fn assistant_lecturer_bypasses_ip_and_pin() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        // Create an assistant lecturer and assign to module
        let al_user = UserModel::create(app_state.db(), "al1", "al1@test.com", "pass", false)
            .await
            .unwrap();
        UserModuleRoleModel::assign_user_to_module(
            app_state.db(),
            al_user.id,
            data.module.id,
            Role::AssistantLecturer,
        )
        .await
        .unwrap();

        // Hardest case: disallow IP and require PIN — should still pass
        write_security_config(
            app_state.db(),
            data.module.id,
            data.assignments[0].id,
            json!({
                "password_enabled": true,
                "password_pin": "S",
                "allowed_cidrs": ["10.0.0.0/8"],
                "cookie_ttl_minutes": 60,
                "bind_cookie_to_user": true
            }),
        )
        .await;

        let (token, _) = generate_jwt(al_user.id, al_user.admin);
        let uri = format!(
            "/api/modules/{}/assignments/{}",
            data.module.id, data.assignments[0].id
        );
        let req = Request::builder()
            .method("GET")
            .uri(uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }
}
