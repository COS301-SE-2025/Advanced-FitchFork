use std::fs;
use std::path::PathBuf;
use axum::{
    extract::Multipart,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sea_orm::{EntityTrait, ColumnTrait, QueryFilter, PaginatorTrait, ActiveModelTrait, ActiveValue::Set, IntoActiveModel};
use serde::{Deserialize, Serialize};
use validator::Validate;
use chrono::{Utc, Duration};
use tokio::io::AsyncWriteExt;
use crate::{
    auth::generate_jwt,
    response::ApiResponse,
    services::email::EmailService,
};
use db::models::{
    user,
    password_reset_token::{self, Model as PasswordResetTokenModel}
};
use crate::auth::AuthUser;

use db::repositories::user_repository::UserRepository;
use services::{
    service::Service,
    user_service::{UserService, CreateUser},
};

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    pub username: String,

    #[validate(email(message = "Invalid email format"))]
    pub email: String,

    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,

    // TODO: Add some more password validation later
}

#[derive(Debug, Serialize, Default)]
pub struct UserResponse {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub admin: bool,
    pub token: String,
    pub expires_at: String,
}

/// POST /api/auth/register
///
/// Register a new user.
///
/// ### Request Body
/// ```json
/// {
///   "username": "u12345678",
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
///     "username": "u12345678",
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
pub async fn register(
    Json(req): Json<RegisterRequest>
) -> impl IntoResponse {
    let db = db::get_connection().await;

    if let Err(validation_errors) = req.validate() {
        let error_message = common::format_validation_errors(&validation_errors);
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<UserResponse>::error(error_message)),
        );
    }

    let email_exists = user::Entity::find()
        .filter(user::Column::Email.eq(req.email.clone()))
        .one(db)
        .await
        .unwrap();

    if email_exists.is_some() {
        return (
            StatusCode::CONFLICT,
            Json(ApiResponse::<UserResponse>::error("A user with this email already exists")),
        );
    }

    let sn_exists = user::Entity::find()
        .filter(user::Column::Username.eq(req.username.clone()))
        .one(db)
        .await
        .unwrap();

    if sn_exists.is_some() {
        return (
            StatusCode::CONFLICT,
            Json(ApiResponse::<UserResponse>::error("A user with this student number already exists")),
        );
    }

    let service = UserService::new(UserRepository::new(db.clone()));
    let inserted_user = match service.create(CreateUser {
        username: req.username,
        email: req.email,
        password: req.password,
        admin: false,
    }).await {
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
        username: inserted_user.username,
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
    pub username: String,
    pub password: String,
}

