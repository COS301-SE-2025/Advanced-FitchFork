use axum::{
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use sea_orm::{EntityTrait, ColumnTrait, QueryFilter, PaginatorTrait, ActiveModelTrait, ActiveValue::Set};

use serde::{Deserialize, Serialize};
use validator::Validate;
use chrono::{Utc, Duration};

use crate::{
    auth::generate_jwt,
    response::ApiResponse,
    services::email::EmailService,
};

use db::{
    models::{
        user::{self, Model as UserModel},
        password_reset_token::{self, Model as PasswordResetTokenModel}
    },
    connect
};

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(regex(
        path = "STUDENT_NUMBER_REGEX",
        message = "Student number must be in format u12345678"
    ))]
    pub student_number: String,

    #[validate(email(message = "Invalid email format"))]
    pub email: String,

    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,

    // TODO: Add some more password validation later
}

#[derive(Debug, Serialize, Default)]
pub struct UserResponse {
    pub id: i64,
    pub student_number: String,
    pub email: String,
    pub admin: bool,
    pub token: String,
    pub expires_at: String,
}

lazy_static::lazy_static! {
    static ref STUDENT_NUMBER_REGEX: regex::Regex = regex::Regex::new("^u\\d{8}$").unwrap();
}

/// POST /auth/register
///
/// Register a new user.
///
/// ### Request Body
/// ```json
/// {
///   "student_number": "u12345678",
///   "email": "user@example.com",
///   "password": "strongpassword"
/// }
/// ```
///
/// ### Responses
///
/// - `201 Created`  
/// ```json
/// {
///   "success": true,
///   "data": {
///     "id": 1,
///     "student_number": "u12345678",
///     "email": "user@example.com",
///     "admin": false,
///     "token": "jwt_token_here",
///     "expires_at": "2025-05-23T11:00:00Z"
///   },
///   "message": "User registered successfully"
/// }
/// ```
///
/// - `400 Bad Request` (validation failure)  
/// ```json
/// {
///   "success": false,
///   "message": "Student number must be in format u12345678"
/// }
/// ```
///
/// - `409 Conflict` (duplicate)  
/// ```json
/// {
///   "success": false,
///   "message": "A user with this email already exists"
/// }
/// ```
///
/// - `500 Internal Server Error`  
/// ```json
/// {
///   "success": false,
///   "message": "Database error: detailed error here"
/// }
/// ```
pub async fn register(Json(req): Json<RegisterRequest>) -> impl IntoResponse {
    if let Err(validation_errors) = req.validate() {
        let error_message = common::format_validation_errors(&validation_errors);
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<UserResponse>::error(error_message)),
        );
    }

    let db = connect().await;

    let email_exists = user::Entity::find()
        .filter(user::Column::Email.eq(req.email.clone()))
        .one(&db)
        .await
        .unwrap();

    if email_exists.is_some() {
        return (
            StatusCode::CONFLICT,
            Json(ApiResponse::<UserResponse>::error("A user with this email already exists")),
        );
    }

    let sn_exists = user::Entity::find()
        .filter(user::Column::StudentNumber.eq(req.student_number.clone()))
        .one(&db)
        .await
        .unwrap();

    if sn_exists.is_some() {
        return (
            StatusCode::CONFLICT,
            Json(ApiResponse::<UserResponse>::error("A user with this student number already exists")),
        );
    }

    let inserted_user = match UserModel::create(
        &db,
        &req.student_number,
        &req.email,
        &req.password,
        false,
    ).await {
        Ok(user) => user,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<UserResponse>::error(format!("Database error: {}", e))),
            );
        }
    };

    let (token, expiry) = generate_jwt(inserted_user.id, inserted_user.admin);
    let user_response = UserResponse {
        id: inserted_user.id,
        student_number: inserted_user.student_number,
        email: inserted_user.email,
        admin: inserted_user.admin,
        token,
        expires_at: expiry,
    };

    (
        StatusCode::CREATED,
        Json(ApiResponse::success(user_response, "User registered successfully")),
    )
}


#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    pub student_number: String,
    pub password: String,
}

