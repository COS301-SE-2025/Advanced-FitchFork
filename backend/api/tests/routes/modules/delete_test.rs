#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use db::{models::{module::{self, Model as Module}, user::Model as UserModel}};
    use tower::ServiceExt;
    use api::auth::generate_jwt;
    use crate::helpers::app::make_test_app;
    use sea_orm::EntityTrait;

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
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

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
            .one(app_state.db())
            .await
            .unwrap()
            .is_some();
        assert!(!exists, "Module should be deleted from database");
    }

    /// Test Case: Non-admin user attempts to delete module
    #[tokio::test]
    async fn test_delete_module_forbidden() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

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
            .one(app_state.db())
            .await
            .unwrap()
            .is_some();
        assert!(exists, "Module should still exist");
    }

    /// Test Case: Delete non-existent module
    #[tokio::test]
    async fn test_delete_module_not_found() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

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
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;

        let uri = format!("/api/modules/{}", data.module.id);
        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let exists = module::Entity::find_by_id(data.module.id)
            .one(app_state.db())
            .await
            .unwrap()
            .is_some();
        assert!(exists, "Module should still exist");
    }
}