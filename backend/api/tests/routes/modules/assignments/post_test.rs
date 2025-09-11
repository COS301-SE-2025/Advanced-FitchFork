#[cfg(test)]
mod tests {
    use db::{
        models::{
            user::Model as UserModel,
            module::Model as ModuleModel,
            user_module_role::{Model as UserModuleRoleModel, Role}
        },
        repositories::user_repository::UserRepository,
    };
    use axum::{
        body::Body,
        http::{Request, StatusCode}
    };
    use services::{
        service::Service,
        user::{UserService, CreateUser},
    };
    use tower::ServiceExt;
    use serde_json::json;
    use api::auth::generate_jwt;
    use crate::helpers::app::make_test_app;
    use serial_test::serial;

    struct TestData {
        admin_user: UserModel,
        lecturer_user: UserModel,
        student_user: UserModel,
        forbidden_user: UserModel,
        module: ModuleModel,
    }

    async fn setup_test_data(db: &sea_orm::DatabaseConnection) -> TestData {
        let module = ModuleModel::create(db, "COS101", 2024, Some("Test Module"), 16).await.unwrap();
        let service = UserService::new(UserRepository::new(db.clone()));
        let admin_user = service.create(CreateUser { username: "admin1".to_string(), email: "admin1@test.com".to_string(), password: "password".to_string(), admin: true }).await.unwrap();
        let lecturer_user = service.create(CreateUser { username: "lecturer1".to_string(), email: "lecturer1@test.com".to_string(), password: "password1".to_string(), admin: false }).await.unwrap();
        let student_user = service.create(CreateUser { username: "student1".to_string(), email: "student1@test.com".to_string(), password: "password2".to_string(), admin: false }).await.unwrap();
        let forbidden_user = service.create(CreateUser { username: "forbidden".to_string(), email: "forbidden@test.com".to_string(), password: "password3".to_string(), admin: false }).await.unwrap();
        UserModuleRoleModel::assign_user_to_module(db, lecturer_user.id, module.id, Role::Lecturer).await.unwrap();
        UserModuleRoleModel::assign_user_to_module(db, student_user.id, module.id, Role::Student).await.unwrap();

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
        let app = make_test_app().await;
        let data = setup_test_data(db::get_connection().await).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments", data.module.id);
        let body = json!({
            "name": "New Assignment",
            "description": "A test assignment",
            "assignment_type": "assignment",
            "status": "setup",
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

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["data"]["name"], "New Assignment");
    }

    #[tokio::test]
    #[serial]
    async fn test_create_assignment_success_as_admin() {
        let app = make_test_app().await;
        let data = setup_test_data(db::get_connection().await).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/assignments", data.module.id);
        let body = json!({
            "name": "Admin Assignment",
            "description": "Admin test assignment",
            "assignment_type": "practical",
            "status": "setup",
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

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["data"]["name"], "Admin Assignment");
    }

    #[tokio::test]
    #[serial]
    async fn test_create_assignment_forbidden_for_student() {
        let app = make_test_app().await;
        let data = setup_test_data(db::get_connection().await).await;

        let (token, _) = generate_jwt(data.student_user.id, data.student_user.admin);
        let uri = format!("/api/modules/{}/assignments", data.module.id);
        let body = json!({
            "name": "Student Assignment",
            "description": "Should not be allowed",
            "assignment_type": "assignment",
            "status": "setup",
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
        let app = make_test_app().await;
        let data = setup_test_data(db::get_connection().await).await;

        let (token, _) = generate_jwt(data.forbidden_user.id, data.forbidden_user.admin);
        let uri = format!("/api/modules/{}/assignments", data.module.id);
        let body = json!({
            "name": "Forbidden Assignment",
            "description": "Should not be allowed",
            "assignment_type": "assignment",
            "status": "setup",
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
        let app = make_test_app().await;
        let data = setup_test_data(db::get_connection().await).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments", data.module.id);
        let body = json!({
            "name": "Invalid Type",
            "description": "Invalid assignment_type",
            "assignment_type": "invalid_type",
            "status": "setup",
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

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert!(json["message"].as_str().unwrap().contains("assignment_type"));
    }

    #[tokio::test]
    #[serial]
    async fn test_create_assignment_invalid_dates() {
        let app = make_test_app().await;
        let data = setup_test_data(db::get_connection().await).await;

        let (token, _) = generate_jwt(data.lecturer_user.id, data.lecturer_user.admin);
        let uri = format!("/api/modules/{}/assignments", data.module.id);
        let body = json!({
            "name": "Bad Dates",
            "description": "Invalid dates",
            "assignment_type": "assignment",
            "status": "setup",
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

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert!(json["message"].as_str().unwrap().contains("datetime"));
    }

    #[tokio::test]
    #[serial]
    async fn test_create_assignment_missing_fields() {
        let app = make_test_app().await;
        let data = setup_test_data(db::get_connection().await).await;

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
        assert!(response.status() == StatusCode::UNPROCESSABLE_ENTITY);
    }
}