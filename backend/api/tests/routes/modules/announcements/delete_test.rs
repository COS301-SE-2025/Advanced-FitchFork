#[cfg(test)]
mod tests {
    use crate::helpers::app::make_test_app_with_storage;
    use api::auth::generate_jwt;
    use db::models::{
        announcements::Model as AnnouncementModel,
        module::Model as ModuleModel,
        user::Model as UserModel,
        user_module_role::{Model as UserModuleRole, Role},
    };
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
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
    async fn delete_announcement_success() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer.id, data.lecturer.admin);
        let uri = format!(
            "/api/modules/{}/announcements/{}",
            data.module.id, data.announcement.id
        );

        let req = Request::builder()
            .method("DELETE")
            .uri(uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn delete_announcement_forbidden_student() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.student.id, data.student.admin);
        let uri = format!(
            "/api/modules/{}/announcements/{}",
            data.module.id, data.announcement.id
        );

        let req = Request::builder()
            .method("DELETE")
            .uri(uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn delete_announcement_forbidden_invalid_user() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.invalid_user.id, data.invalid_user.admin);
        let uri = format!(
            "/api/modules/{}/announcements/{}",
            data.module.id, data.announcement.id
        );

        let req = Request::builder()
            .method("DELETE")
            .uri(uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn delete_nonexistent_announcement() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.lecturer.id, data.lecturer.admin);
        let uri = format!(
            "/api/modules/{}/announcements/999",
            data.module.id
        );

        let req = Request::builder()
            .method("DELETE")
            .uri(uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
