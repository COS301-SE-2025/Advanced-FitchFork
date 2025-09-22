#[cfg(test)]
mod tests {
    use crate::helpers::app::make_test_app_with_storage;
    use api::auth::generate_jwt;
    use axum::{
        body::Body as AxumBody,
        http::{Request, StatusCode},
    };
    use db::models::{
        module::Model as ModuleModel,
        user::Model as UserModel,
        user_module_role::{Model as UserModuleRoleModel, Role},
    };
    use serde_json::Value;
    use tower::ServiceExt;

    struct TestData {
        admin_user: UserModel,
        non_admin_user: UserModel,
        user_to_operate_on: UserModel,
        module: ModuleModel,
    }

    async fn setup_test_data(db: &sea_orm::DatabaseConnection) -> TestData {
        dotenvy::dotenv().expect("Failed to load .env");

        let admin_user =
            UserModel::create(db, "user_admin", "user_admin@test.com", "adminpass", true)
                .await
                .expect("Failed to create admin user");
        let non_admin_user = UserModel::create(
            db,
            "user_regular",
            "user_regular@test.com",
            "userpass",
            false,
        )
        .await
        .expect("Failed to create regular user");
        let user_to_operate_on =
            UserModel::create(db, "target_user", "target@test.com", "targetpass", false)
                .await
                .expect("Failed to create target user");
        let module = ModuleModel::create(db, "USER101", 2024, Some("User Test Module"), 16)
            .await
            .expect("Failed to create test module");
        UserModuleRoleModel::assign_user_to_module(
            db,
            user_to_operate_on.id,
            module.id,
            Role::Student,
        )
        .await
        .expect("Failed to assign role to target user");

        TestData {
            admin_user,
            non_admin_user,
            user_to_operate_on,
            module,
        }
    }

    async fn get_json_body(response: axum::response::Response) -> Value {
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        serde_json::from_slice(&body).unwrap()
    }

    // --- GET /api/users (list_users) ---

    #[tokio::test]
    #[serial]
    async fn test_list_users_success_as_admin() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = "/api/users";
        let req = Request::builder()
            .method("GET")
            .uri(uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let json = get_json_body(response).await;
        assert_eq!(json["success"], true);

        assert!(json["data"]["users"].as_array().is_some());
        assert!(json["data"]["total"].as_u64().is_some());
        assert!(json["data"]["page"].as_u64().is_some());
        assert!(json["data"]["per_page"].as_u64().is_some());
    }

    /// Test Case: Listing Users without Admin Role
    #[tokio::test]
    #[serial]
    async fn test_list_users_forbidden_non_admin() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.non_admin_user.id, data.non_admin_user.admin);
        let uri = "/api/users";
        let req = Request::builder()
            .method("GET")
            .uri(uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    /// Test Case: Listing Users without Authentication
    #[tokio::test]
    #[serial]
    async fn test_list_users_missing_auth() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let _ = setup_test_data(app_state.db()).await;

        let uri = "/api/users";
        let req = Request::builder()
            .method("GET")
            .uri(uri)
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    /// Test Case: Listing Users with Pagination
    #[tokio::test]
    #[serial]
    async fn test_list_users_with_pagination() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = "/api/users?page=1&per_page=2";
        let req = Request::builder()
            .method("GET")
            .uri(uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let json = get_json_body(response).await;
        assert_eq!(json["success"], true);
        assert_eq!(json["data"]["page"], 1);
        assert_eq!(json["data"]["per_page"], 2);
        assert!(json["data"]["users"].as_array().unwrap().len() <= 2);
    }

    /// Test Case: Listing Users with Filtering/Sorting
    #[tokio::test]
    #[serial]
    async fn test_list_users_with_query_params() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = "/api/users?query=target&sort=-username";
        let req = Request::builder()
            .method("GET")
            .uri(uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let json = get_json_body(response).await;
        assert_eq!(json["success"], true);
    }

    // --- GET /api/users/{user_id}/modules (get_user_modules) ---

    /// Test Case: Successful Retrieval of User Modules as Admin
    #[tokio::test]
    #[serial]
    async fn test_get_user_modules_success_as_admin() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/users/{}/modules", data.user_to_operate_on.id);
        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let json = get_json_body(response).await;
        assert_eq!(json["success"], true);

        let modules_array = json["data"]
            .as_array()
            .expect("Data should be an array of modules");
        assert_eq!(modules_array.len(), 1);
        assert_eq!(modules_array[0]["id"], data.module.id);
        assert_eq!(modules_array[0]["code"], data.module.code);
        assert_eq!(modules_array[0]["role"], "student");
    }

    /// Test Case: Retrieving User Modules without Admin Role
    #[tokio::test]
    #[serial]
    async fn test_get_user_modules_forbidden_non_admin() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.non_admin_user.id, data.non_admin_user.admin);
        let uri = format!("/api/users/{}/modules", data.user_to_operate_on.id);
        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    /// Test Case: Retrieving Modules for Non-Existent User
    #[tokio::test]
    #[serial]
    async fn test_get_user_modules_user_not_found() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/users/{}/modules", 999999);
        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let json = get_json_body(response).await;
        assert_eq!(json["success"], false);
    }

    /// Test Case: Retrieving User Modules without Authentication
    #[tokio::test]
    #[serial]
    async fn test_get_user_modules_missing_auth() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let uri = format!("/api/users/{}/modules", data.user_to_operate_on.id);
        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    // --- /api/users/{user_id} (get_user) ---

    /// Test Case: Successful Retrieval of Specific User as Admin
    #[tokio::test]
    #[serial]
    async fn test_get_user_success_as_admin() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/users/{}", data.user_to_operate_on.id);
        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let json = get_json_body(response).await;
        assert_eq!(json["success"], true);
        let user_data = &json["data"];
        assert_eq!(user_data["id"], data.user_to_operate_on.id.to_string());
        assert_eq!(user_data["username"], data.user_to_operate_on.username);
        assert_eq!(user_data["email"], data.user_to_operate_on.email);
        assert_eq!(user_data["admin"], data.user_to_operate_on.admin);
    }

    /// Test Case: Retrieving Specific User without Admin Role
    #[tokio::test]
    #[serial]
    async fn test_get_user_forbidden_non_admin() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.non_admin_user.id, data.non_admin_user.admin);
        let uri = format!("/api/users/{}", data.user_to_operate_on.id);
        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    /// Test Case: Retrieving Non-Existent User
    #[tokio::test]
    #[serial]
    async fn test_get_user_not_found() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = format!("/api/users/{}", 999999);
        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let json = get_json_body(response).await;
        assert_eq!(json["success"], false);
    }

    /// Test Case: Retrieving Specific User without Authentication
    #[tokio::test]
    #[serial]
    async fn test_get_user_missing_auth() {
        let (app, app_state, _tmp) = make_test_app_with_storage().await;
        let data = setup_test_data(app_state.db()).await;

        let uri = format!("/api/users/{}", data.user_to_operate_on.id);
        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
