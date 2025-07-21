#[cfg(test)]
mod tests {
    use axum::{body::Body, http::{self, Request, StatusCode}};
    use tower::ServiceExt;
    use serde_json::{json, Value};
    use api::{routes::routes, auth::generate_jwt};
    use db::{test_utils::setup_test_db, models::{user::Model as UserModel, module::Model as ModuleModel, user_module_role::{Model as UserModuleRoleModel, Role}}};
    use dotenvy;

    struct TestData {
        admin_user: UserModel,
        module: ModuleModel,
        lecturer1: UserModel,
        lecturer2: UserModel,
        student_user: UserModel,
    }

    async fn setup_test_data(db: &sea_orm::DatabaseConnection) -> TestData {
        let admin_user = UserModel::create(db, "admin", "admin@test.com", "password", true).await.unwrap();
        let module = ModuleModel::create(db, "COS301", 2025, Some("Test Module"), 16).await.unwrap();
        let lecturer1 = UserModel::create(db, "lecturer1", "lecturer1@test.com", "password", false).await.unwrap();
        let lecturer2 = UserModel::create(db, "lecturer2", "lecturer2@test.com", "password", false).await.unwrap();
        let student_user = UserModel::create(db, "student", "student@test.com", "password", false).await.unwrap();
        UserModuleRoleModel::assign_user_to_module(db, lecturer1.id, module.id, Role::Lecturer).await.unwrap();
        UserModuleRoleModel::assign_user_to_module(db, lecturer2.id, module.id, Role::Lecturer).await.unwrap();
        UserModuleRoleModel::assign_user_to_module(db, student_user.id, module.id, Role::Student).await.unwrap();
        
        TestData { admin_user, module, lecturer1, lecturer2, student_user }
    }

    /// Test Case: Successful removal of lecturers by an admin
    #[tokio::test]
    async fn test_remove_lecturers_success_as_admin() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = axum::Router::new().nest("/api", routes(db.clone())).with_state(db);
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/lecturers", data.module.id);
        let req = Request::builder()
            .uri(&uri)
            .method(http::Method::DELETE)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(json!({ "user_ids": [data.lecturer1.id] }).to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "Lecturers removed from module successfully");
    }

    /// Test Case: Forbidden for non-admin users
    #[tokio::test]
    async fn test_remove_lecturers_forbidden_for_non_admin() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = axum::Router::new().nest("/api", routes(db.clone())).with_state(db);
        let (token, _) = generate_jwt(data.lecturer1.id, data.lecturer1.admin);
        let uri = format!("/api/modules/{}/lecturers", data.module.id);
        let req = Request::builder()
            .uri(&uri)
            .method(http::Method::DELETE)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(json!({ "user_ids": [data.lecturer2.id] }).to_string()))
            .unwrap();
            
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
    
    /// Test Case: Module not found
    #[tokio::test]
    async fn test_remove_lecturers_module_not_found() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = axum::Router::new().nest("/api", routes(db.clone())).with_state(db);
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/lecturers", 9999);
        let req = Request::builder()
            .uri(&uri)
            .method(http::Method::DELETE)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(json!({ "user_ids": [data.lecturer1.id] }).to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
    
    /// Test Case: User to be removed not found
    #[tokio::test]
    async fn test_remove_lecturers_user_not_found() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = axum::Router::new().nest("/api", routes(db.clone())).with_state(db);
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/lecturers", data.module.id);
        let req = Request::builder()
            .uri(&uri)
            .method(http::Method::DELETE)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(json!({ "user_ids": [9999] }).to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
    
    /// Test Case: Attempting to remove a user who is not a lecturer
    #[tokio::test]
    async fn test_remove_lecturers_user_not_a_lecturer() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = axum::Router::new().nest("/api", routes(db.clone())).with_state(db);
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/lecturers", data.module.id);
        let req = Request::builder()
            .uri(&uri)
            .method(http::Method::DELETE)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(json!({ "user_ids": [data.student_user.id] }).to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
    
    /// Test Case: Empty user_ids list
    #[tokio::test]
    async fn test_remove_lecturers_empty_user_list() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = axum::Router::new().nest("/api", routes(db.clone())).with_state(db);
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/lecturers", data.module.id);
        let req = Request::builder()
            .uri(&uri)
            .method(http::Method::DELETE)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(json!({ "user_ids": [] }).to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}