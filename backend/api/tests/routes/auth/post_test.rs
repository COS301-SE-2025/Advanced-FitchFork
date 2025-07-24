#[cfg(test)]
mod tests {
    // Import necessary modules and functions, mirroring patterns from other test files
    use db::{
        test_utils::setup_test_db,
        models::{
            user::Model as UserModel,
            password_reset_token::Model as PasswordResetTokenModel,
        },
    };
    use axum::{
        body::Body as AxumBody, // Alias for clarity if needed
        http::{Request, StatusCode, header::CONTENT_TYPE},
        response::Response,
    };
    use tower::ServiceExt; // For calling `oneshot` on the app
    use serde_json::{Value, json};
    use api::auth::generate_jwt; // Assuming this exists for creating test tokens
    use crate::test_helpers::make_app; // Assuming this sets up the full app including auth routes
    use tempfile::tempdir;
    use serial_test::serial; // Assuming serial execution is needed if tests modify global state or env vars

    // Helper function to extract JSON body from response
    async fn get_json_body(response: Response) -> Value {
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        serde_json::from_slice(&body).unwrap()
    }

    /// Test Case: Successful User Registration
    #[tokio::test]
    #[serial]
    async fn test_register_success() {
        dotenvy::dotenv().ok(); // Load .env variables
        let db = setup_test_db().await;

        let app = make_app(db.clone());

        let payload = json!({
            "username": "testuser123",
            "email": "testuser123@example.com",
            "password": "securepassword123"
        });

        let uri = "/api/auth/register";
        let req = Request::builder()
            .method("POST")
            .uri(uri)
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED); // Expect 201 Created

