#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use chrono::{Datelike, Utc};
    use db::{models::user::Model as UserModel, repositories::user_repository::UserRepository};
    use services::{
        service::Service,
        user_service::{UserService, CreateUser}
    };
    use api::auth::generate_jwt;
    use crate::helpers::app::make_test_app;
    use serde_json::json;
    use tower::ServiceExt;

    struct TestData {
        admin_user: UserModel,
        regular_user: UserModel,
    }

    async fn setup_test_data(db: &sea_orm::DatabaseConnection) -> TestData {
        dotenvy::dotenv().expect("Failed to load .env"); 

        let service = UserService::new(UserRepository::new(db.clone()));
        let admin_user = service.create(CreateUser{ username: "admin".to_string(), email: "admin@test.com".to_string(), password: "password".to_string(), admin: true }).await.expect("Failed to create admin user");
        let regular_user = service.create(CreateUser{ username: "regular".to_string(), email: "regular@test.com".to_string(), password: "password".to_string(), admin: false }).await.expect("Failed to create regular user");

        TestData {
            admin_user,
            regular_user,
        }
    }

    /// Test Case: Admin creates module successfully
    #[tokio::test]
    async fn test_create_module_success() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let req_body = json!({"code": "COS301", "year": Utc::now().year(), "description": "Advanced Software Engineering", "credits": 16});
        let req = Request::builder()
            .method("POST")
            .uri("/api/modules")
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_vec(&req_body).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        
        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "Module created successfully");
        let data = &json["data"];
        assert_eq!(data["code"], "COS301");
        assert_eq!(data["year"], Utc::now().year());
        assert_eq!(data["description"], "Advanced Software Engineering");
        assert_eq!(data["credits"], 16);
        assert!(data["created_at"].as_str().is_some());
        assert!(data["updated_at"].as_str().is_some());
    }

    /// Test Case: Non-admin user attempts to create module
    #[tokio::test]
    async fn test_create_module_forbidden() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.regular_user.id, data.regular_user.admin);
        let req_body = json!({"code": "COS301", "year": Utc::now().year(), "credits": 16});
        let req = Request::builder()
            .method("POST")
            .uri("/api/modules")
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_vec(&req_body).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "Admin access required");
    }

    /// Test Case: Invalid module code format
    #[tokio::test]
    async fn test_create_module_invalid_code() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let req_body = json!({"code": "abc123", "year": Utc::now().year(), "credits": 16});
        let req = Request::builder()
            .method("POST")
            .uri("/api/modules")
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_vec(&req_body).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert!(json["message"].as_str().unwrap().contains("Module code must be in format ABC123"));
    }

    /// Test Case: Year in the past
    #[tokio::test]
    async fn test_create_module_year_in_past() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let current_year = Utc::now().year();
        let req_body = json!({"code": "COS301", "year": current_year - 1, "credits": 16});
        let req = Request::builder()
            .method("POST")
            .uri("/api/modules")
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_vec(&req_body).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert!(json["message"].as_str().unwrap().contains("Year must be current year or later"));
    }

    /// Test Case: Invalid credits value
    #[tokio::test]
    async fn test_create_module_invalid_credits() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let req_body = json!({"code": "COS301", "year": Utc::now().year(), "credits": 0});
        let req = Request::builder()
            .method("POST")
            .uri("/api/modules")
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_vec(&req_body).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert!(json["message"].as_str().unwrap().contains("Credits must be a positive number"));
    }

    /// Test Case: Description too long
    #[tokio::test]
    async fn test_create_module_description_too_long() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let long_description = "a".repeat(1001);
        let req_body = json!({"code": "COS301", "year": Utc::now().year(), "description": long_description, "credits": 16});
        let req = Request::builder()
            .method("POST")
            .uri("/api/modules")
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_vec(&req_body).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert!(json["message"].as_str().unwrap().contains("Description must be at most 1000 characters"));
    }

    /// Test Case: Duplicate module code
    #[tokio::test]
    async fn test_create_module_duplicate_code() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let req_body1 = json!({"code": "COS301", "year": Utc::now().year(), "credits": 16});
        let req1 = Request::builder()
            .method("POST")
            .uri("/api/modules")
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_vec(&req_body1).unwrap()))
            .unwrap();

        let response1 = app.clone().oneshot(req1).await.unwrap();
        assert_eq!(response1.status(), StatusCode::CREATED);

        let req_body2 = json!({"code": "COS301", "year": Utc::now().year(), "credits": 16});
        let req2 = Request::builder()
            .method("POST")
            .uri("/api/modules")
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_vec(&req_body2).unwrap()))
            .unwrap();

        let response2 = app.oneshot(req2).await.unwrap();
        assert_eq!(response2.status(), StatusCode::CONFLICT);

        let body = axum::body::to_bytes(response2.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "A module with this code already exists");
    }

    /// Test Case: Multiple validation errors
    #[tokio::test]
    async fn test_create_module_multiple_errors() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let req_body = json!({"code": "invalid", "year": 2000, "credits": 0});
        let req = Request::builder()
            .method("POST")
            .uri("/api/modules")
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_vec(&req_body).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let message = json["message"].as_str().unwrap();
        assert!(message.contains("Module code must be in format ABC123"));
        assert!(message.contains("Year must be"));
        assert!(message.contains("Credits must be a positive number"));
    }

    /// Test Case: Missing required fields
    #[tokio::test]
    async fn test_create_module_missing_fields() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let req_body = json!({"year": Utc::now().year()});
        let req = Request::builder()
            .method("POST")
            .uri("/api/modules")
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_vec(&req_body).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }
}