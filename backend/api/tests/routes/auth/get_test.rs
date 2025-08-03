#[cfg(test)]
mod tests {
    use db::{
        models::{
            user::Model as UserModel,
            module::Model as ModuleModel,
            user_module_role::{Model as UserModuleRoleModel, Role},
        },
    };
    use axum::{
        body::Body as AxumBody,
        http::{Request, StatusCode, header},
    };
    use tower::ServiceExt;
    use serde_json::Value;
    use api::auth::generate_jwt;
    use crate::helpers::app::make_test_app;
    use tempfile::{tempdir, TempDir};
    use std::io::Write;
    use serial_test::serial;
    use sea_orm::{Set, ActiveModelTrait};

    struct TestData {
        regular_user: UserModel,
        admin_user: UserModel,
        module: ModuleModel,
    }

    async fn setup_test_data(db: &sea_orm::DatabaseConnection) -> (TestData, TempDir) {
        dotenvy::dotenv().expect("Failed to load .env");
        let temp_dir = tempdir().expect("Failed to create temporary directory for avatars");
        unsafe { std::env::set_var("USER_PROFILE_STORAGE_ROOT", temp_dir.path().to_str().unwrap()); }

        let regular_user = UserModel::create(db, "regular_user", "regular@test.com", "password123", false).await.expect("Failed to create regular user");
        let admin_user = UserModel::create(db, "admin_user", "admin@test.com", "password456", true).await.expect("Failed to create admin user");
        let module = ModuleModel::create(db, "AUTH101", 2024, Some("Auth Test Module"), 16).await.expect("Failed to create test module");
        UserModuleRoleModel::assign_user_to_module(db, regular_user.id, module.id, Role::Student).await.expect("Failed to assign role");

        (
            TestData {
                regular_user,
                admin_user,
                module,
            },
            temp_dir,
        )
    }

    /// Test Case: Successful Retrieval of Current User Info (/api/auth/me) as Regular User
    #[tokio::test]
    #[serial]
    async fn test_get_me_success_regular_user() {
        let (app, app_state) = make_test_app().await;
        let (data, _temp_dir)  = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.regular_user.id, data.regular_user.admin);
        let uri = "/api/auth/me";
        let req = Request::builder()
            .method("GET")
            .uri(uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "User data retrieved successfully");
        let user_data = &json["data"];

        assert_eq!(user_data["id"], data.regular_user.id);
        assert_eq!(user_data["username"], data.regular_user.username);
        assert_eq!(user_data["email"], data.regular_user.email);
        assert_eq!(user_data["admin"], data.regular_user.admin);
        assert!(user_data["created_at"].as_str().is_some());
        assert!(user_data["updated_at"].as_str().is_some());

