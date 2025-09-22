#[cfg(test)]
mod tests {
    use crate::helpers::app::make_test_app_with_storage;
    use api::auth::generate_jwt;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use db::models::{
        announcements::Model as AnnouncementModel,
        module::Model as ModuleModel,
        user::Model as UserModel,
        user_module_role::{Model as UserModuleRole, Role},
    };
    use serde_json::Value;
    use tower::ServiceExt;

    struct TestData {
        student: UserModel,
        lecturer: UserModel,
        invalid_user: UserModel,
        module: ModuleModel,
        announcement_ids: Vec<i64>,
    }

    async fn setup_test_data(db: &sea_orm::DatabaseConnection) -> TestData {
        // Users
        let student = UserModel::create(db, "student_user", "student@example.com", "pass", false)
            .await
            .unwrap();

        let lecturer =
            UserModel::create(db, "lecturer_user", "lecturer@example.com", "pass2", true)
                .await
                .unwrap();

        let invalid_user =
            UserModel::create(db, "invalid_user", "invalid@example.com", "pass3", false)
                .await
                .unwrap();

        // Module
        let module = ModuleModel::create(db, "330", 2025, Some("Test module"), 16)
            .await
            .unwrap();

        // Roles
        UserModuleRole::assign_user_to_module(db, student.id, module.id, Role::Student)
            .await
            .unwrap();

        UserModuleRole::assign_user_to_module(db, lecturer.id, module.id, Role::Lecturer)
            .await
            .unwrap();

        // Seed a few announcements (two pinned, one unpinned)
        let mut ids = Vec::new();
        let id1 = AnnouncementModel::create(
            db,
            module.id,
            lecturer.id,
            "Pinned A",
            "Body A",
            true,
        )
        .await
        .unwrap()
        .id;
        ids.push(id1);

        let id2 = AnnouncementModel::create(
            db,
            module.id,
            lecturer.id,
            "Normal B",
            "Body B",
            false,
        )
        .await
        .unwrap()
        .id;
        ids.push(id2);

        let id3 = AnnouncementModel::create(
            db,
            module.id,
            lecturer.id,
            "Pinned Newest",
            "Body C",
            true,
        )
        .await
        .unwrap()
        .id;
        ids.push(id3);

        TestData {
            student,
            lecturer,
            invalid_user,
            module,
            announcement_ids: ids,
        }
    }

    // ─────────────────────────────────────────────────────────────
    // Helpers
    // ─────────────────────────────────────────────────────────────

    async fn read_json_body(res: axum::response::Response) -> Value {
        let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap();
        serde_json::from_slice::<Value>(&body_bytes).unwrap()
    }

    // ─────────────────────────────────────────────────────────────
    // LIST: GET /api/modules/{module_id}/announcements
    // ─────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn list_announcements_success_assigned_user() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;
        let (token, _) = generate_jwt(data.student.id, data.student.admin);

        let uri = format!("/api/modules/{}/announcements", data.module.id);

        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let json = read_json_body(response).await;

        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "Announcements retrieved successfully");
        let arr = json["data"]["announcements"].as_array().unwrap();
        assert!(arr.len() >= 3);

        // Default sort: pinned DESC then created_at DESC
        // Ensure first one is pinned and all pinned come before any unpinned
        let mut first_unpinned_idx: Option<usize> = None;
        for (i, a) in arr.iter().enumerate() {
            let p = a["pinned"].as_bool().unwrap();
            if !p {
                first_unpinned_idx = Some(i);
                break;
            }
        }
        if let Some(idx) = first_unpinned_idx {
            // After first unpinned, there should be no pinned
            for a in &arr[idx..] {
                assert_eq!(a["pinned"].as_bool().unwrap(), false);
            }
        }
    }

    #[tokio::test]
    async fn list_announcements_filter_pinned_true() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;
        let (token, _) = generate_jwt(data.student.id, data.student.admin);

        let uri = format!("/api/modules/{}/announcements?pinned=true", data.module.id);
        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let json = read_json_body(response).await;

        let arr = json["data"]["announcements"].as_array().unwrap();
        assert!(!arr.is_empty());
        for a in arr {
            assert_eq!(a["pinned"], true);
        }
    }

    #[tokio::test]
    async fn list_announcements_invalid_pinned_param() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;
        let (token, _) = generate_jwt(data.student.id, data.student.admin);

        let uri = format!("/api/modules/{}/announcements?pinned=maybe", data.module.id);
        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let json = read_json_body(response).await;
        assert_eq!(json["success"], false);
    }

    #[tokio::test]
    async fn list_announcements_invalid_sort_field() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;
        let (token, _) = generate_jwt(data.student.id, data.student.admin);

        let uri = format!("/api/modules/{}/announcements?sort=bogus", data.module.id);
        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let json = read_json_body(response).await;
        assert_eq!(json["success"], false);
        assert!(json["message"].as_str().unwrap_or_default().to_lowercase().contains("invalid"));
    }

    #[tokio::test]
    async fn list_announcements_unauthorized_no_token() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let uri = format!("/api/modules/{}/announcements", data.module.id);
        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn list_announcements_forbidden_not_in_module() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;
        let (token, _) = generate_jwt(data.invalid_user.id, data.invalid_user.admin);

        let uri = format!("/api/modules/{}/announcements", data.module.id);
        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    // ─────────────────────────────────────────────────────────────
    // SHOW: GET /api/modules/{module_id}/announcements/{announcement_id}
    // ─────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn get_announcement_success_with_minimal_user() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;
        let (token, _) = generate_jwt(data.student.id, data.student.admin);

        let ann_id = data.announcement_ids[0];
        let uri = format!(
            "/api/modules/{}/announcements/{}",
            data.module.id, ann_id
        );

        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let json = read_json_body(response).await;
        assert_eq!(json["success"], true);

        // Check announcement payload
        assert_eq!(json["data"]["announcement"]["id"], Value::from(ann_id));
        assert_eq!(json["data"]["announcement"]["module_id"], Value::from(data.module.id));

        // Check minimal user (only id & username)
        let user = &json["data"]["user"];
        assert_eq!(user["id"], Value::from(data.lecturer.id));
        assert_eq!(user["username"], Value::from("lecturer_user"));
        // Ensure we don't expose other fields (like email)
        assert!(user.get("email").is_none() || user["email"].is_null());
        assert!(user.get("password_hash").is_none() || user["password_hash"].is_null());
    }

    #[tokio::test]
    async fn get_announcement_not_found() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;
        let (token, _) = generate_jwt(data.student.id, data.student.admin);

        // Non-existent announcement id
        let uri = format!("/api/modules/{}/announcements/{}", data.module.id, 9_999_999);

        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let json = read_json_body(response).await;
        assert_eq!(json["success"], false);
    }

    #[tokio::test]
    async fn get_announcement_unauthorized_no_token() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let ann_id = data.announcement_ids[0];
        let uri = format!(
            "/api/modules/{}/announcements/{}",
            data.module.id, ann_id
        );

        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn get_announcement_forbidden_not_in_module() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;
        let (token, _) = generate_jwt(data.invalid_user.id, data.invalid_user.admin);

        let ann_id = data.announcement_ids[0];
        let uri = format!(
            "/api/modules/{}/announcements/{}",
            data.module.id, ann_id
        );

        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
}
