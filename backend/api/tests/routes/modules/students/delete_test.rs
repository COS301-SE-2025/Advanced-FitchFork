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
        student1: UserModel,
        student2: UserModel,
        outsider: UserModel,
    }

    async fn setup_test_data(db: &sea_orm::DatabaseConnection) -> TestData {
        let module = ModuleModel::create(db, "COS501", 2024, Some("Test Module"), 16).await.unwrap();
        let admin_user = UserModel::create(db, "admin1", "admin1@test.com", "password", true).await.unwrap();
        let forbidden_user = UserModel::create(db, "unauthed", "unauthed@test.com", "password", false).await.unwrap();
        let student1 = UserModel::create(db, "student1", "student1@test.com", "password1", false).await.unwrap();
        let student2 = UserModel::create(db, "student2", "student2@test.com", "password2", false).await.unwrap();
        let outsider = UserModel::create(db, "outsider", "outsider@test.com", "password3", false).await.unwrap();
        UserModuleRoleModel::assign_user_to_module(db, student1.id, module.id, Role::Student).await.unwrap();
        UserModuleRoleModel::assign_user_to_module(db, student2.id, module.id, Role::Student).await.unwrap();

        TestData {
            admin_user,
            forbidden_user,
            module,
            student1,
            student2,
            outsider,
        }
    }

    /// Test Case 1: Successful removal of students by admin
    #[tokio::test]
    async fn test_delete_students_success_as_admin() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = axum::Router::new().nest("/api", routes(db.clone())).with_state(db.clone());
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/students", data.module.id);
        let req = Request::builder()
            .uri(&uri)
            .method("DELETE")
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(json!({ "user_ids": [data.student1.id, data.student2.id] }).to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "Students removed from module successfully");
        let roles = UserModuleRoleModel::get_users_by_module_role(&db, data.module.id as i32, Role::Student).await.unwrap();
        let ids: Vec<_> = roles.iter().map(|r| r.user_id).collect();
        assert!(!ids.contains(&data.student1.id));
        assert!(!ids.contains(&data.student2.id));
    }

    /// Test Case 2: Forbidden for non-admin users
    #[tokio::test]
    async fn test_delete_students_forbidden_for_non_admin() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = axum::Router::new().nest("/api", routes(db.clone())).with_state(db.clone());
        let (token, _) = generate_jwt(data.forbidden_user.id, data.forbidden_user.admin);
        let uri = format!("/api/modules/{}/students", data.module.id);
        let req = Request::builder()
            .uri(&uri)
            .method("DELETE")
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(json!({ "user_ids": [data.student1.id] }).to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    /// Test Case 3: Module not found
    #[tokio::test]
    async fn test_delete_students_module_not_found() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = axum::Router::new().nest("/api", routes(db.clone())).with_state(db.clone());
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/students", 9999);
        let req = Request::builder()
            .uri(&uri)
            .method("DELETE")
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(json!({ "user_ids": [data.student1.id] }).to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "Module not found");
    }

    /// Test Case 4: User not found
    #[tokio::test]
    async fn test_delete_students_user_not_found() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = axum::Router::new().nest("/api", routes(db.clone())).with_state(db.clone());
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/students", data.module.id);
        let req = Request::builder()
            .uri(&uri)
            .method("DELETE")
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(json!({ "user_ids": [9999] }).to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], format!("User with ID {} does not exist", 9999));
    }

    /// Test Case 5: User not a student in the module (conflict)
    #[tokio::test]
    async fn test_delete_students_user_not_a_student() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = axum::Router::new().nest("/api", routes(db.clone())).with_state(db.clone());
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/students", data.module.id);
        let req = Request::builder()
            .uri(&uri)
            .method("DELETE")
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(json!({ "user_ids": [data.student1.id, data.outsider.id] }).to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::CONFLICT);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "Some users are not students for this module");
    }

    /// Test Case 6: Empty user_ids list (bad request)
    #[tokio::test]
    async fn test_delete_students_empty_user_list() {
        dotenvy::dotenv().expect("Failed to load .env");
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = axum::Router::new().nest("/api", routes(db.clone())).with_state(db.clone());
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}/students", data.module.id);
        let req = Request::builder()
            .uri(&uri)
            .method("DELETE")
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(json!({ "user_ids": [] }).to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "Request must include a non-empty list of user_ids");
    }
}