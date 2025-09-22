#[cfg(test)]
mod tests {
    use crate::helpers::app::make_test_app_with_storage;
    use api::auth::generate_jwt;
    use db::{
        models::{
            announcements::Model as AnnouncementModel,
            module::Model as ModuleModel,
            user::Model as UserModel,
            user_module_role::{Model as UserModuleRole, Role},
        },
        repositories::user_repository::UserRepository,
    };
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use services::{
        service::Service,
        user::{CreateUser, UserService},
    };
    use tower::ServiceExt;
    use serial_test::serial;

    struct TestData {
        student: UserModel,
        invalid_user: UserModel,
        lecturer: UserModel,
        module: ModuleModel,
        announcement: AnnouncementModel,
    }

    async fn setup_test_data(db: &sea_orm::DatabaseConnection) -> TestData {
        let service = UserService::new(UserRepository::new(db.clone()));
        
        let student = service
            .create(CreateUser {
                username: "test_user".into(),
                email: "test@example.com".into(),
                password: "pass".into(),
                admin: false,
            })
            .await
            .unwrap();

        let invalid_user = service
            .create(CreateUser {
                username: "invalid_user".into(),
                email: "invalid@email.com".into(),
                password: "pass2".into(),
                admin: false,
            })
            .await
            .unwrap();

        let lecturer = service
            .create(CreateUser {
                username: "lecturer_user".into(),
                email: "lecturer@example.com".into(),
                password: "pass3".into(),
                admin: true,
            })
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
    #[serial]
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
    #[serial]
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
    #[serial]
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
    #[serial]
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