/// POST /auth/login
///
/// Authenticate an existing user and issue a JWT.
///
/// ### Request Body
/// ```json
/// {
///   "student_number": "u12345678",
///   "password": "strongpassword"
/// }
/// ```
///
/// ### Responses
///
/// - `200 OK`  
/// ```json
/// {
///   "success": true,
///   "data": {
///     "id": 1,
///     "student_number": "u12345678",
///     "email": "user@example.com",
///     "admin": false,
///     "token": "jwt_token_here",
///     "expires_at": "2025-05-23T12:00:00Z"
///   },
///   "message": "Login successful"
/// }
/// ```
///
/// - `401 Unauthorized` (invalid credentials)  
/// ```json
/// {
///   "success": false,
///   "message": "Invalid password"
/// }
/// ```
///
/// - `500 Internal Server Error`  
/// ```json
/// {
///   "success": false,
///   "message": "Database error: detailed error here"
/// }
/// ```
pub async fn login(Json(req): Json<LoginRequest>) -> impl IntoResponse {
    if let Err(validation_errors) = req.validate() {
        let error_message = common::format_validation_errors(&validation_errors);
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<UserResponse>::error(error_message)),
        );
    }

    let db = connect().await;

    let user = match UserModel::verify_credentials(&db, &req.student_number, &req.password).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ApiResponse::<UserResponse>::error("Invalid student number or password")),
            );
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<UserResponse>::error(format!("Database error: {}", e))),
            );
        }
    };

    let (token, expiry) = generate_jwt(user.id, user.admin);
    let user_response = UserResponse {
        id: user.id,
        student_number: user.student_number,
        email: user.email,
        admin: user.admin,
        token,
        expires_at: expiry,
    };

    (
        StatusCode::OK,
        Json(ApiResponse::success(user_response, "Login successful")),
    )
}

#[derive(Debug, Deserialize, Validate)]
pub struct RequestPasswordResetRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
}

