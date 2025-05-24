use axum::{
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use db::models::user::User;
use db::pool;
use crate::auth::generate_jwt;
use crate::response::ApiResponse;
use common::format_validation_errors;
use serde::{Deserialize, Serialize};
use validator::Validate;

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
        let error_message = format_validation_errors(&validation_errors);
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<UserResponse>::error(error_message)),
        );
    }

    let pool = pool::get();

    if let Ok(Some(_)) = sqlx::query_as::<_, User>(
        "SELECT id FROM users WHERE email = ?",
    )
    .bind(&req.email)
    .fetch_optional(pool)
    .await
    {
        return (
            StatusCode::CONFLICT,
            Json(ApiResponse::<UserResponse>::error("A user with this email already exists")),
        );
    }

    if let Ok(Some(_)) = User::get_by_student_number(Some(pool), &req.student_number).await {
        return (
            StatusCode::CONFLICT,
            Json(ApiResponse::<UserResponse>::error("A user with this student number already exists")),
        );
    }

    match User::create(Some(pool), &req.student_number, &req.email, &req.password, false).await {
        Ok(user) => {
            let (token, expiry) = generate_jwt(user.id, user.admin);
            let user_response = UserResponse {
                id: user.id,
                student_number: user.student_number,
                email: user.email,
                admin: user.admin,
                token,
                expires_at: expiry,
            };
            return (
                StatusCode::CREATED,
                Json(ApiResponse::success(user_response, "User registered successfully")),
            );
        }

        Err(e) => {
            if let Some(db_err) = e.as_database_error() {
                let msg = db_err.message();
                if msg.contains("users.email") {
                    return (
                        StatusCode::CONFLICT,
                        Json(ApiResponse::<UserResponse>::error("A user with this email already exists")),
                    );
                }
                if msg.contains("users.student_number") {
                    return (
                        StatusCode::CONFLICT,
                        Json(ApiResponse::<UserResponse>::error("A user with this student number already exists")),
                    );
                }
            }

            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<UserResponse>::error(format!("Database error: {}", e))),
            );
        }
    }
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
        let error_message = format_validation_errors(&validation_errors);
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<UserResponse>::error(error_message)),
        );
    }

    match User::verify(None, &req.student_number, &req.password).await {
        Ok(user) => {
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

        Err(e) => {
            let (status, message) = match &e {
                sqlx::Error::RowNotFound => (
                    StatusCode::UNAUTHORIZED,
                    "No account with that student number".to_string(),
                ),

                sqlx::Error::Protocol(msg) if msg == "Invalid credentials" => (
                    StatusCode::UNAUTHORIZED,
                    "Invalid password".to_string(),
                ),

                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Database error: {}", e),
                ),
            };

            (status, Json(ApiResponse::<UserResponse>::error(message)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use db::models::user::User;
    use sqlx::SqlitePool;
    use db::{create_test_db, delete_database};
    use jsonwebtoken::{decode, DecodingKey, Validation};
    use crate::auth::claims::Claims;
    use common::config::Config;

    // Helper function to create test users with unique emails
    async fn create_test_users(pool: &SqlitePool, test_name: &str) -> (User, User) {
        let admin_user = User::create(
            Some(pool),
            "u12345678",
            &format!("admin_{}@example.com", test_name),
            "password1",
            true
        )
        .await
        .unwrap();

        let regular_user = User::create(
            Some(pool),
            "u87654321",
            &format!("user_{}@example.com", test_name),
            "password2",
            false
        )
        .await
        .unwrap();

        (admin_user, regular_user)
    }

    // Helper function to initialize test configuration
    fn init_test_config() {
        // Set required environment variables for Config::init
        std::env::set_var("DATABASE_URL", "sqlite::memory:");
        std::env::set_var("JWT_SECRET", "test_secret_key_for_jwt_generation_and_validation");
        std::env::set_var("JWT_DURATION_MINUTES", "1440"); // 24 hours in minutes
        
        // Initialize config with test values
        Config::init(".env.test");
    }

    #[tokio::test]
    async fn test_register_success() {
        let pool = create_test_db(Some("test_register_success.db")).await;
        
        let register_req = RegisterRequest {
            student_number: "u99999999".to_string(),
            email: "new@example.com".to_string(),
            password: "strongpassword".to_string(),
        };

        // Verify validation passes
        assert!(register_req.validate().is_ok());

        // Test the registration logic
        let result = User::create(
            Some(&pool),
            &register_req.student_number,
            &register_req.email,
            &register_req.password,
            false
        ).await;

        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.student_number, "u99999999");
        assert_eq!(user.email, "new@example.com");
        assert!(!user.admin);

        pool.close().await;
        delete_database("test_register_success.db");
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
        let pool = create_test_db(Some("test_register_duplicate_email.db")).await;
        
        let (existing_user, _) = create_test_users(&pool, "duplicate_email").await;

        // Try to register with existing email
        let register_req = RegisterRequest {
            student_number: "u99999999".to_string(),
            email: existing_user.email.clone(),
            password: "strongpassword".to_string(),
        };

        // Verify validation passes
        assert!(register_req.validate().is_ok());

        // Test the registration logic
        let result = User::create(
            Some(&pool),
            &register_req.student_number,
            &register_req.email,
            &register_req.password,
            false
        ).await;

        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("UNIQUE constraint failed"));
        }

        pool.close().await;
        delete_database("test_register_duplicate_email.db");
    }

    #[tokio::test]
    async fn test_register_duplicate_student_number() {
        let pool = create_test_db(Some("test_register_duplicate_sn.db")).await;
        
        let (existing_user, _) = create_test_users(&pool, "duplicate_sn").await;

        // Try to register with existing student number
        let register_req = RegisterRequest {
            student_number: existing_user.student_number.clone(),
            email: "new@example.com".to_string(),
            password: "strongpassword".to_string(),
        };

        // Verify validation passes
        assert!(register_req.validate().is_ok());

        // Test the registration logic
        let result = User::create(
            Some(&pool),
            &register_req.student_number,
            &register_req.email,
            &register_req.password,
            false
        ).await;

        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("UNIQUE constraint failed"));
        }

        pool.close().await;
        delete_database("test_register_duplicate_sn.db");
    }

    #[tokio::test]
    async fn test_login_success() {
        let pool = create_test_db(Some("test_login_success.db")).await;
        
        let (user, _) = create_test_users(&pool, "login_success").await;

        let login_req = LoginRequest {
            student_number: user.student_number.clone(),
            password: "password1".to_string(),
        };

        // Test the login logic
        let result = User::verify(Some(&pool), &login_req.student_number, &login_req.password).await;

        assert!(result.is_ok());
        let logged_in_user = result.unwrap();
        assert_eq!(logged_in_user.id, user.id);
        assert_eq!(logged_in_user.student_number, user.student_number);
        assert_eq!(logged_in_user.email, user.email);
        assert_eq!(logged_in_user.admin, user.admin);

        pool.close().await;
        delete_database("test_login_success.db");
    }

    #[tokio::test]
    async fn test_login_invalid_password() {
        let pool = create_test_db(Some("test_login_invalid_password.db")).await;
        
        let (user, _) = create_test_users(&pool, "invalid_password").await;

        let login_req = LoginRequest {
            student_number: user.student_number.clone(),
            password: "wrongpassword".to_string(),
        };

        // Test the login logic
        let result = User::verify(Some(&pool), &login_req.student_number, &login_req.password).await;

        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("Invalid credentials"));
        }

        pool.close().await;
        delete_database("test_login_invalid_password.db");
    }

    #[tokio::test]
    async fn test_login_nonexistent_user() {
        let pool = create_test_db(Some("test_login_nonexistent.db")).await;
        
        let login_req = LoginRequest {
            student_number: "u99999999".to_string(),
            password: "password".to_string(),
        };

        // Test the login logic
        let result = User::verify(Some(&pool), &login_req.student_number, &login_req.password).await;

        assert!(result.is_err());
        if let Err(e) = result {
            assert!(matches!(e, sqlx::Error::RowNotFound));
        }

        pool.close().await;
        delete_database("test_login_nonexistent.db");
    }

    #[tokio::test]
    async fn test_jwt_generation() {
        // Initialize test configuration
        init_test_config();

        let pool = create_test_db(Some("test_jwt_generation.db")).await;
        
        let (user, _) = create_test_users(&pool, "jwt_generation").await;

        // Test JWT generation
        let (token, expiry) = generate_jwt(user.id, user.admin);

        // Verify token is not empty
        assert!(!token.is_empty());
        assert!(!expiry.is_empty());

        // Verify token can be decoded
        let decoding_key = DecodingKey::from_secret(Config::get().jwt_secret.as_bytes());
        let validation = Validation::default();
        let decoded = decode::<Claims>(&token, &decoding_key, &validation);

        assert!(decoded.is_ok());
        let claims = decoded.unwrap().claims;
        assert_eq!(claims.sub, user.id);
        assert_eq!(claims.admin, user.admin);

        pool.close().await;
        delete_database("test_jwt_generation.db");
    }

    #[tokio::test]
    async fn test_jwt_expiration() {
        // Initialize test configuration with short expiry
        std::env::set_var("JWT_DURATION_MINUTES", "1");
        init_test_config();

        let pool = create_test_db(Some("test_jwt_expiration.db")).await;
        
        let (user, _) = create_test_users(&pool, "jwt_expiration").await;

        // Generate token
        let (token, expiry) = generate_jwt(user.id, user.admin);

        // Verify token can be decoded
        let decoding_key = DecodingKey::from_secret(Config::get().jwt_secret.as_bytes());
        let mut validation = Validation::default();
        validation.validate_exp = true;
        
        let decoded = decode::<Claims>(&token, &decoding_key, &validation);
        assert!(decoded.is_ok());

        // Verify expiry time is in the future
        let expiry_time = chrono::DateTime::parse_from_rfc3339(&expiry).unwrap();
        assert!(expiry_time > chrono::Utc::now());

        pool.close().await;
        delete_database("test_jwt_expiration.db");
    }

    #[tokio::test]
    async fn test_jwt_signature_validation() {
        // Initialize test configuration
        init_test_config();

        let pool = create_test_db(Some("test_jwt_signature.db")).await;
        
        let (user, _) = create_test_users(&pool, "jwt_signature").await;

        // Generate token
        let (token, _) = generate_jwt(user.id, user.admin);

        // Try to decode with wrong secret
        let wrong_key = DecodingKey::from_secret(b"wrong_secret_key");
        let validation = Validation::default();
        let decoded = decode::<Claims>(&token, &wrong_key, &validation);

        assert!(decoded.is_err());
        if let Err(e) = decoded {
            let error_msg = e.to_string().to_lowercase();
            assert!(
                error_msg.contains("signature") || 
                error_msg.contains("invalid") || 
                error_msg.contains("verification"),
                "Error message should indicate signature verification failure"
            );
        }

        pool.close().await;
        delete_database("test_jwt_signature.db");
    }

    #[tokio::test]
    async fn test_jwt_format() {
        // Initialize test configuration
        init_test_config();

        let pool = create_test_db(Some("test_jwt_format.db")).await;
        
        let (user, _) = create_test_users(&pool, "jwt_format").await;

        // Generate token
        let (token, _) = generate_jwt(user.id, user.admin);

        // Verify token format (should be three base64-encoded parts separated by dots)
        let parts: Vec<&str> = token.split('.').collect();
        assert_eq!(parts.len(), 3, "JWT should have three parts");

        pool.close().await;
        delete_database("test_jwt_format.db");
    }

    #[tokio::test]
    async fn test_jwt_invalid_tokens() {
        // Initialize test configuration
        init_test_config();

        let invalid_tokens = vec![
            "invalid.token.format",  // Wrong format
            "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ",  // Missing signature
            "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c",  // Wrong signature
            "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyLCJleHAiOjF9.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c",  // Expired token
        ];

        let decoding_key = DecodingKey::from_secret(Config::get().jwt_secret.as_bytes());
        let validation = Validation::default();

        for token in invalid_tokens {
            let decoded = decode::<Claims>(token, &decoding_key, &validation);
            assert!(decoded.is_err(), "Invalid token should not decode successfully");
        }
    }

    #[tokio::test]
    async fn test_user_response_format() {
        // Initialize test configuration
        init_test_config();

        let pool = create_test_db(Some("test_user_response.db")).await;
        
        let (user, _) = create_test_users(&pool, "response_format").await;

        // Generate JWT
        let (token, expiry) = generate_jwt(user.id, user.admin);

        // Create response
        let response = UserResponse {
            id: user.id,
            student_number: user.student_number.clone(),
            email: user.email.clone(),
            admin: user.admin,
            token: token.clone(),
            expires_at: expiry.clone(),
        };

        // Verify response format
        assert_eq!(response.id, user.id);
        assert_eq!(response.student_number, user.student_number);
        assert_eq!(response.email, user.email);
        assert_eq!(response.admin, user.admin);
        assert_eq!(response.token, token);
        assert_eq!(response.expires_at, expiry);

        pool.close().await;
        delete_database("test_user_response.db");
    }
}