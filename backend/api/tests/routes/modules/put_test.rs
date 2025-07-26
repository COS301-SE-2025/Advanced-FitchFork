#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use chrono::{Datelike, Utc};
    use db::{test_utils::setup_test_db, models::{module::Model as Module, user::Model as UserModel}};
    use serde_json::json;
    use tower::ServiceExt;
    use api::auth::generate_jwt;
    use crate::test_helpers::make_app;

    struct TestData {
        admin_user: UserModel,
        regular_user: UserModel,
        module: Module,
    }

    async fn setup_test_data(db: &sea_orm::DatabaseConnection) -> TestData {
        dotenvy::dotenv().expect("Failed to load .env");

        let admin_user = UserModel::create(db, "admin", "admin@test.com", "password", true).await.expect("Failed to create admin user");
        let regular_user = UserModel::create(db, "regular", "regular@test.com", "password", false).await.expect("Failed to create regular user");
        let module = Module::create(
            db,
            "COS301",
            Utc::now().year(),
            Some("Initial description"),
            16,
        )
        .await
        .expect("Failed to create test module");

        TestData {
            admin_user,
            regular_user,
            module,
        }
    }

    /// Test Case: Admin updates module successfully
    #[tokio::test]
    async fn test_edit_module_success() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let req_body = json!({"code": "COS302", "year": Utc::now().year() + 1, "description": "Updated description", "credits": 20});
        let uri = format!("/api/modules/{}", data.module.id);
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_vec(&req_body).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        
        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "Module updated successfully");
        let json_data = &json["data"];
        assert_eq!(json_data["code"], "COS302");
        assert_eq!(json_data["year"], Utc::now().year() + 1);
        assert_eq!(json_data["description"], "Updated description");
        assert_eq!(json_data["credits"], 20);
        assert_ne!(json_data["updated_at"].as_str().unwrap(), data.module.updated_at.to_rfc3339());
    }

    /// Test Case: Non-admin user attempts to update module
    #[tokio::test]
    async fn test_edit_module_forbidden() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.regular_user.id, data.regular_user.admin);
        let req_body = json!({"code": "COS302", "year": Utc::now().year() + 1, "description": "Updated description", "credits": 20});
        let uri = format!("/api/modules/{}", data.module.id);
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
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
    async fn test_edit_module_invalid_code() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let req_body = json!({"code": "abc123", "year": Utc::now().year() + 1, "description": "Updated description", "credits": 20});
        let uri = format!("/api/modules/{}", data.module.id);
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
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
    async fn test_edit_module_year_in_past() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let req_body = json!({"code": "COS302", "year": Utc::now().year() - 1, "description": "Updated description", "credits": 20});
        let uri = format!("/api/modules/{}", data.module.id);
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
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
    async fn test_edit_module_invalid_credits() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let req_body = json!({"code": "COS302", "year": Utc::now().year() + 1, "description": "Updated description", "credits": 0});
        let uri = format!("/api/modules/{}", data.module.id);
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
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
    async fn test_edit_module_description_too_long() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let req_body = json!({
            "code": "COS302",
            "year": Utc::now().year() + 1,
            "description": "a".repeat(1001),
            "credits": 20
        });
        let uri = format!("/api/modules/{}", data.module.id);
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
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
    async fn test_edit_module_duplicate_code() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let _other_module = Module::create(
            &db,
            "COS302",
            Utc::now().year(),
            Some("Other module"),
            16,
        )
        .await
        .expect("Failed to create second module");

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let req_body = json!({"code": "COS302", "year": Utc::now().year() + 1, "description": "Updated description", "credits": 20});
        let uri = format!("/api/modules/{}", data.module.id);
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_vec(&req_body).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::CONFLICT);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "Module code already exists");
    }

    /// Test Case: Update non-existent module
    #[tokio::test]
    async fn test_edit_module_not_found() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let req_body = json!({"code": "COS302", "year": Utc::now().year() + 1, "description": "Updated description", "credits": 20});
        let uri = format!("/api/modules/{}", 99999);
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_vec(&req_body).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "Module 99999 not found.");
    }

    /// Test Case: Multiple validation errors
    #[tokio::test]
    async fn test_edit_module_multiple_errors() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let req_body = json!({"code": "invalid", "year": 2000, "description": "a".repeat(1001), "credits": 0});
        let uri = format!("/api/modules/{}", data.module.id);
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
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
        assert!(message.contains("Year must be current year or later"));
        assert!(message.contains("Description must be at most 1000 characters"));
        assert!(message.contains("Credits must be a positive number"));
    }

    /// Test Case: Update with same code (should succeed)
    #[tokio::test]
    async fn test_edit_module_same_code() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let req_body = json!({"code": "COS301", "year": Utc::now().year() + 1, "description": "Updated description", "credits": 20});
        let uri = format!("/api/modules/{}", data.module.id);
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_vec(&req_body).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["data"]["year"], Utc::now().year() + 1);
    }
}