/// POST /auth/request-password-reset
///
/// Request a password reset token for a user.
///
/// ### Request Body
/// ```json
/// {
///   "email": "user@example.com"
/// }
/// ```
///
/// ### Responses
///
/// - `200 OK`  
/// ```json
/// {
///   "success": true,
///   "data": null,
///   "message": "If the account exists, a reset link has been sent."
/// }
/// ```
///
/// - `400 Bad Request` (validation failure)  
/// ```json
/// {
///   "success": false,
///   "message": "Invalid email format"
/// }
/// ```
///
/// - `429 Too Many Requests`  
/// ```json
/// {
///   "success": false,
///   "message": "Too many password reset requests. Please try again later."
/// }
/// ```
///
/// - `500 Internal Server Error`  
/// ```json
/// {
///   "success": false,
///   "message": "Database error: detailed error here"
/// }
/// ```
pub async fn request_password_reset(Json(req): Json<RequestPasswordResetRequest>) -> impl IntoResponse {
    if let Err(validation_errors) = req.validate() {
        let error_message = common::format_validation_errors(&validation_errors);
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(error_message)),
        );
    }

    let db = connect().await;

    let user = match user::Entity::find()
        .filter(user::Column::Email.eq(req.email.clone()))
        .one(&db)
        .await
    {
        Ok(Some(u)) => u,
        Ok(None) => {
            return (
                StatusCode::OK,
                Json(ApiResponse::success(
                    (),
                    "If the account exists, a reset link has been sent.",
                )),
            );
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(format!("Database error: {}", e))),
            );
        }
    };

    let one_hour_ago = Utc::now() - Duration::hours(1);
    let recent_requests = password_reset_token::Entity::find()
        .filter(password_reset_token::Column::UserId.eq(user.id))
        .filter(password_reset_token::Column::CreatedAt.gt(one_hour_ago))
        .count(&db)
        .await
        .unwrap_or(0);

    let max_requests = std::env::var("MAX_PASSWORD_RESET_REQUESTS_PER_HOUR")
        .unwrap_or_else(|_| "3".to_string())
        .parse::<u64>()
        .unwrap_or(3);

    if recent_requests >= max_requests {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(ApiResponse::<()>::error(
                "Too many password reset requests. Please try again later.",
            )),
        );
    }

    let expiry_minutes = std::env::var("RESET_TOKEN_EXPIRY_MINUTES")
        .unwrap_or_else(|_| "15".to_string())
        .parse::<i64>()
        .unwrap_or(15);

    match PasswordResetTokenModel::create(&db, user.id, expiry_minutes).await {
        Ok(token) => {
            match EmailService::send_password_reset_email(&user.email, &token.token).await {
                Ok(_) => (
                    StatusCode::OK,
                    Json(ApiResponse::success(
                        (),
                        "If the account exists, a reset link has been sent.",
                    )),
                ),
                Err(e) => {
                    eprintln!("Failed to send password reset email: {}", e);
                    (
                        StatusCode::OK,
                        Json(ApiResponse::success(
                            (),
                            "If the account exists, a reset link has been sent.",
                        )),
                    )
                }
            }
        }
        Err(e) => {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(format!("Database error: {}", e))),
            )
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct VerifyResetTokenRequest {
    #[validate(length(min = 1, message = "Token is required"))]
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct VerifyResetTokenResponse {
    pub email_hint: String,
}

/// POST /auth/verify-reset-token
///
/// Verify the validity of a password reset token.
///
/// ### Request Body
/// ```json
/// {
///   "token": "abcdef123456"
/// }
/// ```
///
/// ### Responses
///
/// - `200 OK`  
/// ```json
/// {
///   "success": true,
///   "data": {
///     "email_hint": "u***@example.com"
///   },
///   "message": "Token verified. You may now reset your password."
/// }
/// ```
///
/// - `400 Bad Request` (validation failure)  
/// ```json
/// {
///   "success": false,
///   "message": "Token is required"
/// }
/// ```
///
/// - `400 Bad Request` (invalid token)  
/// ```json
/// {
///   "success": false,
///   "message": "Invalid or expired token."
/// }
/// ```
pub async fn verify_reset_token(Json(req): Json<VerifyResetTokenRequest>) -> impl IntoResponse {
    if let Err(validation_errors) = req.validate() {
        let error_message = common::format_validation_errors(&validation_errors);
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<VerifyResetTokenResponse>::error(error_message)),
        );
    }

    let db = connect().await;

    match PasswordResetTokenModel::find_valid_token(&db, &req.token).await {
        Ok(Some(token)) => {
            match user::Entity::find_by_id(token.user_id).one(&db).await {
                Ok(Some(user)) => {
                    let email_parts: Vec<&str> = user.email.split('@').collect();
                    let username = email_parts[0];
                    let domain = email_parts[1];
                    let masked_username = format!("{}***", &username[0..1]);
                    let email_hint = format!("{}@{}", masked_username, domain);

                    let response = VerifyResetTokenResponse { email_hint };
                    (
                        StatusCode::OK,
                        Json(ApiResponse::success(
                            response,
                            "Token verified. You may now reset your password.",
                        )),
                    )
                }
                Ok(None) => (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::<VerifyResetTokenResponse>::error("Invalid or expired token.")),
                ),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<VerifyResetTokenResponse>::error(format!("Database error: {}", e))),
                ),
            }
        }
        Ok(None) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<VerifyResetTokenResponse>::error("Invalid or expired token.")),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<VerifyResetTokenResponse>::error(format!("Database error: {}", e))),
        ),
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct ResetPasswordRequest {
    #[validate(length(min = 1, message = "Token is required"))]
    pub token: String,

    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub new_password: String,
}

