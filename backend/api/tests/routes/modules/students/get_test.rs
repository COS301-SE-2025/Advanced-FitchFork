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
        student1: UserModel,
        module: ModuleModel,
        empty_module: ModuleModel,
    }

    async fn setup_test_data(db: &sea_orm::DatabaseConnection) -> TestData {
        let module = ModuleModel::create(db, "COS201", 2024, Some("Test Module"), 16).await.unwrap();
        let empty_module = ModuleModel::create(db, "COS202", 2024, Some("Empty Module"), 16).await.unwrap();
        let admin_user = UserModel::create(db, "admin2", "admin2@test.com", "password", true).await.unwrap();
        let forbidden_user = UserModel::create(db, "unauthed2", "unauthed2@test.com", "password", false).await.unwrap();
        let student1 = UserModel::create(db, "student1", "student1@test.com", "password1", false).await.unwrap();
        let student2 = UserModel::create(db, "student2", "student2@test.com", "password2", false).await.unwrap();
        let student3 = UserModel::create(db, "student3", "student3@test.com", "password3", false).await.unwrap();
        UserModuleRoleModel::assign_user_to_module(db, student1.id, module.id, Role::Student).await.unwrap();
        UserModuleRoleModel::assign_user_to_module(db, student2.id, module.id, Role::Student).await.unwrap();
        UserModuleRoleModel::assign_user_to_module(db, student3.id, module.id, Role::Student).await.unwrap();

        TestData {
            admin_user,
            forbidden_user,
            student1,
            module,
            empty_module,
        }
    }

    /// Test Case 1: Successful Retrieval of Students as Admin
    #[tokio::test]
    async fn test_get_students_success_as_admin() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = axum::Router::new().nest("/api", routes(db.clone())).with_state(db);
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/students", data.module.id);
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
        assert!(usernames.contains(&"student1"));
        assert!(usernames.contains(&"student2"));
        assert!(usernames.contains(&"student3"));
    }

    /// Test Case 2: Successful Retrieval of Students as Student
    #[tokio::test]
    async fn test_get_students_success_as_student() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = axum::Router::new().nest("/api", routes(db.clone())).with_state(db);
        let (token, _) = generate_jwt(data.student1.id, data.student1.admin);
        let uri = format!("/api/modules/{}/students", data.module.id);
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
        assert!(usernames.contains(&"student1"));
        assert!(usernames.contains(&"student2"));
        assert!(usernames.contains(&"student3"));
    }

    /// Test Case 3: Module Not Found
    #[tokio::test]
    async fn test_get_students_not_found() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = axum::Router::new().nest("/api", routes(db.clone())).with_state(db);
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/students", 9999);
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

    /// Test Case 4: Forbidden Access
    #[tokio::test]
    async fn test_get_students_forbidden() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = axum::Router::new().nest("/api", routes(db.clone())).with_state(db);
        let (token, _) = generate_jwt(data.forbidden_user.id, data.forbidden_user.admin);
        let uri = format!("/api/modules/{}/students", data.module.id);
        let req = Request::builder()
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    /// Test Case 5: Filtering and Sorting
    #[tokio::test]
    async fn test_get_students_with_query_params() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = axum::Router::new().nest("/api", routes(db.clone())).with_state(db);
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/students?query=student&sort=-email", data.module.id);
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
        let mut expected = vec!["student3@test.com", "student2@test.com", "student1@test.com"];
        expected.sort_by(|a, b| b.cmp(a));
        assert_eq!(emails, expected);
    }

    /// Test Case 6: Pagination
    #[tokio::test]
    async fn test_get_students_pagination() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = axum::Router::new().nest("/api", routes(db.clone())).with_state(db);
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/students?page=2&per_page=2", data.module.id);
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
        let usernames: Vec<_> = users.iter().map(|u| u["username"].as_str().unwrap()).collect();
        assert!(usernames[0] == "student3" || usernames[0] == "student2" || usernames[0] == "student1");
    }

    /// Test Case 7: Edge Case - No Students in Module
    #[tokio::test]
    async fn test_get_students_empty_module() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;
        
        let app = axum::Router::new().nest("/api", routes(db.clone())).with_state(db);
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/students", data.empty_module.id);
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
        assert_eq!(json["data"]["total"], 0);
        assert_eq!(json["data"]["users"].as_array().unwrap().len(), 0);
    }
}