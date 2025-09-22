#[cfg(test)]
mod tests {
    use std::sync::Once;

    use api::auth::generate_jwt;
    use axum::{
        body::Body as AxumBody,
        http::{Request, StatusCode},
    };
    use chrono::{TimeZone, Utc};
    use db::models::{
        assignment::{AssignmentType, Model as AssignmentModel},
        assignment_submission::ActiveModel as SubmissionActiveModel,
        module::Model as ModuleModel,
        system_metric::ActiveModel as SystemMetricActiveModel,
        user::Model as UserModel,
        user_module_role::{Model as UserModuleRoleModel, Role},
    };
    use sea_orm::ActiveModelTrait;
    use sea_orm::ActiveValue::{NotSet, Set};
    use serde_json::Value;
    use serial_test::serial;
    use tower::ServiceExt;

    use crate::helpers::app::make_test_app_with_storage;

    // ---------- test helpers ----------

    fn ensure_jwt_env() {
        static INIT: Once = Once::new();
        INIT.call_once(|| {
            if std::env::var("JWT_SECRET").is_err() {
                unsafe { std::env::set_var("JWT_SECRET", "test-secret") };
            }
            if std::env::var("JWT_DURATION_MINUTES").is_err() {
                unsafe { std::env::set_var("JWT_DURATION_MINUTES", "60") };
            }
            if std::env::var("APP_ENV").is_err() {
                unsafe { std::env::set_var("APP_ENV", "test") };
            }
        });
    }

    /// Build an authenticated GET request
    fn authed_get(uri: &str, token: &str) -> Request<AxumBody> {
        Request::builder()
            .method("GET")
            .uri(uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap()
    }

    // ---------- TESTS ----------

    #[serial]
    #[tokio::test]
    async fn admin_can_read_code_manager_max_concurrent() {
        ensure_jwt_env();
        let (app, app_state, _tmp) = make_test_app_with_storage().await;

        // Admin user + JWT
        let admin = UserModel::create(
            app_state.db(),
            "admin_user",
            "admin@test.com",
            "password",
            true,
        )
        .await
        .unwrap();
        let (token, _) = generate_jwt(admin.id, admin.admin);

        // Call real Code Manager via GET /api/system/code-manager/max-concurrent
        let req = authed_get("/api/system/code-manager/max-concurrent", &token);
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "OK");
        let val = json["data"]
            .as_u64()
            .expect("max_concurrent should be a number");
        assert!(
            val >= 1,
            "expected max_concurrent >= 1 from running Code Manager, got {val}"
        );
    }

    #[serial]
    #[tokio::test]
    async fn non_admin_roles_forbidden_on_all_system_get_routes() {
        ensure_jwt_env();
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let db = app_state.db();

        // Create a module and users with various roles
        let module = ModuleModel::create(db, "SYS101", 2024, Some("System Test Module"), 16)
            .await
            .unwrap();

        let roles = [
            (Role::Lecturer, "lecturer_user"),
            (Role::AssistantLecturer, "assistant_user"),
            (Role::Tutor, "tutor_user"),
            (Role::Student, "student_user"),
        ];

        for (role, username) in roles {
            let user =
                UserModel::create(db, username, &format!("{username}@test.com"), "pw", false)
                    .await
                    .unwrap();
            UserModuleRoleModel::assign_user_to_module(db, user.id, module.id, role)
                .await
                .unwrap();
            let (token, _) = generate_jwt(user.id, user.admin);

            for uri in [
                "/api/system/code-manager/max-concurrent",
                "/api/system/metrics?start=2024-01-01T00:00:00Z&end=2024-01-01T23:59:59Z&bucket=day",
                "/api/system/metrics/export?start=2024-01-01T00:00:00Z&end=2024-01-01T23:59:59Z&bucket=day",
                "/api/system/submissions?start=2024-01-01T00:00:00Z&end=2024-01-01T23:59:59Z&bucket=day",
                "/api/system/submissions/export?start=2024-01-01T00:00:00Z&end=2024-01-01T23:59:59Z&bucket=day",
            ] {
                let req = authed_get(uri, &token);
                let resp = app.clone().oneshot(req).await.unwrap();
                assert_eq!(
                    resp.status(),
                    StatusCode::FORBIDDEN,
                    "non-admin should be forbidden for {}",
                    uri
                );
            }
        }
    }

