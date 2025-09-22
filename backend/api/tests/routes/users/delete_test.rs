#[cfg(test)]
mod tests {
    use crate::helpers::app::make_test_app_with_storage;
    use api::auth::generate_jwt;
    use axum::{
        body::Body as AxumBody,
        http::{Request, StatusCode},
    };
    use db::{models::user::Model as UserModel, repositories::user_repository::UserRepository};
    use serde_json::Value;
    use services::{
        service::Service,
        user::{CreateUser, UserService},
    };
    use tower::ServiceExt;

    struct TestData {
        admin_user: UserModel,
        non_admin_user: UserModel,
        user_to_delete: UserModel,
    }

    async fn setup_test_data(db: &sea_orm::DatabaseConnection) -> TestData {
        dotenvy::dotenv().expect("Failed to load .env");

        let service = UserService::new(UserRepository::new(db.clone()));
        let admin_user = service
            .create(CreateUser {
                username: "delete_admin".to_string(),
                email: "delete_admin@test.com".to_string(),
                password: "adminpass".to_string(),
                admin: true,
            })
            .await
            .expect("Failed to create admin user for DELETE tests");
        let non_admin_user = service
            .create(CreateUser {
                username: "delete_regular".to_string(),
                email: "delete_regular@test.com".to_string(),
                password: "userpass".to_string(),
                admin: false,
            })
            .await
            .expect("Failed to create regular user for DELETE tests");
        let user_to_delete = service
            .create(CreateUser {
                username: "target_for_deletion".to_string(),
                email: "target_delete@test.com".to_string(),
                password: "deletepass".to_string(),
                admin: false,
            })
            .await
            .expect("Failed to create target user for deletion");

        TestData {
            admin_user,
            non_admin_user,
            user_to_delete,
        }
    }

    async fn get_json_body(response: axum::response::Response) -> Value {
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        serde_json::from_slice(&body).unwrap()
    }

    /// Test Case: Successful Deletion of User as Admin
    #[tokio::test]
    #[serial]
    async fn test_delete_user_success_as_admin() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/users/{}", data.user_to_delete.id);
        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let response = app.clone().oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let json = get_json_body(response).await;
        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "User deleted successfully");
        assert_eq!(json["data"], serde_json::Value::Null);

        let get_uri = format!("/api/users/{}", data.user_to_delete.id);
        let get_req = Request::builder()
            .method("GET")
            .uri(&get_uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let get_response = app.oneshot(get_req).await.unwrap();
        assert_eq!(get_response.status(), StatusCode::NOT_FOUND);

        let get_json = get_json_body(get_response).await;
        assert_eq!(get_json["success"], false);
        assert_eq!(get_json["message"], "User 3 not found.");
    }

    /// Test Case: Attempt to Delete Own Account as Admin
    #[tokio::test]
    #[serial]
    async fn test_delete_user_forbidden_delete_self_as_admin() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/users/{}", data.admin_user.id);
        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);

        let json = get_json_body(response).await;
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "You cannot delete your own account");
    }

    /// Test Case: Attempt to Delete Own Account as Non-Admin
    #[tokio::test]
    #[serial]
    async fn test_delete_user_forbidden_delete_self_as_non_admin() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.non_admin_user.id, data.non_admin_user.admin);
        let uri = format!("/api/users/{}", data.non_admin_user.id);
        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    /// Test Case: Delete User without Admin Role
    #[tokio::test]
    #[serial]
    async fn test_delete_user_forbidden_non_admin() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.non_admin_user.id, data.non_admin_user.admin);
        let uri = format!("/api/users/{}", data.user_to_delete.id);
        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    /// Test Case: Delete Non-Existent User
    #[tokio::test]
    #[serial]
    async fn test_delete_user_not_found() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/users/{}", 999999);
        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let json = get_json_body(response).await;
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "User 999999 not found.");
    }

    /// Test Case: Delete User without Authentication Header
    #[tokio::test]
    #[serial]
    async fn test_delete_user_missing_auth_header() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let uri = format!("/api/users/{}", data.user_to_delete.id);
        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    /// Test Case: Delete User with Invalid JWT Token
    #[tokio::test]
    #[serial]
    async fn test_delete_user_invalid_token() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let uri = format!("/api/users/{}", data.user_to_delete.id);
        let req = Request::builder()
            .method("DELETE")
            .uri(&uri)
            .header("Authorization", "Bearer invalid.token.here")
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
