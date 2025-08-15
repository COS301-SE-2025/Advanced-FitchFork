use std::path::PathBuf;
use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum::extract::Multipart;
use sea_orm::{ActiveModelTrait, ColumnTrait, Condition, EntityTrait, IntoActiveModel, QueryFilter, Set};
use serde::Deserialize;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use validator::Validate;
use crate::{response::ApiResponse};
use common::format_validation_errors;
use db::models::user;
use crate::routes::common::UserResponse;

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUserRequest {
    pub username: Option<String>,

    #[validate(email(message = "Invalid email format"))]
    pub email: Option<String>,

    pub admin: Option<bool>,
}

lazy_static::lazy_static! {
    static ref username_REGEX: regex::Regex = regex::Regex::new("^u\\d{8}$").unwrap();
}

/// PUT /api/users/{user_id}
///
/// Update a user's information. Only admins can access this endpoint.
///
/// # Path Parameters
/// * `id` - The ID of the user to update
///
/// # Request Body
/// ```json
/// {
///   "username": "u87654321",  // optional
///   "email": "new@example.com",     // optional
///   "admin": true                   // optional
/// }
/// ```
///
/// # Responses
///
/// - `200 OK`
/// ```json
/// {
///   "success": true,
///   "data": {
///     "id": 1,
///     "username": "u87654321",
///     "email": "new@example.com",
///     "admin": true,
///     "created_at": "2025-05-23T18:00:00Z",
///     "updated_at": "2025-05-23T18:00:00Z"
///   },
///   "message": "User updated successfully"
/// }
/// ```
///
/// - `400 Bad Request` (validation error)
/// ```json
/// {
///   "success": false,
///   "message": "Student number must be in format u12345678"
/// }
/// ```
///
/// - `404 Not Found` (user doesn't exist)
/// ```json
/// {
///   "success": false,
///   "message": "User not found"
/// }
/// ```
///
/// - `409 Conflict` (duplicate email/student number)
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
pub async fn update_user(
    Path(user_id): Path<i64>,
    Json(req): Json<UpdateUserRequest>,
) -> impl IntoResponse {
    let db = db::get_connection().await;

    if let Err(e) = req.validate() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<UserResponse>::error(format_validation_errors(&e))),
        );
    }

    if req.username.is_none() && req.email.is_none() && req.admin.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<UserResponse>::error("At least one field must be provided")),
        );
    }

    let current_user = user::Entity::find_by_id(user_id)
        .one(db).await.unwrap().unwrap();

    // TODO: Should probably make a more robust system with a super admin
    // Prevent changing your own admin status or changing others' admin status
    if let Some(_) = req.admin {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<UserResponse>::error("Changing admin status is not allowed")),
        );
    }

    if let Some(email) = &req.email {
        if email != &current_user.email {
            let exists_result = user::Entity::find()
                .filter(
                    Condition::all()
                        .add(user::Column::Email.eq(email.clone()))
                        .add(user::Column::Id.ne(user_id)),
                )
                .one(db)
                .await;

            match exists_result {
                Ok(Some(_)) => {
                    return (
                        StatusCode::CONFLICT,
                        Json(ApiResponse::<UserResponse>::error("A user with this email already exists")),
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
        }
    }

    if let Some(sn) = &req.username {
        if sn != &current_user.username {
            let exists_result = user::Entity::find()
                .filter(
                    Condition::all()
                        .add(user::Column::Username.eq(sn.clone()))
                        .add(user::Column::Id.ne(user_id)),
                )
                .one(db)
                .await;

            match exists_result {
                Ok(Some(_)) => {
                    return (
                        StatusCode::CONFLICT,
                        Json(ApiResponse::<UserResponse>::error(
                            "A user with this student number already exists",
                        )),
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
        }
    }

    let mut active_model: user::ActiveModel = current_user.into();
    if let Some(sn) = req.username {
        active_model.username = Set(sn);
    }
    if let Some(email) = req.email {
        active_model.email = Set(email);
    }
    if let Some(admin) = req.admin {
        active_model.admin = Set(admin);
    }

    match active_model.update(db).await {
        Ok(updated) => (
            StatusCode::OK,
            Json(ApiResponse::success(
                UserResponse::from(updated),
                "User updated successfully",
            )),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<UserResponse>::error(format!("Database error: {}", e))),
        ),
    }
}

#[derive(serde::Serialize)]
struct ProfilePictureResponse {
    profile_picture_path: String,
}

/// PUT /api/users/{user_id}/avatar
///
/// Upload a avatar for a user. Only admins may upload avatars for other users.
///
/// # Path Parameters
/// - `id` - The ID of the user to upload the avatar for
///
/// # Request (multipart/form-data)
/// - `file` (required): The image file to upload.  
///   Allowed types: `image/jpeg`, `image/png`, `image/gif`  
///   Max size: 2MB
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
///     "message": "Avatar uploaded for user."
///   }
///   ```
///
/// - `400 Bad Request`  
///   - No file uploaded
///   - File too large
///   - File type not supported
///
/// - `403 Forbidden`  
///   ```json
///   {
///     "success": false,
///     "message": "Only admins may upload avatars for other users"
///   }
///   ```
///
/// - `404 Not Found`  
///   ```json
///   {
///     "success": false,
///     "message": "User not found."
///   }
///   ```
///
/// - `500 Internal Server Error`  
///   ```json
///   {
///     "success": false,
///     "message": "Database error."
///   }
///   ```
///
pub async fn upload_avatar(
    Path(user_id): Path<i64>,
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
                    )
                }
            }

            let bytes = field.bytes().await.unwrap();
            if bytes.len() as u64 > MAX_SIZE {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::<ProfilePictureResponse>::error("File too large.")),
                )
            }

            file_data = Some(bytes);
        }
    }

    let Some(file_bytes) = file_data else {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<ProfilePictureResponse>::error("No file uploaded.")),
        )
    };

    let ext = match content_type.as_deref() {
        Some("image/png") => "png",
        Some("image/jpeg") => "jpg",
        Some("image/gif") => "gif",
        _ => "bin",
    };

    let root = std::env::var("USER_PROFILE_STORAGE_ROOT")
        .unwrap_or_else(|_| "data/user_profile_pictures".to_string());

    let user_dir = PathBuf::from(&root).join(format!("user_{}", user_id));
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

    let current = user::Entity::find_by_id(user_id)
        .one(db).await.unwrap().unwrap();

    let mut model = current.into_active_model();
    model.profile_picture_path = Set(Some(relative_path.clone()));
    model.update(db).await.unwrap();

    let response = ProfilePictureResponse {
        profile_picture_path: relative_path,
    };

    (
        StatusCode::OK,
        Json(ApiResponse::success(response, "Avatar uploaded for user.")),
    )
}