    #[serial]
    #[tokio::test]
    async fn missing_or_invalid_token_is_unauthorized_for_system_routes() {
        ensure_jwt_env();
        let (app, _app_state, _tmp) = make_test_app_with_storage().await;

        let targets = [
            "/api/system/code-manager/max-concurrent",
            "/api/system/metrics?start=2024-01-01T00:00:00Z&end=2024-01-01T23:59:59Z&bucket=day",
            "/api/system/metrics/export?start=2024-01-01T00:00:00Z&end=2024-01-01T23:59:59Z&bucket=day",
            "/api/system/submissions?start=2024-01-01T00:00:00Z&end=2024-01-01T23:59:59Z&bucket=day",
            "/api/system/submissions/export?start=2024-01-01T00:00:00Z&end=2024-01-01T23:59:59Z&bucket=day",
        ];

        for uri in targets {
            // No auth header
            let req = Request::builder()
                .method("GET")
                .uri(uri)
                .body(AxumBody::empty())
                .unwrap();
            let resp = app.clone().oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::UNAUTHORIZED, "no token: {uri}");

            // Invalid token
            let req = Request::builder()
                .method("GET")
                .uri(uri)
                .header("Authorization", "Bearer invalid.token.here")
                .body(AxumBody::empty())
                .unwrap();
            let resp = app.clone().oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::UNAUTHORIZED, "bad token: {uri}");
        }
    }

    #[serial]
    #[tokio::test]
    async fn admin_can_fetch_metrics_and_export() {
        ensure_jwt_env();
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let db = app_state.db();

        // Admin + token
        let admin = UserModel::create(db, "metrics_admin", "metrics_admin@test.com", "pw", true)
            .await
            .unwrap();
        let (token, _) = generate_jwt(admin.id, admin.admin);

        // Insert one metric at 2024-01-01T12:00:00Z
        let base_ts = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();
        let metric = SystemMetricActiveModel {
            id: NotSet,
            created_at: Set(base_ts),
            cpu_avg: Set(42.0_f32),
            mem_pct: Set(75.0_f32),
        };
        metric.insert(db).await.unwrap();

        // JSON metrics
        let query =
            "/api/system/metrics?start=2024-01-01T00:00:00Z&end=2024-01-01T23:59:59Z&bucket=day";
        let resp = app
            .clone()
            .oneshot(authed_get(query, &token))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        let points = json["points"].as_array().unwrap();
        let expected_ts = "2024-01-01T12:00:00+00:00";
        let point = points
            .iter()
            .find(|p| p["ts"] == expected_ts)
            .expect("expected metrics point present");
        let cpu = point["cpu_avg"].as_f64().unwrap();
        let mem = point["mem_pct"].as_f64().unwrap();
        assert!((cpu - 42.0).abs() < 1e-6);
        assert!((mem - 75.0).abs() < 1e-6);

        // CSV export
        let export_uri = "/api/system/metrics/export?start=2024-01-01T00:00:00Z&end=2024-01-01T23:59:59Z&bucket=day";
        let csv_resp = app
            .clone()
            .oneshot(authed_get(export_uri, &token))
            .await
            .unwrap();
        assert_eq!(csv_resp.status(), StatusCode::OK);
        let csv = axum::body::to_bytes(csv_resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let csv_str = String::from_utf8(csv.to_vec()).unwrap();
        assert!(csv_str.contains("timestamp,cpu_avg,mem_pct"));
        assert!(csv_str.contains("2024-01-01T12:00:00+00:00,42,75"));
    }

    #[serial]
    #[tokio::test]
    async fn admin_can_fetch_submissions_and_export() {
        ensure_jwt_env();
        let (app, app_state, _tmp_storage) = make_test_app_with_storage().await;
        let db = app_state.db();

        // Admin + token
        let admin = UserModel::create(db, "sub_admin", "sub_admin@test.com", "pw", true)
            .await
            .unwrap();
        let (token, _) = generate_jwt(admin.id, admin.admin);

        // Create module + assignment + student + one submission at 2024-01-01T12:00:00Z
        let module = ModuleModel::create(db, "COS999", 2024, Some("System module"), 16)
            .await
            .unwrap();

        let available_from = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let due_date = Utc.with_ymd_and_hms(2024, 1, 10, 0, 0, 0).unwrap();
        let assignment = AssignmentModel::create(
            db,
            module.id,
            "A01",
            Some("Test Assignment"),
            AssignmentType::Practical,
            available_from,
            due_date,
        )
        .await
        .unwrap();

        let student = UserModel::create(db, "student_sub", "student_sub@test.com", "pw", false)
            .await
            .unwrap();

        let submission_time = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();
        let submission = SubmissionActiveModel {
            id: NotSet,
            assignment_id: Set(assignment.id),
            user_id: Set(student.id),
            attempt: Set(1),
            earned: Set(80),
            total: Set(100),
            filename: Set("submission.zip".into()),
            file_hash: Set("hash".into()),
            path: Set("path/to/file".into()),
            is_practice: Set(false),
            ignored: Set(false),
            status: Set(db::models::assignment_submission::SubmissionStatus::Graded),
            created_at: Set(submission_time),
            updated_at: Set(submission_time),
        };
        submission.insert(db).await.unwrap();

        // JSON submissions
        let query = format!(
            "/api/system/submissions?start=2024-01-01T00:00:00Z&end=2024-01-01T23:59:59Z&bucket=day&assignment_id={}",
            assignment.id
        );
        let resp = app
            .clone()
            .oneshot(authed_get(&query, &token))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        let points = json["points"].as_array().unwrap();
        let point = points
            .iter()
            .find(|p| p["period"] == "2024-01-01 12:00:00")
            .expect("expected submissions point present");
        assert_eq!(point["count"], 1);

        // CSV export
        let export_uri = format!(
            "/api/system/submissions/export?start=2024-01-01T00:00:00Z&end=2024-01-01T23:59:59Z&bucket=day&assignment_id={}",
            assignment.id
        );
        let csv_resp = app
            .clone()
            .oneshot(authed_get(&export_uri, &token))
            .await
            .unwrap();
        assert_eq!(csv_resp.status(), StatusCode::OK);
        let csv = axum::body::to_bytes(csv_resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let csv_str = String::from_utf8(csv.to_vec()).unwrap();
        assert!(csv_str.contains("period,count"));
        assert!(csv_str.contains("2024-01-01 12:00:00,1"));
    }
}
