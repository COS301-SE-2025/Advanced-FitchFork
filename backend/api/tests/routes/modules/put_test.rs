#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use chrono::{Datelike, Utc};
    use db::{models::{module::Model as Module, user::Model as UserModel, module::Entity as ModuleEntity,}, repositories::user_repository::UserRepository};
    use services::{
        service::Service,
        user::{UserService, CreateUser}
    };
    use sea_orm::{DatabaseConnection, EntityTrait};
    use serde_json::json;
    use tower::ServiceExt;
    use api::auth::generate_jwt;
    use crate::helpers::app::make_test_app_with_storage;

    struct TestData {
        admin_user: UserModel,
        regular_user: UserModel,
        module: Module,
    }

    async fn setup_test_data(db: &sea_orm::DatabaseConnection) -> TestData {
        dotenvy::dotenv().expect("Failed to load .env");

        let service = UserService::new(UserRepository::new(db.clone()));
        let admin_user = service.create(CreateUser{ username: "admin".to_string(), email: "admin@test.com".to_string(), password: "password".to_string(), admin: true }).await.expect("Failed to create admin user");
        let regular_user = service.create(CreateUser{ username: "regular".to_string(), email: "regular@test.com".to_string(), password: "password".to_string(), admin: false }).await.expect("Failed to create regular user");
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
    #[serial]
    async fn test_edit_module_success() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

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
    #[serial]
    async fn test_edit_module_forbidden() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

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
    #[serial]
    async fn test_edit_module_invalid_code() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

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
    #[serial]
    async fn test_edit_module_year_in_past() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

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
    #[serial]
    async fn test_edit_module_invalid_credits() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

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
    #[serial]
    async fn test_edit_module_description_too_long() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

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
    #[serial]
    async fn test_edit_module_duplicate_code() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let _other_module = Module::create(
            db::get_connection().await,
            "COS302",
            Utc::now().year(),
            Some("Other module"),
            16,
        )
        .await
        .expect("Failed to create second module");

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
    #[serial]
    async fn test_edit_module_not_found() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

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
    #[serial]
    async fn test_edit_module_multiple_errors() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

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
    #[serial]
    async fn test_edit_module_same_code() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

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

    async fn create_multiple_modules(db: &DatabaseConnection, count: usize) -> Vec<Module> {
        let mut modules = Vec::new();
        for i in 0..count {
            let module = Module::create(
                db,
                &format!("MOD{}", i),
                2025,
                Some("Test module"),
                15,
            )
            .await
            .expect("Failed to create test module");
            modules.push(module);
        }
        modules
    }

    /// Test Case: Admin bulk updates modules successfully
    #[tokio::test]
    #[serial]
    async fn test_bulk_update_modules_success() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let db = app_state.db();
        let data = setup_test_data(db).await;

        let modules = create_multiple_modules(&db, 3).await;
        let module_ids: Vec<i64> = modules.iter().map(|m| m.id).collect();

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let req_body = json!({
            "module_ids": module_ids,
            "year": 2026,
            "description": "Updated description"
        });
        let req = Request::builder()
            .method("PUT")
            .uri("/api/modules/bulk")
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_vec(&req_body).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "Updated 3/3 modules");
        assert_eq!(json["data"]["updated"], 3);
        assert!(json["data"]["failed"].as_array().unwrap().is_empty());

        for id in module_ids {
            let module = ModuleEntity::find_by_id(id)
                .one(db)
                .await
                .unwrap()
                .unwrap();
            assert_eq!(module.year, 2026);
            assert_eq!(module.description, Some("Updated description".into()));
        }
    }

    /// Test Case: Attempt to update module code
    #[tokio::test]
    #[serial]
    async fn test_bulk_update_code_forbidden() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let db = app_state.db();
        let data = setup_test_data(db).await;

        let modules = create_multiple_modules(&db, 2).await;
        let module_ids: Vec<i64> = modules.iter().map(|m| m.id).collect();

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let req_body = json!({
            "module_ids": module_ids,
            "code": "NEWCODE"
        });
        let req = Request::builder()
            .method("PUT")
            .uri("/api/modules/bulk")
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_vec(&req_body).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "Bulk update cannot change module code");
    }

    /// Test Case: Bulk update with no module IDs
    #[tokio::test]
    #[serial]
    async fn test_bulk_update_no_ids() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let db = app_state.db();
        let data = setup_test_data(db).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let req_body = json!({
            "module_ids": [],
            "year": 2026
        });
        let req = Request::builder()
            .method("PUT")
            .uri("/api/modules/bulk")
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_vec(&req_body).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "At least one module ID is required");
    }

    /// Test Case: Partial success with some updates failing
    #[tokio::test]
    #[serial]
    async fn test_bulk_update_partial_success() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let db = app_state.db();
        let data = setup_test_data(db).await;

        let modules = create_multiple_modules(&db, 2).await;
        let mut module_ids: Vec<i64> = modules.iter().map(|m| m.id).collect();
        module_ids.push(99999); // Non-existent ID

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let req_body = json!({
            "module_ids": module_ids,
            "credits": 20
        });
        let req = Request::builder()
            .method("PUT")
            .uri("/api/modules/bulk")
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_vec(&req_body).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        
        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "Updated 2/3 modules");
        assert_eq!(json["data"]["updated"], 2);
        
        let failed = json["data"]["failed"].as_array().unwrap();
        assert_eq!(failed.len(), 1);
        assert_eq!(failed[0]["id"], 99999);
        assert_eq!(failed[0]["error"], "Module not found");

        // Verify successful updates
        for id in modules.iter().map(|m| m.id) {
            let module = ModuleEntity::find_by_id(id)
                .one(db)
                .await
                .unwrap()
                .unwrap();
            assert_eq!(module.credits, 20);
        }
    }

    /// Test Case: Validation errors in bulk update
    #[tokio::test]
    #[serial]
    async fn test_bulk_update_validation_errors() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let db = app_state.db();
        let data = setup_test_data(db).await;

        let modules = create_multiple_modules(&db, 2).await;
        let module_ids: Vec<i64> = modules.iter().map(|m| m.id).collect();

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let req_body = json!({
            "module_ids": module_ids,
            "year": 2000,  // Invalid (past year)
            "credits": 0    // Invalid (must be positive)
        });
        let req = Request::builder()
            .method("PUT")
            .uri("/api/modules/bulk")
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_vec(&req_body).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        let message = json["message"].as_str().unwrap();
        assert!(message.contains("Year must be at least 2024"));
        assert!(message.contains("Credits must be positive"));
    }

    /// Test Case: Non-admin attempts bulk update
    #[tokio::test]
    #[serial]
    async fn test_bulk_update_forbidden() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let db = app_state.db();
        let data = setup_test_data(db).await;

        let modules = create_multiple_modules(&db, 2).await;
        let module_ids: Vec<i64> = modules.iter().map(|m| m.id).collect();

        let (token, _) = generate_jwt(data.regular_user.id, data.regular_user.admin);
        let req_body = json!({
            "module_ids": module_ids,
            "year": 2026
        });
        let req = Request::builder()
            .method("PUT")
            .uri("/api/modules/bulk")
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_vec(&req_body).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);

        // Verify no updates occurred
        for id in module_ids {
            let module = ModuleEntity::find_by_id(id)
                .one(db)
                .await
                .unwrap()
                .unwrap();
            assert_eq!(module.year, 2025); // Original value
        }
    }
}