/// POST /auth/reset-password
///
/// Reset a user's password using a valid reset token.
///
/// ### Request Body
/// ```json
/// {
///   "token": "abcdef123456",
///   "new_password": "SecureP@ssw0rd!"
/// }
/// ```
///
/// ### Responses
///
/// - `200 OK`  
/// ```json
/// {
///   "success": true,
///   "data": null,
///   "message": "Password has been reset successfully."
/// }
/// ```
///
/// - `400 Bad Request` (validation failure)  
/// ```json
/// {
///   "success": false,
///   "message": "Password must be at least 8 characters"
/// }
/// ```
///
/// - `400 Bad Request` (invalid token)  
/// ```json
/// {
///   "success": false,
///   "message": "Reset failed. The token may be invalid or expired."
/// }
/// ```
pub async fn reset_password(Json(req): Json<ResetPasswordRequest>) -> impl IntoResponse {
    if let Err(validation_errors) = req.validate() {
        let error_message = common::format_validation_errors(&validation_errors);
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(error_message)),
        );
    }

    let db = connect().await;

    match PasswordResetTokenModel::find_valid_token(&db, &req.token).await {
        Ok(Some(token)) => {
            match user::Entity::find_by_id(token.user_id).one(&db).await {
                Ok(Some(user)) => {
                    let user_email = user.email.clone();
                    
                    let mut active_model: user::ActiveModel = user.into();
                    active_model.password_hash = Set(UserModel::hash_password(&req.new_password));
                    
                    match active_model.update(&db).await {
                        Ok(_) => {
                            if let Err(e) = token.mark_as_used(&db).await {
                                eprintln!("Failed to mark token as used: {}", e);
                            }

                            if let Err(e) = EmailService::send_password_changed_email(&user_email).await {
                                eprintln!("Failed to send password change confirmation email: {}", e);
                            }

                            (
                                StatusCode::OK,
                                Json(ApiResponse::success(
                                    (),
                                    "Password has been reset successfully.",
                                )),
                            )
                        }
                        Err(e) => (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(ApiResponse::<()>::error(format!("Database error: {}", e))),
                        ),
                    }
                }
                Ok(None) => (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::<()>::error("Reset failed. The token may be invalid or expired.")),
                ),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()>::error(format!("Database error: {}", e))),
                ),
            }
        }
        Ok(None) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error("Reset failed. The token may be invalid or expired.")),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Database error: {}", e))),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::DatabaseConnection;
    use crate::auth::claims::Claims;
    use db::models::user::Model as UserModel;
    use db::test_utils::setup_test_db;


    // Helper function to create test users with unique emails
    async fn create_test_users(db: &DatabaseConnection, test_name: &str) -> (UserModel, UserModel) {
        let admin_user = UserModel::create(
            db,
            "u12345678",
            &format!("admin_{}@example.com", test_name),
            "password1",
            true,
        ).await.unwrap();

        let regular_user = UserModel::create(
            db,
            "u87654321",
            &format!("user_{}@example.com", test_name),
            "password2",
            false,
        ).await.unwrap();

        (admin_user, regular_user)
    }

    #[tokio::test]
    async fn test_register_success() {
        use db::models::user::Model as UserModel;

        let db = setup_test_db().await;

        let register_req = RegisterRequest {
            student_number: "u99999999".to_string(),
            email: "new@example.com".to_string(),
            password: "strongpassword".to_string(),
        };

        // Ensure validation passes
        assert!(register_req.validate().is_ok());

        // Perform registration using SeaORM model directly
        let result = UserModel::create(
            &db,
            &register_req.student_number,
            &register_req.email,
            &register_req.password,
            false,
        )
        .await;

        assert!(result.is_ok());

        let user = result.unwrap();
        assert_eq!(user.student_number, "u99999999");
        assert_eq!(user.email, "new@example.com");
        assert!(!user.admin);
    }

    #[tokio::test]
    async fn test_register_validation() {
        // Test invalid student number format
        let invalid_sn = RegisterRequest {
            student_number: "invalid".to_string(),
            email: "valid@example.com".to_string(),
            password: "strongpassword".to_string(),
        };
        assert!(invalid_sn.validate().is_err());

        // Test invalid email format
        let invalid_email = RegisterRequest {
            student_number: "u12345678".to_string(),
            email: "not-an-email".to_string(),
            password: "strongpassword".to_string(),
        };
        assert!(invalid_email.validate().is_err());

        // Test short password
        let short_password = RegisterRequest {
            student_number: "u12345678".to_string(),
            email: "valid@example.com".to_string(),
            password: "short".to_string(),
        };
        assert!(short_password.validate().is_err());

        // Test valid registration
        let valid_register = RegisterRequest {
            student_number: "u12345678".to_string(),
            email: "valid@example.com".to_string(),
            password: "strongpassword".to_string(),
        };
        assert!(valid_register.validate().is_ok());
    }

    #[tokio::test]
    async fn test_register_duplicate_email() {
        use db::models::user::Model as UserModel;

        let db = setup_test_db().await;

        // Create initial user
        let existing_user = UserModel::create(
            &db,
            "u11111111",
            "duplicate@example.com",
            "password123",
            false,
        )
        .await
        .unwrap();

        // Attempt to register with same email but different student number
        let register_req = RegisterRequest {
            student_number: "u99999999".to_string(),
            email: existing_user.email.clone(),
            password: "strongpassword".to_string(),
        };

        assert!(register_req.validate().is_ok());

        let result = UserModel::create(
            &db,
            &register_req.student_number,
            &register_req.email,
            &register_req.password,
            false,
        )
        .await;

        assert!(result.is_err());

        if let Err(e) = result {
            assert!(
                e.to_string().to_lowercase().contains("unique"),
                "Expected UNIQUE constraint error but got: {}",
                e
            );
        }
    }

    #[tokio::test]
    async fn test_register_duplicate_student_number() {
        use db::models::user::Model as UserModel;

        let db = setup_test_db().await;

        // Insert user with original student number
        let existing_user = UserModel::create(
            &db,
            "u12345678",
            "original@example.com",
            "password123",
            false,
        )
        .await
        .unwrap();

        // Attempt to register another user with same student number
        let register_req = RegisterRequest {
            student_number: existing_user.student_number.clone(),
            email: "new@example.com".to_string(),
            password: "strongpassword".to_string(),
        };

        assert!(register_req.validate().is_ok());

        let result = UserModel::create(
            &db,
            &register_req.student_number,
            &register_req.email,
            &register_req.password,
            false,
        )
        .await;

        assert!(result.is_err());

        if let Err(e) = result {
            assert!(
                e.to_string().to_lowercase().contains("unique"),
                "Expected UNIQUE constraint error but got: {}",
                e
            );
        }
    }

    #[tokio::test]
    async fn test_login_success() {
        use db::models::user::Model as UserModel;

        let db = setup_test_db().await;

        let user = UserModel::create(
            &db,
            "u87654321",
            "login@example.com",
            "password1",
            false,
        )
        .await
        .unwrap();

        let login_req = LoginRequest {
            student_number: user.student_number.clone(),
            password: "password1".to_string(),
        };

        let result = UserModel::verify_credentials(&db, &login_req.student_number, &login_req.password).await;

        assert!(result.is_ok());

        let logged_in_user = result.unwrap().unwrap(); // unwrap Result, then Option

        assert_eq!(logged_in_user.id, user.id);
        assert_eq!(logged_in_user.student_number, user.student_number);
        assert_eq!(logged_in_user.email, user.email);
        assert_eq!(logged_in_user.admin, user.admin);
    }

    #[tokio::test]
    async fn test_login_invalid_password() {
        use db::models::user::Model as UserModel;

        let db = setup_test_db().await;

        let user = UserModel::create(
            &db,
            "u12345678",
            "wrongpass@example.com",
            "correct_password",
            false,
        )
        .await
        .unwrap();

        let login_req = LoginRequest {
            student_number: user.student_number.clone(),
            password: "wrong_password".to_string(),
        };

        let result = UserModel::verify_credentials(&db, &login_req.student_number, &login_req.password).await;

        assert!(result.is_ok());
        assert!(
            result.unwrap().is_none(),
            "Expected login to fail due to incorrect password"
        );
    }


    #[tokio::test]
    async fn test_login_nonexistent_user() {
        use db::models::user::Model as UserModel;

        let db = setup_test_db().await;

        let login_req = LoginRequest {
            student_number: "u99999999".to_string(), // Not inserted
            password: "any_password".to_string(),
        };

        let result = UserModel::verify_credentials(&db, &login_req.student_number, &login_req.password).await;

        assert!(result.is_ok());
        assert!(
            result.unwrap().is_none(),
            "Expected login to fail for nonexistent user"
        );
    }

    #[tokio::test]
    async fn test_jwt_generation() {
        use std::env;
        use dotenvy::dotenv;
        use jsonwebtoken::{decode, DecodingKey, Validation};

        dotenv().ok(); // Load environment variables from .env

        let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");

        let db = setup_test_db().await;
        let (user, _) = create_test_users(&db, "jwt_generation").await;

        // Generate JWT
        let (token, expiry) = generate_jwt(user.id, user.admin);

        assert!(!token.is_empty());
        assert!(!expiry.is_empty());

        // Verify token
        let decoding_key = DecodingKey::from_secret(jwt_secret.as_bytes());
        let validation = Validation::default();
        let decoded = decode::<Claims>(&token, &decoding_key, &validation);

        assert!(decoded.is_ok());
        let claims = decoded.unwrap().claims;
        assert_eq!(claims.sub, user.id);
        assert_eq!(claims.admin, user.admin);
    }


    #[tokio::test]
    async fn test_jwt_expiration() {
        use std::env;
        use dotenvy::dotenv;
        use jsonwebtoken::{decode, DecodingKey, Validation};

        dotenv().ok();
        env::set_var("JWT_DURATION_MINUTES", "1");

        let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");

        let db = setup_test_db().await;
        let (user, _) = create_test_users(&db, "jwt_expiration").await;

        let (token, expiry) = generate_jwt(user.id, user.admin);

        let decoding_key = DecodingKey::from_secret(jwt_secret.as_bytes());
        let mut validation = Validation::default();
        validation.validate_exp = true;

        let decoded = decode::<Claims>(&token, &decoding_key, &validation);
        assert!(decoded.is_ok());

        let expiry_time = chrono::DateTime::parse_from_rfc3339(&expiry).unwrap();
        assert!(expiry_time > chrono::Utc::now());
    }


    #[tokio::test]
    async fn test_jwt_signature_validation() {
        use dotenvy::dotenv;
        use jsonwebtoken::{decode, DecodingKey, Validation};

        dotenv().ok();

        let db = setup_test_db().await;
        let (user, _) = create_test_users(&db, "jwt_signature").await;

        let (token, _) = generate_jwt(user.id, user.admin);

        let wrong_key = DecodingKey::from_secret(b"wrong_secret_key");
        let validation = Validation::default();
        let decoded = decode::<Claims>(&token, &wrong_key, &validation);

        assert!(decoded.is_err());
        if let Err(e) = decoded {
            let error_msg = e.to_string().to_lowercase();
            assert!(
                error_msg.contains("signature")
                    || error_msg.contains("invalid")
                    || error_msg.contains("verification"),
                "Expected signature verification failure, got: {error_msg}"
            );
        }
    }

    #[tokio::test]
    async fn test_jwt_format() {
        use dotenvy::dotenv;

        dotenv().ok();

        let db = setup_test_db().await;
        let (user, _) = create_test_users(&db, "jwt_format").await;

        let (token, _) = generate_jwt(user.id, user.admin);

        let parts: Vec<&str> = token.split('.').collect();
        assert_eq!(parts.len(), 3, "JWT should have three parts");
    }

    #[tokio::test]
    async fn test_jwt_invalid_tokens() {
        use std::env;
        use dotenvy::dotenv;
        use jsonwebtoken::{decode, DecodingKey, Validation};
        use crate::auth::claims::Claims;

        dotenv().ok();

        let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        let decoding_key = DecodingKey::from_secret(jwt_secret.as_bytes());
        let validation = Validation::default();

        let invalid_tokens = vec![
            "invalid.token.format",
            "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ",
            "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c",
            "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyLCJleHAiOjF9.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c",
        ];

        for token in invalid_tokens {
            let decoded = decode::<Claims>(token, &decoding_key, &validation);
            assert!(decoded.is_err(), "Token should be invalid: {}", token);
        }
    }

    #[tokio::test]
    async fn test_user_response_format() {
        use dotenvy::dotenv;
        use std::env;
        use crate::auth::{generate_jwt};

        dotenv().ok();

        let db = setup_test_db().await;
        let (user, _) = create_test_users(&db, "response_format").await;

        let _jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        let (token, expiry) = generate_jwt(user.id, user.admin);

        let response = UserResponse {
            id: user.id,
            student_number: user.student_number.clone(),
            email: user.email.clone(),
            admin: user.admin,
            token: token.clone(),
            expires_at: expiry.clone(),
        };

        assert_eq!(response.id, user.id);
        assert_eq!(response.student_number, user.student_number);
        assert_eq!(response.email, user.email);
        assert_eq!(response.admin, user.admin);
        assert_eq!(response.token, token);
        assert_eq!(response.expires_at, expiry);
    }
}