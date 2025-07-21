#[cfg(test)]
mod tests {
    use db::{test_utils::setup_test_db, models::{user::Model as UserModel, module::Model as ModuleModel, user_module_role::{Model as UserModuleRoleModel, Role}}};
    use axum::{body::Body, http::{Request, StatusCode}};
    use tower::ServiceExt;
    use serde_json::Value;
    use api::{routes::routes, auth::generate_jwt};
    use dotenvy;

    struct TestData {
        admin_user: UserModel,
        forbidden_user: UserModel,
        lecturer1: UserModel,
        module: ModuleModel,
    }

    async fn setup_test_data(db: &sea_orm::DatabaseConnection) -> TestData {
        let module = ModuleModel::create(db, "COS101", 2024, Some("Test Module"), 16).await.unwrap();
        let admin_user = UserModel::create(db, "admin1", "admin1@test.com", "password", true).await.unwrap();
        let forbidden_user = UserModel::create(db, "unauthed", "unauthed@test.com", "password", false).await.unwrap();
        let lecturer1 = UserModel::create(db, "lecturer1", "lecturer1@test.com", "password1", false).await.unwrap();
        let lecturer2 = UserModel::create(db, "lecturer2", "lecturer2@test.com", "password2", false).await.unwrap();
        let lecturer3 = UserModel::create(db, "lecturer3", "lecturer3@test.com", "password3", false).await.unwrap();
        UserModuleRoleModel::assign_user_to_module(db, lecturer1.id, module.id, Role::Lecturer).await.unwrap();
        UserModuleRoleModel::assign_user_to_module(db, lecturer2.id, module.id, Role::Lecturer).await.unwrap();
        UserModuleRoleModel::assign_user_to_module(db, lecturer3.id, module.id, Role::Lecturer).await.unwrap();

        TestData {
            admin_user,
            forbidden_user,
            lecturer1,
            module,
        }
    }

    /// Test Case: Successful Retrieval of Lecturers as Admin
    #[tokio::test]
    async fn test_get_lecturers_success_as_admin() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = axum::Router::new().nest("/api", routes(db.clone())).with_state(db);
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/lecturers", data.module.id);
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["data"]["total"], 3);
        let usernames: Vec<_> = json["data"]["users"].as_array().unwrap().iter().map(|u| u["username"].as_str().unwrap()).collect();
        assert!(usernames.contains(&"lecturer1"));
        assert!(usernames.contains(&"lecturer2"));
        assert!(usernames.contains(&"lecturer3"));
    }

    /// Test Case: Successful Retrieval of Lecturers as Lecturer
    #[tokio::test]
    async fn test_get_lecturers_success_as_lecturer() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = axum::Router::new().nest("/api", routes(db.clone())).with_state(db);
        let (token, _) = generate_jwt(data.lecturer1.id, data.lecturer1.admin);
        let uri = format!("/api/modules/{}/lecturers", data.module.id);
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["data"]["total"], 3);
        let usernames: Vec<_> = json["data"]["users"].as_array().unwrap().iter().map(|u| u["username"].as_str().unwrap()).collect();
        assert!(usernames.contains(&"lecturer1"));
        assert!(usernames.contains(&"lecturer2"));
        assert!(usernames.contains(&"lecturer3"));
    }

    /// Test Case: Module Not Found
    #[tokio::test]
    async fn test_get_lecturers_not_found() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = axum::Router::new().nest("/api", routes(db.clone())).with_state(db);
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/lecturers", 9999);
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "Module not found");
    }

    /// Test Case: Forbidden Access
    #[tokio::test]
    async fn test_get_lecturers_forbidden() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = axum::Router::new().nest("/api", routes(db.clone())).with_state(db);
        let (token, _) = generate_jwt(data.forbidden_user.id, data.forbidden_user.admin);
        let uri = format!("/api/modules/{}/lecturers", data.module.id);
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    /// Test Case: Filtering and Sorting
    #[tokio::test]
    async fn test_get_lecturers_with_query_params() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = axum::Router::new().nest("/api", routes(db.clone())).with_state(db);
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/lecturers?query=lecturer&sort=-email", data.module.id);
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["data"]["total"], 3);
        let emails: Vec<_> = json["data"]["users"].as_array().unwrap().iter().map(|u| u["email"].as_str().unwrap()).collect();
        let mut expected = vec!["lecturer3@test.com", "lecturer2@test.com", "lecturer1@test.com"];
        expected.sort_by(|a, b| b.cmp(a));
        assert_eq!(emails, expected);
    }

    /// Test Case: Pagination
    #[tokio::test]
    async fn test_get_lecturers_pagination() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;
        
        let app = axum::Router::new().nest("/api", routes(db.clone())).with_state(db);
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/lecturers?page=2&per_page=2", data.module.id);
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["data"]["total"], 3);
        assert_eq!(json["data"]["page"], 2);
        assert_eq!(json["data"]["per_page"], 2);
        let users = json["data"]["users"].as_array().unwrap();
        assert_eq!(users.len(), 1);
        assert_eq!(users[0]["username"], "lecturer3");
    }
}