#[cfg(test)]
mod tests {
    use crate::helpers::app::make_test_app_with_storage;
    use api::auth::generate_jwt;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use db::models::{
        module::Model as ModuleModel,
        user::Model as UserModel,
        user_module_role::{Model as UserModuleRole, Role},
    };
    use serde_json::json;
    use tower::ServiceExt;

    struct TestData {
        student: UserModel,
        lecturer: UserModel,
        invalid_user: UserModel,
        module: ModuleModel,
    }

    async fn setup_test_data(db: &sea_orm::DatabaseConnection) -> TestData {
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

        let module = ModuleModel::create(db, "330", 2025, Some("Test module"), 16)
            .await
            .unwrap();

        UserModuleRole::assign_user_to_module(db, student.id, module.id, Role::Student)
            .await
            .unwrap();

        UserModuleRole::assign_user_to_module(db, lecturer.id, module.id, Role::Lecturer)
            .await
            .unwrap();

        TestData {
            student,
            lecturer,
            invalid_user,
            module,
        }
    }

    #[tokio::test]
    async fn create_announcement_success() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;
        let (token, _) = generate_jwt(data.lecturer.id, data.lecturer.admin);

        let uri = format!("/api/modules/{}/announcements", data.module.id);
        let body = json!({
            "title": "Exam Schedule",
            "body": "The exam will be held next Friday at 9AM.",
            "pinned": true
        });

        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(json["message"], "Announcement created successfully");
        assert_eq!(json["data"]["title"], "Exam Schedule");
    }

    #[tokio::test]
    async fn create_announcement_unauthorized_no_token() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let uri = format!("/api/modules/{}/announcements", data.module.id);
        let body = json!({
            "title": "Exam Schedule",
            "body": "Exam date announced.",
            "pinned": false
        });

        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Content-Type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn create_announcement_forbidden_student() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;
        let (token, _) = generate_jwt(data.student.id, data.student.admin);

        let uri = format!("/api/modules/{}/announcements", data.module.id);
        let body = json!({
            "title": "Unauthorized attempt",
            "body": "Student trying to post",
            "pinned": false
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
    async fn create_announcement_forbidden_invalid_user_not_in_module() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;
        let (token, _) = generate_jwt(data.invalid_user.id, data.invalid_user.admin);

        let uri = format!("/api/modules/{}/announcements", data.module.id);
        let body = json!({
            "title": "Invalid user attempt",
            "body": "User not in module",
            "pinned": false
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
    async fn create_announcement_empty_body() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;
        let (token, _) = generate_jwt(data.lecturer.id, data.lecturer.admin);

        let uri = format!("/api/modules/{}/announcements", data.module.id);

        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(""))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn create_announcement_invalid_json() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;
        let (token, _) = generate_jwt(data.lecturer.id, data.lecturer.admin);

        let uri = format!("/api/modules/{}/announcements", data.module.id);

        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from("{ invalid json }"))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn create_announcement_missing_fields() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;
        let (token, _) = generate_jwt(data.lecturer.id, data.lecturer.admin);

        let uri = format!("/api/modules/{}/announcements", data.module.id);

        let body = json!({
            "title": "Missing body field"
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
}
