#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use db::{test_utils::setup_test_db, models::{module::{self, Model as Module}, user::Model as UserModel}};
    use tower::ServiceExt;
    use serde_json::json;
    use api::auth::generate_jwt;
    use crate::test_helpers::make_app;
    use sea_orm::{EntityTrait, DatabaseConnection};

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
            2025,
            Some("Module to be deleted"),
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

    /// Test Case: Admin deletes module successfully
    #[tokio::test]
    async fn test_delete_module_success() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}", data.module.id);
        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        
        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "Module deleted successfully");
        assert!(json["data"].is_null());

        let exists = module::Entity::find_by_id(data.module.id)
            .one(&db)
            .await
            .unwrap()
            .is_some();
        assert!(!exists, "Module should be deleted from database");
    }

    /// Test Case: Non-admin user attempts to delete module
    #[tokio::test]
    async fn test_delete_module_forbidden() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;
    
        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.regular_user.id, data.regular_user.admin);
        let uri = format!("/api/modules/{}", data.module.id);
        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "Admin access required");

        let exists = module::Entity::find_by_id(data.module.id)
            .one(&db)
            .await
            .unwrap()
            .is_some();
        assert!(exists, "Module should still exist");
    }

    /// Test Case: Delete non-existent module
    #[tokio::test]
    async fn test_delete_module_not_found() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/modules/{}", 99999);
        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "Module 99999 not found.");
    }

    /// Test Case: Missing authorization header
    #[tokio::test]
    async fn test_delete_module_unauthorized() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let uri = format!("/api/modules/{}", data.module.id);
        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let exists = module::Entity::find_by_id(data.module.id)
            .one(&db)
            .await
            .unwrap()
            .is_some();
        assert!(exists, "Module should still exist");
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

    /// Test Case: Admin bulk deletes modules successfully
    #[tokio::test]
    async fn test_bulk_delete_modules_success() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;
        let modules = create_multiple_modules(&db, 3).await;
        let module_ids: Vec<i64> = modules.iter().map(|m| m.id).collect();

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let req_body = json!({ "module_ids": module_ids });
        let req = Request::builder()
            .method("DELETE")
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
        assert_eq!(json["message"], "Deleted 3/3 modules");
        assert_eq!(json["data"]["deleted"], 3);
        assert!(json["data"]["failed"].as_array().unwrap().is_empty());

        for id in module_ids {
            let exists = module::Entity::find_by_id(id)
                .one(&db)
                .await
                .unwrap()
                .is_some();
            assert!(!exists, "Module {} should be deleted", id);
        }
    }

    /// Test Case: Bulk delete with no module IDs
    #[tokio::test]
    async fn test_bulk_delete_no_ids() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let req_body = json!({ "module_ids": [] });
        let req = Request::builder()
            .method("DELETE")
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

    /// Test Case: Non-admin attempts bulk delete
    #[tokio::test]
    async fn test_bulk_delete_forbidden() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;
        let modules = create_multiple_modules(&db, 2).await;
        let module_ids: Vec<i64> = modules.iter().map(|m| m.id).collect();

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.regular_user.id, data.regular_user.admin);
        let req_body = json!({ "module_ids": module_ids });
        let req = Request::builder()
            .method("DELETE")
            .uri("/api/modules/bulk")
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_vec(&req_body).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);

        for id in module_ids {
            let exists = module::Entity::find_by_id(id)
                .one(&db)
                .await
                .unwrap()
                .is_some();
            assert!(exists, "Module {} should still exist", id);
        }
    }

    /// Test Case: Partial success with some modules not found
    #[tokio::test]
    async fn test_bulk_delete_partial_success() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;
        let modules = create_multiple_modules(&db, 2).await;
        let mut module_ids: Vec<i64> = modules.iter().map(|m| m.id).collect();
        module_ids.push(99999); // Non-existent ID

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let req_body = json!({ "module_ids": module_ids });
        let req = Request::builder()
            .method("DELETE")
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
        assert_eq!(json["message"], "Deleted 2/3 modules");
        assert_eq!(json["data"]["deleted"], 2);
        
        let failed = json["data"]["failed"].as_array().unwrap();
        assert_eq!(failed.len(), 1);
        assert_eq!(failed[0]["id"], 99999);
        assert_eq!(failed[0]["error"], "Module not found");
    }

    /// Test Case: Database error during bulk delete
    #[tokio::test]
    async fn test_bulk_delete_database_error() {
        let db = setup_test_db().await;
        let data = setup_test_data(&db).await;
        
        // Create invalid ID (negative)
        let module_ids = vec![-1];

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let req_body = json!({ "module_ids": module_ids });
        let req = Request::builder()
            .method("DELETE")
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
        assert_eq!(json["message"], "Deleted 0/1 modules");
        
        let failed = json["data"]["failed"].as_array().unwrap();
        assert_eq!(failed.len(), 1);
        assert!(failed[0]["error"].as_str().unwrap().contains("Module not found"));
    }
}