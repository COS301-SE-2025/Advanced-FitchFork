#[cfg(test)]
mod tests {
    use axum::{body::Body, http::{Request, StatusCode}};
    use tower::ServiceExt;
    use serde_json::json;
    use api::auth::generate_jwt;
    use db::{
        models::{
            user::Model as UserModel,
            module::Model as ModuleModel,
            user_module_role::{Model as UserModuleRoleModel, Role},
        },
    };
    use crate::helpers::app::make_test_app;
    use services::{
        service::Service,
        user_service::{UserService, CreateUser}
    };
    use db::repositories::user_repository::UserRepository;
    use serial_test::serial;

    struct TestData {
        admin: UserModel,
        lecturer: UserModel,
        student: UserModel,
        outsider: UserModel,
        module: ModuleModel,
    }

    async fn setup_data(db: &sea_orm::DatabaseConnection) -> TestData {
        let module = ModuleModel::create(db, "COS999", 2025, Some("Test Module"), 12).await.unwrap();
        let service = UserService::new(UserRepository::new(db.clone()));
        let admin = service.create(CreateUser{ username: "admin".to_string(), email: "admin@test.com".to_string(), password: "pw".to_string(), admin: true }).await.unwrap();
        let lecturer = service.create(CreateUser{ username: "lect1".to_string(), email: "lect@test.com".to_string(), password: "pw".to_string(), admin: false }).await.unwrap();
        let student = service.create(CreateUser{ username: "stud1".to_string(), email: "stud@test.com".to_string(), password: "pw".to_string(), admin: false }).await.unwrap();
        let outsider = service.create(CreateUser{ username: "outsider".to_string(), email: "out@test.com".to_string(), password: "pw".to_string(), admin: false }).await.unwrap();
        UserModuleRoleModel::assign_user_to_module(db, lecturer.id, module.id, Role::Lecturer).await.unwrap();
        UserModuleRoleModel::assign_user_to_module(db, student.id, module.id, Role::Student).await.unwrap();
        TestData { admin, lecturer, student, outsider, module }
    }

    #[tokio::test]
    #[serial]
    async fn assign_personnel_as_admin_success() {
        let app = make_test_app().await;
        let data = setup_data(db::get_connection().await).await;

        let (token, _) = generate_jwt(data.admin.id, data.admin.admin);
        let uri = format!("/api/modules/{}/personnel", data.module.id);

        let body = serde_json::json!({
            "user_ids": [data.outsider.id],
            "role": "tutor"
        });

        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        let status = res.status();
        let _ = axum::body::to_bytes(res.into_body(), 1024 * 1024).await.unwrap(); // consume body for completeness

        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    #[serial]
    async fn assign_personnel_as_lecturer_success_for_non_lecturer_roles() {
        let app = make_test_app().await;
        let data = setup_data(db::get_connection().await).await;

        let (token, _) = generate_jwt(data.lecturer.id, data.lecturer.admin);
        let uri = format!("/api/modules/{}/personnel", data.module.id);

        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(json!({ "user_ids": [data.outsider.id], "role": "assistant_lecturer" }).to_string()))
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    #[serial]
    async fn assign_lecturer_role_as_lecturer_forbidden() {
        let app = make_test_app().await;
        let data = setup_data(db::get_connection().await).await;

        let (token, _) = generate_jwt(data.lecturer.id, data.lecturer.admin);
        let uri = format!("/api/modules/{}/personnel", data.module.id);

        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(json!({ "user_ids": [data.outsider.id], "role": "lecturer" }).to_string()))
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    #[serial]
    async fn assign_personnel_as_non_lecturer_forbidden() {
        let app = make_test_app().await;
        let data = setup_data(db::get_connection().await).await;

        let (token, _) = generate_jwt(data.outsider.id, false);
        let uri = format!("/api/modules/{}/personnel", data.module.id);

        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(json!({ "user_ids": [data.lecturer.id], "role": "tutor" }).to_string()))
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    #[serial]
    async fn assign_personnel_as_student_forbidden() {
        let app = make_test_app().await;
        let data = setup_data(db::get_connection().await).await;

        // Student is assigned to the module, but still shouldn't have assign permissions
        let (token, _) = generate_jwt(data.student.id, false);
        let uri = format!("/api/modules/{}/personnel", data.module.id);

        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(json!({ "user_ids": [data.outsider.id], "role": "tutor" }).to_string()))
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    #[serial]
    async fn assign_personnel_user_not_found() {
        let app = make_test_app().await;
        let data = setup_data(db::get_connection().await).await;

        let (token, _) = generate_jwt(data.admin.id, true);
        let uri = format!("/api/modules/{}/personnel", data.module.id);

        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(json!({ "user_ids": [9999999], "role": "student" }).to_string()))
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    #[serial]
    async fn assign_personnel_empty_user_ids() {
        let app = make_test_app().await;
        let data = setup_data(db::get_connection().await).await;

        let (token, _) = generate_jwt(data.admin.id, true);
        let uri = format!("/api/modules/{}/personnel", data.module.id);

        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(json!({ "user_ids": [], "role": "student" }).to_string()))
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    }
}
