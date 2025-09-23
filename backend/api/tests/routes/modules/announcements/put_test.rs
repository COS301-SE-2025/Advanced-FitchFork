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
    use serde_json::json;
    use tower::ServiceExt;

    struct TestData {
        student: UserModel,
        invalid_user: UserModel,
        lecturer: UserModel,
        module: ModuleModel,
        announcement: AnnouncementModel,
    }

    async fn setup_test_data(db: &sea_orm::DatabaseConnection) -> TestData {
        let student = UserModel::create(db, "test_user", "test@example.com", "pass", false)
            .await
            .unwrap();

        let invalid_user =
            UserModel::create(db, "invalid_user", "invalid@email.com", "pass2", false)
                .await
                .unwrap();

        let lecturer =
            UserModel::create(db, "lecturer_user", "lecturer@example.com", "pass3", true)
                .await
                .unwrap();

        let module = ModuleModel::create(db, "330", 2025, Some("test description"), 16)
            .await
            .unwrap();

        UserModuleRole::assign_user_to_module(db, student.id, module.id, Role::Student)
            .await
            .unwrap();

        UserModuleRole::assign_user_to_module(db, lecturer.id, module.id, Role::Lecturer)
            .await
            .unwrap();

        let announcement = AnnouncementModel::create(
            db,
            module.id,
            lecturer.id,
            "Test Announcement",
            "This is a test announcement.",
            false,
        )
        .await
        .unwrap();

        TestData {
            student,
            invalid_user,
            lecturer,
            module,
            announcement,
        }
    }

    #[tokio::test]
    async fn update_announcement_success() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer.id, data.lecturer.admin);
        let uri = format!(
            "/api/modules/{}/announcements/{}",
            data.module.id, data.announcement.id
        );

        let body_json = json!({
            "title": "Updated deadline",
            "body": "The assignment is now due Monday 9AM.",
            "pinned": true
        });

        let req = Request::builder()
            .method("PUT")
            .uri(uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(body_json.to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        let status = response.status();
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();

        assert_eq!(status, StatusCode::OK);

        let response_data: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(response_data["data"]["title"], "Updated deadline");
        assert_eq!(
            response_data["data"]["body"],
            "The assignment is now due Monday 9AM."
        );
        assert_eq!(response_data["data"]["pinned"], true);
        assert_eq!(
            response_data["message"],
            "Announcement updated successfully"
        );
    }

    #[tokio::test]
    async fn update_announcement_forbidden_student() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.student.id, data.student.admin);
        let uri = format!(
            "/api/modules/{}/announcements/{}",
            data.module.id, data.announcement.id
        );

        let body_json = json!({
            "title": "Should not work",
            "body": "No permission.",
            "pinned": false
        });

        let req = Request::builder()
            .method("PUT")
            .uri(uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(body_json.to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        let status = response.status();
        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn update_announcement_forbidden_invalid_user() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.invalid_user.id, data.invalid_user.admin);
        let uri = format!(
            "/api/modules/{}/announcements/{}",
            data.module.id, data.announcement.id
        );

        let body_json = json!({
            "title": "Should not work",
            "body": "Invalid user no access",
            "pinned": false
        });

        let req = Request::builder()
            .method("PUT")
            .uri(uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(body_json.to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn update_announcement_empty_body() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer.id, data.lecturer.admin);
        let uri = format!(
            "/api/modules/{}/announcements/{}",
            data.module.id, data.announcement.id
        );

        let req = Request::builder()
            .method("PUT")
            .uri(uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(""))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn update_announcement_invalid_json() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer.id, data.lecturer.admin);
        let uri = format!(
            "/api/modules/{}/announcements/{}",
            data.module.id, data.announcement.id
        );

        let body = json!({
            "title": "Missing body field"
        });

        let req = Request::builder()
            .method("PUT")
            .uri(uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn update_nonexistent_announcement() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer.id, data.lecturer.admin);
        let uri = format!("/api/modules/{}/announcements/999", data.module.id);

        let body_json = json!({
            "title": "Nonexistent",
            "body": "Trying to update non-existent announcement",
            "pinned": false
        });

        let req = Request::builder()
            .method("PUT")
            .uri(uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(body_json.to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        let status = response.status();
        assert_eq!(status, StatusCode::NOT_FOUND);
    }
}
