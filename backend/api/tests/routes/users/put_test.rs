#[cfg(test)]
mod tests {
    use db::{
        models::user::Model as UserModel,
    };
    use axum::{
        body::Body as AxumBody,
        http::{Request, StatusCode, header::CONTENT_TYPE},
    };
    use tower::ServiceExt;
    use serde_json::{Value, json};
    use api::auth::generate_jwt;
    use crate::helpers::app::make_test_app_with_storage;

    struct TestData {
        admin_user: UserModel,
        non_admin_user: UserModel,
        user_to_update: UserModel,
    }

    async fn setup_test_data(db: &sea_orm::DatabaseConnection) -> TestData {
        dotenvy::dotenv().expect("Failed to load .env");

        let admin_user = UserModel::create(db, "put_admin", "put_admin@test.com", "adminpass", true).await.expect("Failed to create admin user for PUT tests");
        let non_admin_user = UserModel::create(db, "put_regular", "put_regular@test.com", "userpass", false).await.expect("Failed to create regular user for PUT tests");
        let user_to_update = UserModel::create(db, "target_for_update", "target_original@test.com", "originalpass", false).await.expect("Failed to create target user for update");

        TestData {
            admin_user,
            non_admin_user,
            user_to_update,
        }
    }

    async fn get_json_body(response: axum::response::Response) -> Value {
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        serde_json::from_slice(&body).unwrap()
    }

    // --- PUT /api/users/{user_id} (update_user) ---

    /// Test Case: Successful Update of User (Username and Email) as Admin
    #[tokio::test]
    async fn test_update_user_success_as_admin() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let payload = json!({ "username": "updated_username_put", "email": "updated_put@test.com" });
        let uri = format!("/api/users/{}", data.user_to_update.id);
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let json = get_json_body(response).await;
        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "User updated successfully");
        let updated_user_data = &json["data"];

        assert_eq!(updated_user_data["id"], data.user_to_update.id);
        assert_eq!(updated_user_data["username"], "updated_username_put");
        assert_eq!(updated_user_data["email"], "updated_put@test.com");
        assert_eq!(updated_user_data["admin"], data.user_to_update.admin);
        assert!(updated_user_data["created_at"].as_str().is_some());
        assert!(updated_user_data["updated_at"].as_str().is_some());
        assert!(updated_user_data["updated_at"].as_str().unwrap() >= updated_user_data["created_at"].as_str().unwrap());
    }

    /// Test Case: Successful Update of User Admin Status as Admin
    #[tokio::test]
    async fn test_update_user_admin_status_as_admin() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let payload = json!({ "admin": true });
        let uri = format!("/api/users/{}", data.user_to_update.id);
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);

        let json = get_json_body(response).await;
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "Changing admin status is not allowed");

    }

    /// Test Case: Update User without Admin Role
    #[tokio::test]
    async fn test_update_user_forbidden_non_admin() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.non_admin_user.id, data.non_admin_user.admin);
        let payload = json!({ "username": "hacker_attempt_put" });
        let uri = format!("/api/users/{}", data.user_to_update.id);
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

     /// Test Case: Update Own User Info as Non-Admin
    #[tokio::test]
    async fn test_update_user_forbidden_update_self_as_non_admin() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.non_admin_user.id, data.non_admin_user.admin);
        let payload = json!({ "username": "allowed_to_change_myself" });
        let uri = format!("/api/users/{}", data.non_admin_user.id);
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    /// Test Case: Update Non-Existent User
    #[tokio::test]
    async fn test_update_user_not_found() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let payload = json!({ "username": "doesnt_matter_for_not_found" });
        let uri = format!("/api/users/{}", 999999);
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let json = get_json_body(response).await;
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "User 999999 not found.");
    }

    /// Test Case: Update User with Invalid Data (e.g., Duplicate Email)
    #[tokio::test]
    async fn test_update_user_invalid_data_duplicate_email() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let payload = json!({ "email": data.admin_user.email });
        let uri = format!("/api/users/{}", data.user_to_update.id);
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::CONFLICT);

        let json = get_json_body(response).await;
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "A user with this email already exists");
    }

    /// Test Case: Update User with Invalid Data (e.g., Duplicate Username)
    #[tokio::test]
    async fn test_update_user_invalid_data_duplicate_username() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let payload = json!({ "username": data.admin_user.username });
        let uri = format!("/api/users/{}", data.user_to_update.id);
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::CONFLICT);

        let json = get_json_body(response).await;
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "A user with this student number already exists");
    }

    /// Test Case: Update User without Authentication Header
    #[tokio::test]
    async fn test_update_user_missing_auth_header() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let payload = json!({ "username": "no_auth_update_attempt" });
        let uri = format!("/api/users/{}", data.user_to_update.id);
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    /// Test Case: Update User with Invalid JWT Token
    #[tokio::test]
    async fn test_update_user_invalid_token() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let payload = json!({ "username": "invalid_token_update" });
        let uri = format!("/api/users/{}", data.user_to_update.id);
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Authorization", "Bearer invalid.token.here")
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}