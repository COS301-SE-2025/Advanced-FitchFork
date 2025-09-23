#[cfg(test)]
mod tests {
    use crate::helpers::app::{make_test_app, make_test_app_with_storage};
    use api::auth::generate_jwt;
    use axum::{
        body::Body as AxumBody,
        http::{Request, StatusCode, header::CONTENT_TYPE},
        response::Response,
    };
    use db::models::{
        password_reset_token::Model as PasswordResetTokenModel, user::Model as UserModel,
    };
    use serde_json::{Value, json};
    use serial_test::serial;
    use tempfile::tempdir;
    use tower::ServiceExt;

    async fn get_json_body(response: Response) -> Value {
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        serde_json::from_slice(&body).unwrap()
    }

    /// Test Case: Successful User Registration
    #[tokio::test]
    async fn test_register_success() {
        let (app, _) = make_test_app().await;

        let payload = json!({"username": "testuser123", "email": "testuser123@example.com", "password": "securepassword123"});
        let uri = "/api/auth/register";
        let req = Request::builder()
            .method("POST")
            .uri(uri)
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);

        let json = get_json_body(response).await;
        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "User registered successfully");
        let user_data = &json["data"];

        assert_eq!(user_data["username"], "testuser123");
        assert_eq!(user_data["email"], "testuser123@example.com");
        assert_eq!(user_data["admin"], false);

        assert!(user_data["token"].as_str().is_some());
        assert!(user_data["expires_at"].as_str().is_some());
        assert!(user_data["id"].as_i64().is_some());
    }

    /// Test Case: User Registration with Invalid Email
    #[tokio::test]
    async fn test_register_invalid_email() {
        let (app, _) = make_test_app().await;

        let payload = json!({"username": "testuser456", "email": "not-an-email", "password": "securepassword456"});
        let uri = "/api/auth/register";
        let req = Request::builder()
            .method("POST")
            .uri(uri)
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let json = get_json_body(response).await;
        assert_eq!(json["success"], false);
        assert!(json["message"].as_str().unwrap().contains("email"));
    }

    /// Test Case: User Registration with Short Password
    #[tokio::test]
    async fn test_register_short_password() {
        let (app, _) = make_test_app().await;

        let payload = json!({"username": "testuser789", "email": "testuser789@example.com", "password": "short"});
        let uri = "/api/auth/register";
        let req = Request::builder()
            .method("POST")
            .uri(uri)
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let json = get_json_body(response).await;
        assert_eq!(json["success"], false);
        assert!(
            json["message"]
                .as_str()
                .unwrap()
                .contains("Password must be at least 8 characters")
        );
    }

    /// Test Case: User Registration with Duplicate Email
    #[tokio::test]
    async fn test_register_duplicate_email() {
        let (app, app_state) = make_test_app().await;

        let _existing_user = UserModel::create(
            app_state.db(),
            "existinguser",
            "duplicate@test.com",
            "existingpass",
            false,
        )
        .await
        .expect("Failed to create existing user");

        let payload = json!({"username": "newuser", "email": "duplicate@test.com", "password": "newuserpassword"});
        let uri = "/api/auth/register";
        let req = Request::builder()
            .method("POST")
            .uri(uri)
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::CONFLICT);

        let json = get_json_body(response).await;
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "A user with this email already exists");
    }

    /// Test Case: User Registration with Duplicate Username
    #[tokio::test]
    async fn test_register_duplicate_username() {
        let (app, app_state) = make_test_app().await;

        let _existing_user = UserModel::create(
            app_state.db(),
            "duplicateuser",
            "original@test.com",
            "existingpass",
            false,
        )
        .await
        .expect("Failed to create existing user");

        let payload = json!({"username": "duplicateuser", "email": "newuser@test.com", "password": "newuserpassword"});
        let uri = "/api/auth/register";
        let req = Request::builder()
            .method("POST")
            .uri(uri)
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::CONFLICT);

        let json = get_json_body(response).await;
        assert_eq!(json["success"], false);
        assert_eq!(
            json["message"],
            "A user with this student number already exists"
        );
    }

    /// Test Case: Successful User Login
    #[tokio::test]
    async fn test_login_success() {
        let (app, app_state) = make_test_app().await;

        let user_password = "correctpassword";
        let user = UserModel::create(
            app_state.db(),
            "logintestuser",
            "login@test.com",
            user_password,
            false,
        )
        .await
        .expect("Failed to create user for login");

        let payload = json!({"username": "logintestuser", "password": user_password});
        let uri = "/api/auth/login";
        let req = Request::builder()
            .method("POST")
            .uri(uri)
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let json = get_json_body(response).await;
        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "Login successful");
        let user_data = &json["data"];

        assert_eq!(user_data["username"], "logintestuser");
        assert_eq!(user_data["email"], "login@test.com");
        assert_eq!(user_data["admin"], false);

        assert!(user_data["token"].as_str().is_some());
        assert!(user_data["expires_at"].as_str().is_some());
        assert_eq!(user_data["id"], user.id);
    }

    /// Test Case: User Login with Invalid Credentials (Wrong Password)
    #[tokio::test]
    async fn test_login_invalid_password() {
        let (app, app_state) = make_test_app().await;

        let _user = UserModel::create(
            app_state.db(),
            "wrongpasstest",
            "wrongpass@test.com",
            "realpassword",
            false,
        )
        .await
        .expect("Failed to create user for login");

        let payload = json!({"username": "wrongpasstest", "password": "wrongpassword"});
        let uri = "/api/auth/login";
        let req = Request::builder()
            .method("POST")
            .uri(uri)
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let json = get_json_body(response).await;
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "Invalid student number or password");
    }

    /// Test Case: User Login with Non-Existent User
    #[tokio::test]
    async fn test_login_nonexistent_user() {
        let (app, _) = make_test_app().await;

        let payload = json!({"username": "nonexistentuser", "password": "anypassword"});
        let uri = "/api/auth/login";
        let req = Request::builder()
            .method("POST")
            .uri(uri)
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let json = get_json_body(response).await;
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "Invalid student number or password");
    }

    /// Test Case: Successful Password Reset Request
    #[tokio::test]
    async fn test_request_password_reset_success() {
        let (app, app_state) = make_test_app().await;

        let _user = UserModel::create(
            app_state.db(),
            "resetrequser",
            "resetreq@test.com",
            "oldpassword",
            false,
        )
        .await
        .expect("Failed to create user for reset request");

        let payload = json!({"email": "resetreq@test.com"});
        let uri = "/api/auth/request-password-reset";
        let req = Request::builder()
            .method("POST")
            .uri(uri)
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let json = get_json_body(response).await;
        assert_eq!(json["success"], true);
        assert_eq!(
            json["message"],
            "If the account exists, a reset link has been sent."
        );
        assert_eq!(json["data"], serde_json::Value::Null);
    }

    /// Test Case: Password Reset Request with Invalid Email Format
    #[tokio::test]
    async fn test_request_password_reset_invalid_email() {
        let (app, _) = make_test_app().await;

        let payload = json!({"email": "invalid-email-format"});
        let uri = "/api/auth/request-password-reset";
        let req = Request::builder()
            .method("POST")
            .uri(uri)
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let json = get_json_body(response).await;
        assert_eq!(json["success"], false);
        assert!(
            json["message"]
                .as_str()
                .unwrap()
                .contains("Invalid email format")
        );
    }

    /// Test Case: Successful Password Reset Token Verification
    #[tokio::test]
    async fn test_verify_reset_token_success() {
        let (app, app_state) = make_test_app().await;

        let user = UserModel::create(
            app_state.db(),
            "verifytokenuser",
            "verifytoken@test.com",
            "oldpassword",
            false,
        )
        .await
        .expect("Failed to create user for token verification");

        let token_model = PasswordResetTokenModel::create(app_state.db(), user.id, 15)
            .await
            .expect("Failed to create reset token");

        let payload = json!({"token": token_model.token});
        let uri = "/api/auth/verify-reset-token";
        let req = Request::builder()
            .method("POST")
            .uri(uri)
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let json = get_json_body(response).await;
        assert_eq!(json["success"], true);
        assert_eq!(
            json["message"],
            "Token verified. You may now reset your password."
        );
        let data = &json["data"];
        assert_eq!(data["email_hint"], "v***@test.com");
    }

    /// Test Case: Password Reset Token Verification with Invalid/Expired Token
    #[tokio::test]
    async fn test_verify_reset_token_invalid() {
        let (app, _) = make_test_app().await;

        let payload = json!({"token": "definitelynotavalidtoken123456"});
        let uri = "/api/auth/verify-reset-token";
        let req = Request::builder()
            .method("POST")
            .uri(uri)
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let json = get_json_body(response).await;
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "Invalid or expired token.");
    }

    /// Test Case: Successful Password Reset
    #[tokio::test]
    async fn test_reset_password_success() {
        let (app, app_state) = make_test_app().await;

        let original_password = "originalpassword";
        let user = UserModel::create(
            app_state.db(),
            "resetpassuser",
            "resetpass@test.com",
            original_password,
            false,
        )
        .await
        .expect("Failed to create user for password reset");

        let token_model = PasswordResetTokenModel::create(app_state.db(), user.id, 15)
            .await
            .expect("Failed to create reset token for password reset");

        let new_password = "brandnewsecurepassword";

        let payload = json!({"token": token_model.token, "new_password": new_password});
        let uri = "/api/auth/reset-password";
        let req = Request::builder()
            .method("POST")
            .uri(uri)
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.clone().oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let json = get_json_body(response).await;
        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "Password has been reset successfully.");
        assert_eq!(json["data"], serde_json::Value::Null);

        let login_payload_old = json!({"username": "resetpassuser", "password": original_password});
        let login_req_old = Request::builder()
            .method("POST")
            .uri("/api/auth/login")
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(
                serde_json::to_vec(&login_payload_old).unwrap(),
            ))
            .unwrap();

        let login_response_old = app.clone().oneshot(login_req_old).await.unwrap();
        assert_eq!(login_response_old.status(), StatusCode::UNAUTHORIZED);

        let login_payload_new = json!({"username": "resetpassuser", "password": new_password});
        let login_req_new = Request::builder()
            .method("POST")
            .uri("/api/auth/login")
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(
                serde_json::to_vec(&login_payload_new).unwrap(),
            ))
            .unwrap();

        let login_response_new = app.oneshot(login_req_new).await.unwrap();
        assert_eq!(login_response_new.status(), StatusCode::OK);
    }

    /// Test Case: Password Reset with Invalid Token
    #[tokio::test]
    async fn test_reset_password_invalid_token() {
        let (app, _) = make_test_app().await;

        let payload = json!({"token": "invalidresettoken123456", "new_password": "newpassword123"});
        let uri = "/api/auth/reset-password";
        let req = Request::builder()
            .method("POST")
            .uri(uri)
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let json = get_json_body(response).await;
        assert_eq!(json["success"], false);
        assert_eq!(
            json["message"],
            "Reset failed. The token may be invalid or expired."
        );
    }

    /// Test Case: Password Reset with Short New Password
    #[tokio::test]
    async fn test_reset_password_short_new_password() {
        let (app, app_state) = make_test_app().await;

        let user = UserModel::create(
            app_state.db(),
            "shortpassuser",
            "shortpass@test.com",
            "oldpass",
            false,
        )
        .await
        .expect("Failed to create user");
        let token_model = PasswordResetTokenModel::create(app_state.db(), user.id, 15)
            .await
            .expect("Failed to create token");

        let payload = json!({"token": token_model.token, "new_password": "short"});
        let uri = "/api/auth/reset-password";
        let req = Request::builder()
            .method("POST")
            .uri(uri)
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let json = get_json_body(response).await;
        assert_eq!(json["success"], false);
        assert!(
            json["message"]
                .as_str()
                .unwrap()
                .contains("Password must be at least 8 characters")
        );
    }

    /// Test Case: Successful Profile Picture Upload
    #[tokio::test]
    #[serial]
    async fn test_upload_profile_picture_success() {
        let (app, app_state, temp_dir) = make_test_app_with_storage().await;

        // Create a test user
        let user = UserModel::create(
            app_state.db(),
            "avataruser",
            "avatar@test.com",
            "avatarpass",
            false,
        )
        .await
        .expect("Failed to create user for avatar upload");

        // Generate an auth token
        let (token, _) = generate_jwt(user.id, user.admin);

        // Build multipart body with fake file content
        let boundary = "----WebKitFormBoundary7MA4YWxkTrZu0gW";
        let file_content = b"fake_jpeg_data_content";
        let multipart_body = format!(
            "--{}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"test_avatar.jpg\"\r\nContent-Type: image/jpeg\r\n\r\n{}\r\n--{}--\r\n",
            boundary,
            std::str::from_utf8(file_content).unwrap(),
            boundary
        );

        // Send request
        let req = Request::builder()
            .method("POST")
            .uri("/api/auth/upload-profile-picture")
            .header("Authorization", format!("Bearer {}", token))
            .header(
                CONTENT_TYPE,
                format!("multipart/form-data; boundary={}", boundary),
            )
            .body(AxumBody::from(multipart_body))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Parse JSON response
        let json = get_json_body(response).await;
        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "Profile picture uploaded.");
        assert!(json.get("data").is_none() || json["data"].is_null());

        // Verify file was written to STORAGE_ROOT/users/user_{id}/profile/avatar.jpg
        let full_path = temp_dir
            .path()
            .join("users")
            .join(format!("user_{}", user.id))
            .join("profile")
            .join("avatar.jpg");
        assert!(
            std::fs::metadata(&full_path).is_ok(),
            "Expected avatar file at {:?}",
            full_path
        );
        let saved_content = std::fs::read(&full_path).expect("Failed to read saved avatar file");
        assert_eq!(saved_content, file_content);
    }

    /// Test Case: Profile Picture Upload with Unsupported File Type
    #[tokio::test]
    #[serial]
    async fn test_upload_profile_picture_invalid_type() {
        let (app, app_state) = make_test_app().await;

        let user = UserModel::create(
            app_state.db(),
            "invalidtypeuser",
            "invalidtype@test.com",
            "pass",
            false,
        )
        .await
        .expect("Failed to create user");

        let temp_dir = tempdir().expect("Failed to create temp dir");
        unsafe {
            std::env::set_var(
                "USER_PROFILE_STORAGE_ROOT",
                temp_dir.path().to_str().unwrap(),
            );
        }

        let (token, _) = generate_jwt(user.id, user.admin);
        let boundary = "----InvalidTypeBoundary";
        let file_content = b"this is text content";
        let multipart_body = format!(
            "--{}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"badfile.txt\"\r\nContent-Type: text/plain\r\n\r\n{}\r\n--{}--\r\n",
            boundary,
            std::str::from_utf8(file_content).unwrap(),
            boundary
        );
        let uri = "/api/auth/upload-profile-picture";
        let req = Request::builder()
            .method("POST")
            .uri(uri)
            .header("Authorization", format!("Bearer {}", token))
            .header(
                CONTENT_TYPE,
                format!("multipart/form-data; boundary={}", boundary),
            )
            .body(AxumBody::from(multipart_body))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let json = get_json_body(response).await;
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "File type not supported.");
    }

    /// Test Case: Profile Picture Upload without Authentication (Forbidden)
    #[tokio::test]
    #[serial]
    async fn test_upload_profile_picture_missing_auth() {
        let (app, _) = make_test_app().await;

        let temp_dir = tempdir().expect("Failed to create temp dir");
        unsafe {
            std::env::set_var(
                "USER_PROFILE_STORAGE_ROOT",
                temp_dir.path().to_str().unwrap(),
            );
        }

        let boundary = "----NoAuthBoundary";
        let file_content = b"some_data";
        let multipart_body = format!(
            "--{}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"test.jpg\"\r\nContent-Type: image/jpeg\r\n\r\n{}\r\n--{}--\r\n",
            boundary,
            std::str::from_utf8(file_content).unwrap(),
            boundary
        );
        let uri = "/api/auth/upload-profile-picture";
        let req = Request::builder()
            .method("POST")
            .uri(uri)
            .header(
                CONTENT_TYPE,
                format!("multipart/form-data; boundary={}", boundary),
            )
            .body(AxumBody::from(multipart_body))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    /// Test Case: Successful Password Change
    #[tokio::test]
    async fn test_change_password_success() {
        let (app, app_state) = make_test_app().await;

        let original_password = "originalPassword123";
        let user = UserModel::create(
            app_state.db(),
            "changepassuser",
            "changepass@test.com",
            original_password,
            false,
        )
        .await
        .expect("Failed to create user for password change");

        let (token, _) = generate_jwt(user.id, user.admin);
        let payload = json!({
            "current_password": original_password,
            "new_password": "NewSecurePassword456"
        });
        let uri = "/api/auth/change-password";
        let req = Request::builder()
            .method("POST")
            .uri(uri)
            .header(CONTENT_TYPE, "application/json")
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.clone().oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let json = get_json_body(response).await;
        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "Password changed successfully.");

        let login_payload =
            json!({"username": "changepassuser", "password": "NewSecurePassword456"});
        let login_req = Request::builder()
            .method("POST")
            .uri("/api/auth/login")
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&login_payload).unwrap()))
            .unwrap();

        let login_response = app.clone().oneshot(login_req).await.unwrap();
        assert_eq!(login_response.status(), StatusCode::OK);

        let old_login_payload =
            json!({"username": "changepassuser", "password": original_password});
        let old_login_req = Request::builder()
            .method("POST")
            .uri("/api/auth/login")
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(
                serde_json::to_vec(&old_login_payload).unwrap(),
            ))
            .unwrap();

        let old_login_response = app.oneshot(old_login_req).await.unwrap();
        assert_eq!(old_login_response.status(), StatusCode::UNAUTHORIZED);
    }

    /// Test Case: Incorrect Current Password
    #[tokio::test]
    async fn test_change_password_incorrect_current() {
        let (app, app_state) = make_test_app().await;

        let user = UserModel::create(
            app_state.db(),
            "wrongpassuser",
            "wrongpass@test.com",
            "correctPassword",
            false,
        )
        .await
        .expect("Failed to create user");

        let (token, _) = generate_jwt(user.id, user.admin);
        let payload = json!({
            "current_password": "wrongCurrentPassword",
            "new_password": "NewPassword123"
        });
        let uri = "/api/auth/change-password";
        let req = Request::builder()
            .method("POST")
            .uri(uri)
            .header(CONTENT_TYPE, "application/json")
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let json = get_json_body(response).await;
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "Current password is incorrect");
    }

    /// Test Case: Short New Password
    #[tokio::test]
    async fn test_change_password_short_new() {
        let (app, app_state) = make_test_app().await;

        let user = UserModel::create(
            app_state.db(),
            "shortpassuser",
            "shortpass@test.com",
            "currentPassword",
            false,
        )
        .await
        .expect("Failed to create user");

        let (token, _) = generate_jwt(user.id, user.admin);
        let payload = json!({
            "current_password": "currentPassword",
            "new_password": "short"
        });
        let uri = "/api/auth/change-password";
        let req = Request::builder()
            .method("POST")
            .uri(uri)
            .header(CONTENT_TYPE, "application/json")
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let json = get_json_body(response).await;
        assert_eq!(json["success"], false);
        assert!(
            json["message"]
                .as_str()
                .unwrap()
                .contains("Password must be at least 8 characters")
        );
    }

    /// Test Case: Missing Authentication Token
    #[tokio::test]
    async fn test_change_password_missing_token() {
        let (app, _) = make_test_app().await;

        let payload = json!({
            "current_password": "anyPassword",
            "new_password": "NewPassword123"
        });
        let uri = "/api/auth/change-password";
        let req = Request::builder()
            .method("POST")
            .uri(uri)
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let json = get_json_body(response).await;
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "Authentication required");
    }

    /// Test Case: Invalid Authentication Token
    #[tokio::test]
    async fn test_change_password_invalid_token() {
        let (app, app_state) = make_test_app().await;

        let user = UserModel::create(
            app_state.db(),
            "invalidtokenuser",
            "invalidtoken@test.com",
            "password",
            false,
        )
        .await
        .expect("Failed to create user");

        let (mut token, _) = generate_jwt(user.id, user.admin);
        token.push_str("invalid");
        let payload = json!({
            "current_password": "anyPassword",
            "new_password": "NewPassword123"
        });
        let uri = "/api/auth/change-password";
        let req = Request::builder()
            .method("POST")
            .uri(uri)
            .header(CONTENT_TYPE, "application/json")
            .header("Authorization", format!("Bearer {}", token))
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let json = get_json_body(response).await;
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "Authentication required");
    }
}