        let json = get_json_body(response).await;
        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "User registered successfully");
        let user_data = &json["data"];

        assert_eq!(user_data["username"], "testuser123");
        assert_eq!(user_data["email"], "testuser123@example.com");
        assert_eq!(user_data["admin"], false);
        // Check that token and expiry are present
        assert!(user_data["token"].as_str().is_some());
        assert!(user_data["expires_at"].as_str().is_some());
        assert!(user_data["id"].as_i64().is_some());
    }

    /// Test Case: User Registration with Invalid Email
    #[tokio::test]
    #[serial]
    async fn test_register_invalid_email() {
        dotenvy::dotenv().ok();
        let db = setup_test_db().await;
        let app = make_app(db.clone());

        let payload = json!({
            "username": "testuser456",
            "email": "not-an-email", // Invalid email
            "password": "securepassword456"
        });

        let uri = "/api/auth/register";
        let req = Request::builder()
            .method("POST")
            .uri(uri)
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST); // Expect 400 Bad Request

        let json = get_json_body(response).await;
        assert_eq!(json["success"], false);
        // Message depends on validation implementation, likely mentions email format
        assert!(json["message"].as_str().unwrap().contains("email")); // Check for email-related error
    }

    /// Test Case: User Registration with Short Password
    #[tokio::test]
    #[serial]
    async fn test_register_short_password() {
        dotenvy::dotenv().ok();
        let db = setup_test_db().await;
        let app = make_app(db.clone());

        let payload = json!({
            "username": "testuser789",
            "email": "testuser789@example.com",
            "password": "short" // Too short, assuming min 8 chars
        });

        let uri = "/api/auth/register";
        let req = Request::builder()
            .method("POST")
            .uri(uri)
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST); // Expect 400 Bad Request

        let json = get_json_body(response).await;
        assert_eq!(json["success"], false);
        // Message depends on validation, likely mentions password length
        assert!(json["message"].as_str().unwrap().contains("Password must be at least 8 characters"));
    }

    /// Test Case: User Registration with Duplicate Email
    #[tokio::test]
    #[serial]
    async fn test_register_duplicate_email() {
        dotenvy::dotenv().ok();
        let db = setup_test_db().await;

        // Pre-create a user with the email
        let _existing_user = UserModel::create(&db, "existinguser", "duplicate@test.com", "existingpass", false)
            .await
            .expect("Failed to create existing user");

        let app = make_app(db.clone());

        let payload = json!({
            "username": "newuser",
            "email": "duplicate@test.com", // Duplicate email
            "password": "newuserpassword"
        });

        let uri = "/api/auth/register";
        let req = Request::builder()
            .method("POST")
            .uri(uri)
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::CONFLICT); // Expect 409 Conflict

        let json = get_json_body(response).await;
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "A user with this email already exists");
    }

    /// Test Case: User Registration with Duplicate Username
    #[tokio::test]
    #[serial]
    async fn test_register_duplicate_username() {
        dotenvy::dotenv().ok();
        let db = setup_test_db().await;

        // Pre-create a user with the username
        let _existing_user = UserModel::create(&db, "duplicateuser", "original@test.com", "existingpass", false)
            .await
            .expect("Failed to create existing user");

        let app = make_app(db.clone());

        let payload = json!({
            "username": "duplicateuser", // Duplicate username
            "email": "newuser@test.com",
            "password": "newuserpassword"
        });

        let uri = "/api/auth/register";
        let req = Request::builder()
            .method("POST")
            .uri(uri)
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::CONFLICT); // Expect 409 Conflict

        let json = get_json_body(response).await;
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "A user with this student number already exists"); // Assuming message format
    }

    /// Test Case: Successful User Login
    #[tokio::test]
    #[serial]
    async fn test_login_success() {
        dotenvy::dotenv().ok();
        let db = setup_test_db().await;

        // Pre-create a user to login as
        let user_password = "correctpassword";
        let user = UserModel::create(&db, "logintestuser", "login@test.com", user_password, false)
            .await
            .expect("Failed to create user for login");

        let app = make_app(db.clone());

        let payload = json!({
            "username": "logintestuser",
            "password": user_password // Correct password
        });

        let uri = "/api/auth/login";
        let req = Request::builder()
            .method("POST")
            .uri(uri)
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK); // Expect 200 OK

        let json = get_json_body(response).await;
        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "Login successful");
        let user_data = &json["data"];

        assert_eq!(user_data["username"], "logintestuser");
        assert_eq!(user_data["email"], "login@test.com");
        assert_eq!(user_data["admin"], false);
        // Check that token and expiry are present
        assert!(user_data["token"].as_str().is_some());
        assert!(user_data["expires_at"].as_str().is_some());
        assert_eq!(user_data["id"], user.id);
    }

    /// Test Case: User Login with Invalid Credentials (Wrong Password)
    #[tokio::test]
    #[serial]
    async fn test_login_invalid_password() {
        dotenvy::dotenv().ok();
        let db = setup_test_db().await;

        // Pre-create a user
        let _user = UserModel::create(&db, "wrongpasstest", "wrongpass@test.com", "realpassword", false)
            .await
            .expect("Failed to create user for login");

        let app = make_app(db.clone());

        let payload = json!({
            "username": "wrongpasstest",
            "password": "wrongpassword" // Incorrect password
        });

        let uri = "/api/auth/login";
        let req = Request::builder()
            .method("POST")
            .uri(uri)
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED); // Expect 401 Unauthorized

        let json = get_json_body(response).await;
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "Invalid student number or password"); // Assuming message format
    }

    /// Test Case: User Login with Non-Existent User
    #[tokio::test]
    #[serial]
    async fn test_login_nonexistent_user() {
        dotenvy::dotenv().ok();
        let db = setup_test_db().await;
        let app = make_app(db.clone());

        let payload = json!({
            "username": "nonexistentuser",
            "password": "anypassword"
        });

        let uri = "/api/auth/login";
        let req = Request::builder()
            .method("POST")
            .uri(uri)
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED); // Expect 401 Unauthorized

        let json = get_json_body(response).await;
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "Invalid student number or password"); // Assuming message format
    }

    /// Test Case: Successful Password Reset Request
    #[tokio::test]
    #[serial]
    // Note: This test might be complex if EmailService is involved. Consider mocking or using a test email backend.
    async fn test_request_password_reset_success() {
        dotenvy::dotenv().ok();
        let db = setup_test_db().await;

        // Pre-create a user whose email we will request a reset for
        let user = UserModel::create(&db, "resetrequser", "resetreq@test.com", "oldpassword", false)
            .await
            .expect("Failed to create user for reset request");

        let app = make_app(db.clone());

        let payload = json!({
            "email": "resetreq@test.com" // Email of existing user
        });

        let uri = "/api/auth/request-password-reset";
        let req = Request::builder()
            .method("POST")
            .uri(uri)
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        // Handler returns 200 OK even if email is sent or not, for security
        assert_eq!(response.status(), StatusCode::OK);

        let json = get_json_body(response).await;
        assert_eq!(json["success"], true);
        // Standard message regardless of email existence (for security)
        assert_eq!(json["message"], "If the account exists, a reset link has been sent.");
        // Data is typically null for this endpoint
        assert_eq!(json["data"], serde_json::Value::Null);

        // Optionally, verify a token was created in the database for the user
        // This requires querying the password_reset_token table.
        /*
        let tokens = PasswordResetTokenModel::Entity::find()
            .filter(PasswordResetTokenModel::Column::UserId.eq(user.id))
            .all(&db)
            .await
            .expect("Failed to query reset tokens");
        assert!(!tokens.is_empty()); // At least one token should exist
        */
    }

    /// Test Case: Password Reset Request with Invalid Email Format
    #[tokio::test]
    #[serial]
    async fn test_request_password_reset_invalid_email() {
        dotenvy::dotenv().ok();
        let db = setup_test_db().await;
        let app = make_app(db.clone());

        let payload = json!({
            "email": "invalid-email-format" // Invalid email
        });

        let uri = "/api/auth/request-password-reset";
        let req = Request::builder()
            .method("POST")
            .uri(uri)
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST); // Expect 400 Bad Request

        let json = get_json_body(response).await;
        assert_eq!(json["success"], false);
        // Message depends on validation
        assert!(json["message"].as_str().unwrap().contains("Invalid email format"));
    }

     /// Test Case: Successful Password Reset Token Verification
    #[tokio::test]
    #[serial]
    async fn test_verify_reset_token_success() {
        dotenvy::dotenv().ok();
        let db = setup_test_db().await;

        // Pre-create a user
        let user = UserModel::create(&db, "verifytokenuser", "verifytoken@test.com", "oldpassword", false)
            .await
            .expect("Failed to create user for token verification");

        // Pre-create a valid reset token for the user
        let token_model = PasswordResetTokenModel::create(&db, user.id, 15) // 15 min expiry
            .await
            .expect("Failed to create reset token");

        let app = make_app(db.clone());

        let payload = json!({
            "token": token_model.token // Use the valid token
        });

        let uri = "/api/auth/verify-reset-token";
        let req = Request::builder()
            .method("POST")
            .uri(uri)
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK); // Expect 200 OK

        let json = get_json_body(response).await;
        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "Token verified. You may now reset your password.");
        let data = &json["data"];
        // Check email hint (masked)
        assert_eq!(data["email_hint"], "v***@test.com"); // Based on handler logic
    }

    /// Test Case: Password Reset Token Verification with Invalid/Expired Token
    #[tokio::test]
    #[serial]
    async fn test_verify_reset_token_invalid() {
        dotenvy::dotenv().ok();
        let db = setup_test_db().await;
        let app = make_app(db.clone());

        let payload = json!({
            "token": "definitelynotavalidtoken123456" // Invalid token
        });

        let uri = "/api/auth/verify-reset-token";
        let req = Request::builder()
            .method("POST")
            .uri(uri)
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST); // Expect 400 Bad Request

        let json = get_json_body(response).await;
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "Invalid or expired token."); // Standard error message
    }

    /// Test Case: Successful Password Reset
    #[tokio::test]
    #[serial]
    // Note: This test changes the user's password. Subsequent login tests should use the new password.
    async fn test_reset_password_success() {
        dotenvy::dotenv().ok();
        let db = setup_test_db().await;

        // Pre-create a user
        let original_password = "originalpassword";
        let user = UserModel::create(&db, "resetpassuser", "resetpass@test.com", original_password, false)
            .await
            .expect("Failed to create user for password reset");

        // Pre-create a valid reset token for the user
        let token_model = PasswordResetTokenModel::create(&db, user.id, 15) // 15 min expiry
            .await
            .expect("Failed to create reset token for password reset");

        let new_password = "brandnewsecurepassword";
        let app = make_app(db.clone());

        let payload = json!({
            "token": token_model.token, // Valid token
            "new_password": new_password // New password
        });

        let uri = "/api/auth/reset-password";
        let req = Request::builder()
            .method("POST")
            .uri(uri)
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.clone().oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK); // Expect 200 OK

        let json = get_json_body(response).await;
        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "Password has been reset successfully.");
        assert_eq!(json["data"], serde_json::Value::Null);

        // --- Optional: Verify password was actually changed ---
        // Try to login with the OLD password - should fail
        let login_payload_old = json!({
            "username": "resetpassuser",
            "password": original_password
        });
        let login_req_old = Request::builder()
            .method("POST")
            .uri("/api/auth/login")
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&login_payload_old).unwrap()))
            .unwrap();
        let login_response_old = app.clone().oneshot(login_req_old).await.unwrap();
        assert_eq!(login_response_old.status(), StatusCode::UNAUTHORIZED);

        // Try to login with the NEW password - should succeed
        let login_payload_new = json!({
            "username": "resetpassuser",
            "password": new_password // New password
        });
        let login_req_new = Request::builder()
            .method("POST")
            .uri("/api/auth/login")
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&login_payload_new).unwrap()))
            .unwrap();
        let login_response_new = app.oneshot(login_req_new).await.unwrap();
        assert_eq!(login_response_new.status(), StatusCode::OK);
    }

    /// Test Case: Password Reset with Invalid Token
    #[tokio::test]
    #[serial]
    async fn test_reset_password_invalid_token() {
        dotenvy::dotenv().ok();
        let db = setup_test_db().await;
        let app = make_app(db.clone());

        let payload = json!({
            "token": "invalidresettoken123456", // Invalid token
            "new_password": "newpassword123"
        });

        let uri = "/api/auth/reset-password";
        let req = Request::builder()
            .method("POST")
            .uri(uri)
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST); // Expect 400 Bad Request

        let json = get_json_body(response).await;
        assert_eq!(json["success"], false);
        // Standard error message for invalid/expired token during reset
        assert_eq!(json["message"], "Reset failed. The token may be invalid or expired.");
    }

    /// Test Case: Password Reset with Short New Password
    #[tokio::test]
    #[serial]
    async fn test_reset_password_short_new_password() {
        dotenvy::dotenv().ok();
        let db = setup_test_db().await;

        // Pre-create a user and token (token validity isn't checked first)
        let user = UserModel::create(&db, "shortpassuser", "shortpass@test.com", "oldpass", false)
            .await
            .expect("Failed to create user");
        let token_model = PasswordResetTokenModel::create(&db, user.id, 15)
            .await
            .expect("Failed to create token");

        let app = make_app(db.clone());

        let payload = json!({
            "token": token_model.token,
            "new_password": "short" // Too short password
        });

        let uri = "/api/auth/reset-password";
        let req = Request::builder()
            .method("POST")
            .uri(uri)
            .header(CONTENT_TYPE, "application/json")
            .body(AxumBody::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST); // Expect 400 Bad Request

        let json = get_json_body(response).await;
        assert_eq!(json["success"], false);
        // Message depends on validation
        assert!(json["message"].as_str().unwrap().contains("Password must be at least 8 characters"));
    }

    /// Test Case: Successful Profile Picture Upload
    #[tokio::test]
    #[serial]
    async fn test_upload_profile_picture_success() {
        dotenvy::dotenv().ok();
        let db = setup_test_db().await;

        // Pre-create a user for whom to upload the picture
        let user = UserModel::create(&db, "avataruser", "avatar@test.com", "avatarpass", false)
            .await
            .expect("Failed to create user for avatar upload");

        let temp_dir = tempdir().expect("Failed to create temporary directory for avatars");
        // Set the avatar storage root environment variable for tests
        unsafe {
            std::env::set_var("USER_PROFILE_STORAGE_ROOT", temp_dir.path().to_str().unwrap());
        }

        let app = make_app(db.clone());
        // Generate JWT for the user
        let (token, _) = generate_jwt(user.id, user.admin);

        // Create mock multipart data
        // This is a simplified simulation. Real multipart/form-data is more complex.
        // For thorough testing, consider using a library like `multer` or crafting raw multipart bodies.
        // Here, we simulate a basic JPEG upload.
        let boundary = "----WebKitFormBoundary7MA4YWxkTrZu0gW";
        let file_content = b"fake_jpeg_data_content"; // Not a real JPEG, but sufficient for testing file handling logic
        let multipart_body = format!(
            "--{}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"test_avatar.jpg\"\r\nContent-Type: image/jpeg\r\n\r\n{}\r\n--{}--\r\n",
            boundary,
            std::str::from_utf8(file_content).unwrap(), // Safe for our fake data
            boundary
        );

        let uri = "/api/auth/upload-profile-picture";
        let req = Request::builder()
            .method("POST")
            .uri(uri)
            .header("Authorization", format!("Bearer {}", token)) // Auth required
            .header(CONTENT_TYPE, format!("multipart/form-data; boundary={}", boundary))
            .body(AxumBody::from(multipart_body))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK); // Expect 200 OK

        let json = get_json_body(response).await;
        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "Profile picture uploaded.");
        let data = &json["data"];
        let path = data["profile_picture_path"].as_str().expect("Path should be a string");

        // Assert the path format (e.g., contains user ID)
        assert!(path.starts_with(&format!("user_{}", user.id)), "Path '{}' does not start with expected prefix 'user_{}'", path, user.id);
        assert!(path.ends_with(".jpg")); // Check extension based on content type

        // Optionally, verify the file was created in the temp directory
        // This requires parsing the relative path and checking the temp dir
        /*
        let full_path = temp_dir.path().join(path);
        assert!(std::fs::metadata(&full_path).is_ok()); // File should exist
        let saved_content = std::fs::read(&full_path).expect("Failed to read saved avatar file");
        assert_eq!(saved_content, file_content); // Content should match
        */
    }

    /// Test Case: Profile Picture Upload with Unsupported File Type
    #[tokio::test]
    #[serial]
    async fn test_upload_profile_picture_invalid_type() {
        dotenvy::dotenv().ok();
        let db = setup_test_db().await;
        let user = UserModel::create(&db, "invalidtypeuser", "invalidtype@test.com", "pass", false)
            .await
            .expect("Failed to create user");
        let temp_dir = tempdir().expect("Failed to create temp dir");
        unsafe {
            std::env::set_var("USER_PROFILE_STORAGE_ROOT", temp_dir.path().to_str().unwrap());
        }

        let app = make_app(db.clone());
        let (token, _) = generate_jwt(user.id, user.admin);

        let boundary = "----InvalidTypeBoundary";
        // Simulate uploading a text file
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
            .header(CONTENT_TYPE, format!("multipart/form-data; boundary={}", boundary))
            .body(AxumBody::from(multipart_body))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST); // Expect 400 Bad Request

        let json = get_json_body(response).await;
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "File type not supported."); // Error from handler
    }

    /// Test Case: Profile Picture Upload without Authentication (Forbidden)
    #[tokio::test]
    #[serial]
    async fn test_upload_profile_picture_missing_auth() {
        dotenvy::dotenv().ok();
        let db = setup_test_db().await;
        let temp_dir = tempdir().expect("Failed to create temp dir");
        unsafe {
            std::env::set_var("USER_PROFILE_STORAGE_ROOT", temp_dir.path().to_str().unwrap());
        }

        let app = make_app(db.clone());

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
            // No Authorization header
            .header(CONTENT_TYPE, format!("multipart/form-data; boundary={}", boundary))
            .body(AxumBody::from(multipart_body))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        // Expect 401 Unauthorized or 403 Forbidden, depending on AuthUser extractor
        // Based on previous auth tests and the extractor code, it should be 401.
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED); // Or FORBIDDEN
    }

    // Note: Testing for 500 Internal Server Error for database failures
    // typically requires mocking the database layer or specific model functions
    // to return errors. This is more complex and often done with specific
    // mocking libraries. Consider adding such tests if comprehensive coverage is required.

    // --- Add more tests as needed, e.g., for specific edge cases in handlers ---
}