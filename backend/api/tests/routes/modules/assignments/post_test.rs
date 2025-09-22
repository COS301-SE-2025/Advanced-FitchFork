#[cfg(test)]
mod tests {
    use crate::helpers::app::make_test_app_with_storage;
    use crate::helpers::config_assert::assert_default_config;
    use api::auth::generate_jwt;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use db::models::{
        module::Model as ModuleModel,
        user::Model as UserModel,
        user_module_role::{Model as UserModuleRoleModel, Role},
    };
    use serde_json::json;
    use tower::ServiceExt;

    struct TestData {
        admin_user: UserModel,
        lecturer_user: UserModel,
        student_user: UserModel,
        forbidden_user: UserModel,
        module: ModuleModel,
    }

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
        UserModuleRoleModel::assign_user_to_module(db, student_user.id, module.id, Role::Student)
            .await
            .unwrap();

        TestData {
            admin_user,
            lecturer_user,
            student_user,
            forbidden_user,
            module,
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_create_assignment_success_as_lecturer() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments", data.module.id);
        let body = json!({
            "name": "New Assignment",
            "description": "A test assignment",
            "assignment_type": "assignment",
            "available_from": "2024-01-01T00:00:00Z",
            "due_date": "2024-01-31T23:59:59Z"
        });
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["data"]["name"], "New Assignment");
    }

    #[tokio::test]
    #[serial]
    async fn test_create_assignment_success_as_admin() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/assignments", data.module.id);
        let body = json!({
            "name": "Admin Assignment",
            "description": "Admin test assignment",
            "assignment_type": "practical",
            "available_from": "2024-02-01T00:00:00Z",
            "due_date": "2024-02-28T23:59:59Z"
        });
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["data"]["name"], "Admin Assignment");
    }

    #[tokio::test]
    #[serial]
    async fn test_create_assignment_forbidden_for_student() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!("/api/modules/{}/assignments", data.module.id);
        let body = json!({
            "name": "Student Assignment",
            "description": "Should not be allowed",
            "assignment_type": "assignment",
            "available_from": "2024-03-01T00:00:00Z",
            "due_date": "2024-03-31T23:59:59Z"
        });
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    #[serial]
    async fn test_create_assignment_forbidden_for_unassigned_user() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.forbidden_user.id, data.forbidden_user.admin);
        let uri = format!("/api/modules/{}/assignments", data.module.id);
        let body = json!({
            "name": "Forbidden Assignment",
            "description": "Should not be allowed",
            "assignment_type": "assignment",
            "available_from": "2024-04-01T00:00:00Z",
            "due_date": "2024-04-30T23:59:59Z"
        });
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    #[serial]
    async fn test_create_assignment_invalid_assignment_type() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments", data.module.id);
        let body = json!({
            "name": "Invalid Type",
            "description": "Invalid assignment_type",
            "assignment_type": "invalid_type",
            "available_from": "2024-05-01T00:00:00Z",
            "due_date": "2024-05-31T23:59:59Z"
        });
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert!(
            json["message"]
                .as_str()
                .unwrap()
                .contains("assignment_type")
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_create_assignment_invalid_dates() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments", data.module.id);
        let body = json!({
            "name": "Bad Dates",
            "description": "Invalid dates",
            "assignment_type": "assignment",
            "available_from": "not-a-date",
            "due_date": "also-not-a-date"
        });
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert!(json["message"].as_str().unwrap().contains("datetime"));
    }

    #[tokio::test]
    #[serial]
    async fn test_create_assignment_missing_fields() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments", data.module.id);
        let body = json!({
            "name": "Missing Fields"
        });
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn test_create_assignment_sets_default_config() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        // Create as lecturer (route calls AssignmentModel::create which writes default config)
        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let post_uri = format!("/api/modules/{}/assignments", data.module.id);
        let body = json!({
            "name": "API Created Assignment",
            "description": "should write default config",
            "assignment_type": "assignment",
            "available_from": "2024-01-01T00:00:00Z",
            "due_date": "2024-01-31T23:59:59Z"
        });

        let post_req = Request::builder()
            .method("POST")
            .uri(&post_uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();

        let post_resp = app.clone().oneshot(post_req).await.unwrap();
        assert_eq!(post_resp.status(), StatusCode::CREATED);
        let post_body = axum::body::to_bytes(post_resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let post_json: serde_json::Value = serde_json::from_slice(&post_body).unwrap();
        assert_eq!(post_json["success"], true);
        let assignment_id = post_json["data"]["id"].as_i64().expect("id > 0");
        assert!(assignment_id > 0);

        // GET /config to verify defaults and assert exact shape/values
        let get_uri = format!(
            "/api/modules/{}/assignments/{}/config",
            data.module.id, assignment_id
        );
        let get_req = Request::builder()
            .uri(&get_uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let get_resp = app.clone().oneshot(get_req).await.unwrap();
        assert_eq!(get_resp.status(), StatusCode::OK);
        let get_body = axum::body::to_bytes(get_resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let get_json: serde_json::Value = serde_json::from_slice(&get_body).unwrap();
        assert_eq!(get_json["success"], true);

        assert_default_config(&get_json["data"]);
    }
}