/// POST /api/auth/login
///
/// Authenticate an existing user and issue a JWT.
///
/// ### Request Body
/// ```json
/// {
///   "username": "u12345678",
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
///     "username": "u12345678",
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
pub async fn login(
    Json(req): Json<LoginRequest>
) -> impl IntoResponse {
    let db = db::get_connection().await;

    if let Err(validation_errors) = req.validate() {
        let error_message = common::format_validation_errors(&validation_errors);
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<UserResponse>::error(error_message)),
        );
    }

    let service = UserService::new(UserRepository::new(db.clone()));
    let user = match service.verify_credentials(&req.username, &req.password).await {
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
        username: user.username,
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

/// POST /api/auth/request-password-reset
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
pub async fn request_password_reset(
    Json(req): Json<RequestPasswordResetRequest>
) -> impl IntoResponse {
    let db = db::get_connection().await;

    if let Err(validation_errors) = req.validate() {
        let error_message = common::format_validation_errors(&validation_errors);
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(error_message)),
        );
    }

    let user = match user::Entity::find()
        .filter(user::Column::Email.eq(req.email.clone()))
        .one(db)
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
        .count(db)
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

    match PasswordResetTokenModel::create(db, user.id, expiry_minutes).await {
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

/// POST /api/auth/verify-reset-token
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
pub async fn verify_reset_token(
    Json(req): Json<VerifyResetTokenRequest>
) -> impl IntoResponse {
    let db = db::get_connection().await;

    if let Err(validation_errors) = req.validate() {
        let error_message = common::format_validation_errors(&validation_errors);
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<VerifyResetTokenResponse>::error(error_message)),
        );
    }

    match PasswordResetTokenModel::find_valid_token(db, &req.token).await {
        Ok(Some(token)) => {
            match user::Entity::find_by_id(token.user_id).one(db).await {
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

/// POST /api/auth/reset-password
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
pub async fn reset_password(
    Json(req): Json<ResetPasswordRequest>
) -> impl IntoResponse {
    let db = db::get_connection().await;

    if let Err(validation_errors) = req.validate() {
        let error_message = common::format_validation_errors(&validation_errors);
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(error_message)),
        );
    }

    match PasswordResetTokenModel::find_valid_token(db, &req.token).await {
        Ok(Some(token)) => {
            match user::Entity::find_by_id(token.user_id).one(db).await {
                Ok(Some(user)) => {
                    let user_email = user.email.clone();
                    
                    let mut active_model: user::ActiveModel = user.into();
                    active_model.password_hash = Set(UserService::hash_password(&req.new_password));
                    
                    match active_model.update(db).await {
                        Ok(_) => {
                            if let Err(e) = token.mark_as_used(db).await {
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

#[derive(serde::Serialize)]
struct ProfilePictureResponse {
    profile_picture_path: String,
}

/// POST /api/auth/upload-profile-picture
///
/// Upload a profile picture for the authenticated user.
///
/// This endpoint accepts a `multipart/form-data` request containing a single file field named `file`.
/// Only JPEG, PNG, and GIF images are allowed, and the file size must not exceed 2MB.
/// The uploaded file is stored in a user-specific directory, and the user's profile in the database
/// is updated with the relative path to the profile picture.
///
/// # Request (multipart/form-data)
/// - `file`: The image file to upload (JPEG, PNG, or GIF, max 2MB)
///
/// # Responses
///
/// - `200 OK`  
///   ```json
///   {
///     "success": true,
///     "data": {
///       "profile_picture_path": "user_1/avatar.jpg"
///     },
///     "message": "Profile picture uploaded."
///   }
///   ```
///
/// - `400 Bad Request` (invalid file type, too large, or missing file)  
///   ```json
///   {
///     "success": false,
///     "message": "File type not supported."
///   }
///   ```
///   or
///   ```json
///   {
///     "success": false,
///     "message": "File too large."
///   }
///   ```
///   or
///   ```json
///   {
///     "success": false,
///     "message": "No file uploaded."
///   }
///   ```
///
/// - `500 Internal Server Error`  
///   ```json
///   {
///     "success": false,
///     "message": "Database error: detailed error here"
///   }
///   ```
pub async fn upload_profile_picture(
    AuthUser(claims): AuthUser,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let db = db::get_connection().await;

    const MAX_SIZE: u64 = 2 * 1024 * 1024;
    const ALLOWED_MIME: &[&str] = &["image/jpeg", "image/png", "image/gif"];

    let mut content_type = None;
    let mut file_data = None;

    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        if field.name() == Some("file") {
            content_type = field.content_type().map(|ct| ct.to_string());

            if let Some(ct) = &content_type {
                if !ALLOWED_MIME.contains(&ct.as_str()) {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(ApiResponse::<ProfilePictureResponse>::error("File type not supported.")),
                    );
                }
            }

            let bytes = field.bytes().await.unwrap();
            if bytes.len() as u64 > MAX_SIZE {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::<ProfilePictureResponse>::error("File too large.")),
                );
            }

            file_data = Some(bytes);
        }
    }

    let Some(file_bytes) = file_data else {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<ProfilePictureResponse>::error("No file uploaded.")),
        );
    };

    let ext = match content_type.as_deref() {
        Some("image/png") => "png",
        Some("image/jpeg") => "jpg",
        Some("image/gif") => "gif",
        _ => "bin",
    };

    let root = std::env::var("USER_PROFILE_STORAGE_ROOT")
        .unwrap_or_else(|_| "data/user_profile_pictures".to_string());

    let user_dir = PathBuf::from(&root).join(format!("user_{}", claims.sub));
    let _ = fs::create_dir_all(&user_dir);

    let filename = format!("avatar.{}", ext);
    let path = user_dir.join(&filename);
    let mut file = tokio::fs::File::create(&path).await.unwrap();
    file.write_all(&file_bytes).await.unwrap();

    let relative_path = path
        .strip_prefix(&root)
        .unwrap()
        .to_string_lossy()
        .to_string();

    let current = user::Entity::find_by_id(claims.sub).one(db).await.unwrap().unwrap();
    let mut model = current.into_active_model();
    model.profile_picture_path = Set(Some(relative_path.clone()));
    model.update(db).await.unwrap();

    let response = ProfilePictureResponse {
        profile_picture_path: relative_path,
    };

   (
        StatusCode::OK,
        Json(ApiResponse::success(response, "Profile picture uploaded.")),
    )
}

#[derive(Debug, Deserialize, Validate)]
pub struct ChangePasswordRequest {
    #[validate(length(min = 1, message = "Current password is required"))]
    pub current_password: String,

    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub new_password: String,
}

/// POST /api/auth/change_password
///
/// Change the password for an authenticated user.
///
/// ### Request Body
/// ```json
/// {
///   "current_password": "OldPassword123",
///   "new_password": "NewSecurePassword456"
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
///   "message": "Password changed successfully."
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
/// - `401 Unauthorized` (invalid current password)  
/// ```json
/// {
///   "success": false,
///   "message": "Current password is incorrect"
/// }
/// ```
///
/// - `401 Unauthorized` (not authenticated)  
/// ```json
/// {
///   "success": false,
///   "message": "Authentication required"
/// }
/// ```
pub async fn change_password(
    AuthUser(claims): AuthUser,
    Json(req): Json<ChangePasswordRequest>,
) -> impl IntoResponse {
    let db = db::get_connection().await;

    if let Err(validation_errors) = req.validate() {
        let error_message = common::format_validation_errors(&validation_errors);
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(error_message)),
        );
    }

    let user = match user::Entity::find_by_id(claims.sub).one(db).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ApiResponse::<()>::error("Authentication required")),
            );
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(format!("Database error: {}", e))),
            );
        }
    };

    if !UserService::verify_password(&user, &req.current_password) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::<()>::error("Current password is incorrect")),
        );
    }

    let mut active_user = user.into_active_model();
    active_user.password_hash = Set(UserService::hash_password(&req.new_password));
    
    match active_user.update(db).await {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse::success((), "Password changed successfully.")),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Database error: {}", e))),
        ),
    }
}