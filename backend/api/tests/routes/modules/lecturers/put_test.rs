#[cfg(test)]
mod tests {
    use db::{test_utils::setup_test_db, models::{user::Model as UserModel, module::Model as ModuleModel, user_module_role::{Model as UserModuleRoleModel, Role}}};
    use axum::{body::Body, http::{Request, StatusCode}};
    use tower::ServiceExt;
    use serde_json::{json, Value};
    use api::{routes::routes, auth::generate_jwt};
    use dotenvy;

    struct TestData {
        admin_user: UserModel,
        forbidden_user: UserModel,
        module: ModuleModel,
        lecturer1: UserModel,
        lecturer2: UserModel,
        student1: UserModel,
        outsider: UserModel,
    }

    async fn setup_test_data(db: &sea_orm::DatabaseConnection) -> TestData {
        let module = ModuleModel::create(db, "COS101", 2024, Some("Test Module"), 16).await.unwrap();
        let admin_user = UserModel::create(db, "admin1", "admin1@test.com", "password", true).await.unwrap();
        let forbidden_user = UserModel::create(db, "unauthed", "unauthed@test.com", "password", false).await.unwrap();
        let lecturer1 = UserModel::create(db, "lecturer1", "lecturer1@test.com", "password1", false).await.unwrap();
        let lecturer2 = UserModel::create(db, "lecturer2", "lecturer2@test.com", "password2", false).await.unwrap();
        let student1 = UserModel::create(db, "student1", "student1@test.com", "password3", false).await.unwrap();
        let outsider = UserModel::create(db, "outsider", "outsider@test.com", "password4", false).await.unwrap();
        UserModuleRoleModel::assign_user_to_module(db, lecturer1.id, module.id, Role::Lecturer).await.unwrap();
        UserModuleRoleModel::assign_user_to_module(db, lecturer2.id, module.id, Role::Tutor).await.unwrap();
        UserModuleRoleModel::assign_user_to_module(db, student1.id, module.id, Role::Student).await.unwrap();

        TestData {
            admin_user,
            forbidden_user,
            module,
            lecturer1,
            lecturer2,
            student1,
            outsider,
        }
    }

    /// Test Case: Successful update of roles to Lecturer by admin
    #[tokio::test]
    async fn test_edit_lecturers_success_as_admin() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = axum::Router::new().nest("/api", routes(db.clone())).with_state(db);
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/lecturers", data.module.id);
        let req = Request::builder()
            .uri(&uri)
            .method("PUT")
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(json!({ "user_ids": [data.lecturer1.id, data.lecturer2.id, data.student1.id] }).to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "Users set as lecturers successfully");
    }

    /// Test Case: Forbidden for non-admin users
    #[tokio::test]
    async fn test_edit_lecturers_forbidden_for_non_admin() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = axum::Router::new().nest("/api", routes(db.clone())).with_state(db);
        let (token, _) = generate_jwt(data.forbidden_user.id, data.forbidden_user.admin);
        let uri = format!("/api/modules/{}/lecturers", data.module.id);
        let req = Request::builder()
            .uri(&uri)
            .method("PUT")
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(json!({ "user_ids": [data.lecturer1.id] }).to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    /// Test Case: Module not found
    #[tokio::test]
    async fn test_edit_lecturers_module_not_found() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = axum::Router::new().nest("/api", routes(db.clone())).with_state(db);
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/lecturers", 9999);
        let req = Request::builder()
            .uri(&uri)
            .method("PUT")
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(json!({ "user_ids": [data.lecturer1.id] }).to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "Module not found");
    }

    /// Test Case: User not found
    #[tokio::test]
    async fn test_edit_lecturers_user_not_found() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = axum::Router::new().nest("/api", routes(db.clone())).with_state(db);
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/lecturers", data.module.id);
        let req = Request::builder()
            .uri(&uri)
            .method("PUT")
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(json!({ "user_ids": [data.lecturer1.id, 9999] }).to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], format!("User with ID {} does not exist", 9999));
    }

    /// Test Case: User not assigned to module
    #[tokio::test]
    async fn test_edit_lecturers_user_not_assigned_to_module() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = axum::Router::new().nest("/api", routes(db.clone())).with_state(db);
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/lecturers", data.module.id);
        let req = Request::builder()
            .uri(&uri)
            .method("PUT")
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(json!({ "user_ids": [data.lecturer1.id, data.outsider.id] }).to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], format!("User with ID {} is not assigned to this module", data.outsider.id));
    }

    /// Test Case: Empty user_ids list
    #[tokio::test]
    async fn test_edit_lecturers_empty_user_list() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = axum::Router::new().nest("/api", routes(db.clone())).with_state(db);
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/lecturers", data.module.id);
        let req = Request::builder()
            .uri(&uri)
            .method("PUT")
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(json!({ "user_ids": [] }).to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert!(json["message"].as_str().unwrap().contains("non-empty"));
    }

    /// Test Case: Idempotency (all users already lecturers)
    #[tokio::test]
    async fn test_edit_lecturers_idempotency() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = axum::Router::new().nest("/api", routes(db.clone())).with_state(db.clone());
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/lecturers", data.module.id);
        let req = Request::builder()
            .uri(&uri)
            .method("PUT")
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(json!({ "user_ids": [data.lecturer1.id, data.lecturer2.id, data.student1.id] }).to_string()))
            .unwrap();

        let response = app.clone().oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let req2 = Request::builder()
            .uri(&uri)
            .method("PUT")
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(json!({ "user_ids": [data.lecturer1.id, data.lecturer2.id, data.student1.id] }).to_string()))
            .unwrap();

        let response2 = app.oneshot(req2).await.unwrap();
        assert_eq!(response2.status(), StatusCode::OK);
        
        let body2 = axum::body::to_bytes(response2.into_body(), usize::MAX).await.unwrap();
        let json2: Value = serde_json::from_slice(&body2).unwrap();
        assert_eq!(json2["success"], true);
        assert_eq!(json2["message"], "Users set as lecturers successfully");
    }
}