        let modules_array = user_data["modules"].as_array().expect("Modules should be an array");
        assert_eq!(modules_array.len(), 1);
        let module_data = &modules_array[0];
        assert_eq!(module_data["id"], data.module.id);
        assert_eq!(module_data["code"], data.module.code);
        assert_eq!(module_data["role"], "student");
    }

    /// Test Case: Successful Retrieval of Current User Info (/api/auth/me) as Admin User
    #[tokio::test]
    #[serial]
    async fn test_get_me_success_admin_user() {
        let (app, app_state) = make_test_app().await;
        let (data, _temp_dir)  = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.admin_user.id, data.admin_user.admin);
        let uri = "/api/auth/me";
        let req = Request::builder()
            .method("GET")
            .uri(uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "User data retrieved successfully");
        let user_data = &json["data"];

        assert_eq!(user_data["id"], data.admin_user.id);
        assert_eq!(user_data["username"], data.admin_user.username);
        assert_eq!(user_data["email"], data.admin_user.email);
        assert_eq!(user_data["admin"], data.admin_user.admin);

        let modules_array = user_data["modules"].as_array().expect("Modules should be an array");
        assert_eq!(modules_array.len(), 0);
    }

    /// Test Case: Retrieving User Info for Non-Existent User ID in Token Claims
    #[tokio::test]
    #[serial]
    async fn test_get_me_user_not_found() {
        let (app, app_state) = make_test_app().await;
        let _  = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(999999, false);
        let uri = "/api/auth/me";
        let req = Request::builder()
            .method("GET")
            .uri(uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "User not found");
    }

    /// Test Case: Accessing /api/auth/me without Authorization Header
    #[tokio::test]
    #[serial]
    async fn test_get_me_missing_auth_header() {
        let (app, app_state) = make_test_app().await;
        let _  = setup_test_data(app_state.db()).await;

        let uri = "/api/auth/me";
        let req = Request::builder()
            .method("GET")
            .uri(uri)
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    /// Test Case: Accessing /api/auth/me with Invalid JWT Token (Forbidden)
    #[tokio::test]
    #[serial]
    async fn test_get_me_invalid_token() {
        let (app, app_state) = make_test_app().await;
        let _  = setup_test_data(app_state.db()).await;

        let uri = "/api/auth/me";
        let req = Request::builder()
            .method("GET")
            .uri(uri)
            .header("Authorization", "Bearer invalid.token.here")
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    /// Test Case: Successful Retrieval of User Avatar (/api/auth/avatar/{user_id})
    #[tokio::test]
    #[serial]
    async fn test_get_avatar_success() {
        let (app, app_state) = make_test_app().await;
        let (data, temp_dir)  = setup_test_data(app_state.db()).await;

        let avatar_filename = format!("avatar_{}.png", data.regular_user.id);
        let avatar_path = temp_dir.path().join(&avatar_filename);

        let mut file = std::fs::File::create(&avatar_path).expect("Failed to create avatar file");
        file.write_all(b"fake_png_data_content").expect("Failed to write to avatar file");

        let mut user_active_model: db::models::user::ActiveModel = data.regular_user.clone().into();
        user_active_model.profile_picture_path = Set(Some(avatar_filename.clone()));
        let _ = user_active_model.update(app_state.db()).await.expect("Failed to update user profile picture path");

        let uri = format!("/api/auth/avatar/{}", data.regular_user.id);
        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let headers = response.headers();
        let content_type = headers.get(header::CONTENT_TYPE).expect("Content-Type header should be present");
        assert_eq!(content_type, "image/png");
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(&body_bytes[..], b"fake_png_data_content");
    }

    /// Test Case: Retrieving Avatar for Non-Existent User (/api/auth/avatar/{user_id})
    #[tokio::test]
    #[serial]
    async fn test_get_avatar_user_not_found() {
        let (app, app_state) = make_test_app().await;
        let _  = setup_test_data(app_state.db()).await;

        let uri = format!("/api/auth/avatar/{}", 999999);
        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "User 999999 not found.");
    }

    /// Test Case: Retrieving Avatar when No Avatar is Set for User (/api/auth/avatar/{user_id})
    #[tokio::test]
    #[serial]
    async fn test_get_avatar_no_avatar_set() {
        let (app, app_state) = make_test_app().await;
        let (data, _temp_dir)  = setup_test_data(app_state.db()).await;

        let uri = format!("/api/auth/avatar/{}", data.regular_user.id);
        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "No avatar set");
    }

    /// Test Case: Retrieving Avatar when Avatar File is Missing from Disk (/api/auth/avatar/{user_id})
    #[tokio::test]
    #[serial]
    async fn test_get_avatar_file_missing() {
        let (app, app_state) = make_test_app().await;
        let (data, _temp_dir)  = setup_test_data(app_state.db()).await;

        let uri = format!("/api/auth/avatar/{}", data.regular_user.id);
        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "No avatar set");
    }

    /// Test Case: Successful Role Check (/api/auth/has-role) - User Has Role
    #[tokio::test]
    #[serial]
    async fn test_has_role_in_module_success_has_role() {
        let (app, app_state) = make_test_app().await;
        let (data, _temp_dir)  = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.regular_user.id, data.regular_user.admin);
        let uri = format!("/api/auth/has-role?module_id={}&role=student", data.module.id);
        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "Role check completed");
        assert_eq!(json["data"]["has_role"], true);
    }

    /// Test Case: Successful Role Check (/api/auth/has-role) - User Does Not Have Role
    #[tokio::test]
    #[serial]
    async fn test_has_role_in_module_success_does_not_have_role() {
        let (app, app_state) = make_test_app().await;
        let (data, _temp_dir)  = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.regular_user.id, data.regular_user.admin);
        let uri = format!("/api/auth/has-role?module_id={}&role=lecturer", data.module.id);
        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "Role check completed");
        assert_eq!(json["data"]["has_role"], false);
    }

     /// Test Case: Role Check with Invalid Role Parameter (/api/auth/has-role)
    #[tokio::test]
    #[serial]
    async fn test_has_role_in_module_invalid_role() {
        let (app, app_state) = make_test_app().await;
        let (data, _temp_dir)  = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.regular_user.id, data.regular_user.admin);
        let uri = format!("/api/auth/has-role?module_id={}&role=invalidrole", data.module.id);
        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "Invalid role specified");
    }

    /// Test Case: Role Check without Required Query Parameters (/api/auth/has-role)
    #[tokio::test]
    #[serial]
    async fn test_has_role_in_module_missing_parameters() {
        let (app, app_state) = make_test_app().await;
        let (data, _temp_dir)  = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.regular_user.id, data.regular_user.admin);
        let uri = "/api/auth/has-role";
        let req = Request::builder()
            .method("GET")
            .uri(uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

     /// Test Case: Role Check without Authorization Header
    #[tokio::test]
    #[serial]
    async fn test_has_role_in_module_missing_auth() {
        let (app, app_state) = make_test_app().await;
        let (data, _temp_dir)  = setup_test_data(app_state.db()).await;

        let uri = format!("/api/auth/has-role?module_id={}&role=student", data.module.id);
        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    /// Test Case: Successful Role Retrieval - User Has Role in Module
    #[tokio::test]
    #[serial]
    async fn test_get_module_role_success_with_role() {
        let (app, app_state) = make_test_app().await;
        let (data, _temp_dir)  = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.regular_user.id, data.regular_user.admin);
        let uri = format!("/api/auth/module-role?module_id={}", data.module.id);
        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        
        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "Role fetched successfully");
        assert_eq!(json["data"]["role"], "student");
    }

    /// Test Case: Successful Role Retrieval - User Has No Role in Module
    #[tokio::test]
    #[serial]
    async fn test_get_module_role_success_no_role() {
        let (app, app_state) = make_test_app().await;
        let (data, _temp_dir)  = setup_test_data(app_state.db()).await;

        // Create a new module that user is not assigned to
        let new_module = ModuleModel::create(app_state.db(), "MATH101", 2024, Some("Math Module"), 16)
            .await
            .expect("Failed to create test module");

        let (token, _) = generate_jwt(data.regular_user.id, data.regular_user.admin);
        let uri = format!("/api/auth/module-role?module_id={}", new_module.id);
        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        
        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "Role fetched successfully");
        assert!(json["data"]["role"].is_null());
    }

    /// Test Case: Missing Module ID Parameter
    #[tokio::test]
    #[serial]
    async fn test_get_module_role_missing_module_id() {
        let (app, app_state) = make_test_app().await;
        let (data, _temp_dir)  = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.regular_user.id, data.regular_user.admin);
        let uri = "/api/auth/module-role";
        let req = Request::builder()
            .method("GET")
            .uri(uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    /// Test Case: Unauthenticated Access
    #[tokio::test]
    #[serial]
    async fn test_get_module_role_unauthenticated() {
        let (app, app_state) = make_test_app().await;
        let (data, _temp_dir)  = setup_test_data(app_state.db()).await;

        let uri = format!("/api/auth/module-role?module_id={}", data.module.id);
        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    /// Test Case: Invalid Module ID Format
    #[tokio::test]
    #[serial]
    async fn test_get_module_role_invalid_module_id() {
        let (app, app_state) = make_test_app().await;
        let (data, _temp_dir)  = setup_test_data(app_state.db()).await;

        let (token, _) = generate_jwt(data.regular_user.id, data.regular_user.admin);
        let uri = "/api/auth/module-role?module_id=invalid";
        let req = Request::builder()
            .method("GET")
            .uri(uri)
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}