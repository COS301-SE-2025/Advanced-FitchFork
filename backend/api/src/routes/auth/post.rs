use axum::{
    extract::Multipart,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use util::{config, paths::{ensure_parent_dir, user_profile_path}, state::AppState};
use validator::Validate;
use chrono::{Utc, Duration};
use tokio::io::AsyncWriteExt;
use crate::{
    auth::generate_jwt,
    response::ApiResponse,
    services::email::EmailService,
};
use crate::auth::AuthUser;
use util::filters::FilterParam;
use services::service::Service;
use services::user::{UserService, CreateUser, UpdateUser};
use services::password_reset_token::{PasswordResetTokenService, CreatePasswordResetToken, UpdatePasswordResetToken};

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
    if let Err(validation_errors) = req.validate() {
        let error_message = common::format_validation_errors(&validation_errors);
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<UserResponse>::error(error_message)),
        );
    }

    match UserService::find_one(
        &vec![FilterParam::eq("email", req.email.clone())],
        &vec![],
        None,
    ).await {
        Ok(Some(_)) => {
            return (
                StatusCode::CONFLICT,
                Json(ApiResponse::<UserResponse>::error("A user with this email already exists")),
            );
        }
        Ok(None) => false,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<UserResponse>::error(format!("Database error: {}", e))),
            );
        }
    };

    match UserService::find_one(
        &vec![FilterParam::eq("username", req.username.clone())],
        &vec![],
        None,
    ).await {
        Ok(Some(_)) => {
            return (
                StatusCode::CONFLICT,
                Json(ApiResponse::<UserResponse>::error("A user with this student number already exists")),
            );
        }
        Ok(None) => {}
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<UserResponse>::error(format!("Database error: {}", e))),
            );
        }
    }

    let inserted_user = match UserService::create(CreateUser {
        id: None,
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
    if let Err(validation_errors) = req.validate() {
        let error_message = common::format_validation_errors(&validation_errors);
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<UserResponse>::error(error_message)),
        );
    }

    let user = match UserService::verify_credentials(&req.username, &req.password).await {
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
    if let Err(validation_errors) = req.validate() {
        let error_message = common::format_validation_errors(&validation_errors);
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(error_message)),
        );
    }

    let user = match UserService::find_one(
        &vec![FilterParam::eq("email", req.email.clone())],
        &vec![],
        None,
    ).await {
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
    let recent_requests = PasswordResetTokenService::count(
        &vec![
            FilterParam::eq("user_id", user.id),
            FilterParam::gt("created_at", one_hour_ago),
        ],
        &vec![],
    ).await.unwrap_or(0);

    let max_requests = config::max_password_reset_requests_per_hour() as u64;

    if recent_requests >= max_requests {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(ApiResponse::<()>::error(
                "Too many password reset requests. Please try again later.",
            )),
        );
    }

    let expiry_minutes = config::reset_token_expiry_minutes() as i64;

    match PasswordResetTokenService::create(
        CreatePasswordResetToken {
            user_id: user.id,
            expiry_minutes,
        }
    ).await {
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
    if let Err(validation_errors) = req.validate() {
        let error_message = common::format_validation_errors(&validation_errors);
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<VerifyResetTokenResponse>::error(error_message)),
        );
    }

    match PasswordResetTokenService::find_valid_token(req.token).await {
        Ok(Some(token)) => {
            match UserService::find_by_id(token.user_id).await {
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
    if let Err(validation_errors) = req.validate() {
        let error_message = common::format_validation_errors(&validation_errors);
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(error_message)),
        );
    }

    match PasswordResetTokenService::find_valid_token(req.token).await {
        Ok(Some(token)) => {
            match UserService::find_by_id(token.user_id).await {
                Ok(Some(user)) => {
                    let user_email = user.email.clone();
                    
                    match UserService::update(
                        UpdateUser {
                            id: token.user_id,
                            username: None,
                            email: None,
                            password: Some(req.new_password),
                            admin: None,
                            profile_picture_path: None,
                        }
                    ).await {
                        Ok(_) => {
                            if let Err(e) = PasswordResetTokenService::update(
                                UpdatePasswordResetToken {
                                    id: token.id,
                                    used: Some(true),
                                }
                            ).await {
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
                        Json(ApiResponse::<()>::error("File type not supported.")),
                    );
                }
            }

            let bytes = field.bytes().await.unwrap();
            if bytes.len() as u64 > MAX_SIZE {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::<()>::error("File too large.")),
                );
            }

            file_data = Some(bytes);
        }
    }

    let Some(file_bytes) = file_data else {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error("No file uploaded.")),
        );
    };

    let ext = match content_type.as_deref() {
        Some("image/png") => "png",
        Some("image/jpeg") => "jpg",
        Some("image/gif") => "gif",
        _ => "bin",
    };

    let filename = format!("avatar.{}", ext);
    let abs_path = user_profile_path(claims.sub, &filename);

    // Ensure parent directory exists
    if let Err(e) = ensure_parent_dir(&abs_path) {
        eprintln!("Failed to ensure profile dir: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error("Failed to prepare storage")),
        );
    }

    let mut file = tokio::fs::File::create(&abs_path).await.unwrap();
    file.write_all(&file_bytes).await.unwrap();

    // Store only the file name; path is derived with user_profile_path(user_id, filename)
    let stored_filename = filename.clone();

    let current = user::Entity::find_by_id(claims.sub).one(db).await.unwrap().unwrap();
    let mut model = current.into_active_model();
    model.profile_picture_path = Set(Some(stored_filename.clone()));
    model.update(db).await.unwrap();

    (
        StatusCode::OK,
        Json(ApiResponse::success_without_data("Profile picture uploaded.")),
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
    if let Err(validation_errors) = req.validate() {
        let error_message = common::format_validation_errors(&validation_errors);
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(error_message)),
        );
    }

    let user = match UserService::find_by_id(claims.sub).await {
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
    
    match UserService::update(
        UpdateUser {
            id: claims.sub,
            username: None,
            email: None,
            password: Some(req.new_password),
            admin: None,
            profile_picture_path: None,
        }
    ).await {
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