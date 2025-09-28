#[cfg(test)]
mod tests {
    use crate::helpers::app::make_test_app_with_storage;
    use api::auth::generate_jwt;
    use axum::{
        body::Body as AxumBody,
        http::{Request, StatusCode},
    };
    use db::models::user::Model as UserModel;
    use serde_json::{Value, json};
    use tower::ServiceExt;

    struct TestData {
        admin_user: UserModel,
        non_admin_user: UserModel,
    }

    async fn setup_test_data(db: &sea_orm::DatabaseConnection) -> TestData {
        dotenvy::dotenv().expect("Failed to load .env");

        let admin_user = UserModel::create(db, "admin_user", "admin@test.com", "adminpass", true)
            .await
            .unwrap();
        let non_admin_user =
            UserModel::create(db, "normal_user", "user@test.com", "userpass", false)
                .await
                .unwrap();

        TestData {
            admin_user,
            non_admin_user,
        }
    }

    async fn get_json_body(response: axum::response::Response) -> Value {
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        serde_json::from_slice(&body).unwrap()
    }

    /// Test Case: Successful Creation of Single User as Admin
    #[tokio::test]
    async fn test_create_user_success_as_admin() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let req_body = json!({
            "username": "newuser",
            "email": "newuser@example.com",
            "password": "securepass"
        });

        let req = Request::builder()
            .method("POST")
            .uri("/api/users")
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(AxumBody::from(req_body.to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);

        let json = get_json_body(response).await;
        assert_eq!(json["success"], true);
        assert_eq!(json["data"]["username"], "newuser");
    }

    /// Test Case: Creating Single User is Forbidden for Non-Admin
    #[tokio::test]
    async fn test_create_user_forbidden_as_non_admin() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.non_admin_user.id, data.non_admin_user.admin);
        let req_body = json!({
            "username": "blockeduser",
            "email": "blocked@example.com",
            "password": "password"
        });

        let req = Request::builder()
            .method("POST")
            .uri("/api/users")
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(AxumBody::from(req_body.to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    /// Test Case: Single User Creation Validation Failure
    #[tokio::test]
    async fn test_create_user_validation_failure() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let req_body = json!({
            "username": "",
            "email": "invalid",
            "password": "123"
        });

        let req = Request::builder()
            .method("POST")
            .uri("/api/users")
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(AxumBody::from(req_body.to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    /// Test Case: Successful Bulk User Creation
    #[tokio::test]
    async fn test_bulk_create_users_success_as_admin() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let req_body = json!({
            "users": [
                { "username": "bulkuser1", "email": "b1@test.com", "password": "bulkpass1" },
                { "username": "bulkuser2", "email": "b2@test.com", "password": "bulkpass2" }
            ]
        });

        let req = Request::builder()
            .method("POST")
            .uri("/api/users/bulk")
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(AxumBody::from(req_body.to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);

        let json = get_json_body(response).await;
        assert_eq!(json["success"], true);
        assert_eq!(json["data"].as_array().unwrap().len(), 2);
    }

    /// Test Case: Bulk User Creation Validation Failure
    #[tokio::test]
    async fn test_bulk_create_users_validation_failure() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let req_body = json!({
            "users": [
                { "username": "", "email": "bad", "password": "123" }
            ]
        });

        let req = Request::builder()
            .method("POST")
            .uri("/api/users/bulk")
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(AxumBody::from(req_body.to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    /// Test Case: Bulk User Creation Conflict on Duplicate
    #[tokio::test]
    async fn test_bulk_create_users_conflict() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        UserModel::create(app_state.db(), "dupe", "dupe@test.com", "pass1234", false)
            .await
            .unwrap();
        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);

        let req_body = json!({
            "users": [
                { "username": "dupe", "email": "dupe@test.com", "password": "pass1234" },
                { "username": "another", "email": "another@test.com", "password": "pass1234" }
            ]
        });

        let req = Request::builder()
            .method("POST")
            .uri("/api/users/bulk")
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(AxumBody::from(req_body.to_string()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::CONFLICT);

        let json = get_json_body(response).await;
        assert_eq!(json["success"], false);
        assert!(json["message"].as_str().unwrap().contains("dupe"));
    }
}
