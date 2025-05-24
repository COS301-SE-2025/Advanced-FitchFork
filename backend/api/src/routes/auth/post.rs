use axum::{
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use db::models::user::User;
use db::pool;
use crate::auth::generate_jwt;
use crate::response::ApiResponse;
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
        let error_message = common::format_validation_errors(&validation_errors);
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
        let error_message = common::format_validation_errors(&validation_errors);
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