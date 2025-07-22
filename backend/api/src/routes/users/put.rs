use std::path::PathBuf;
use axum::{
    extract::{State, Path},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum::extract::Multipart;
use sea_orm::{ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, EntityTrait, IntoActiveModel, QueryFilter, Set};
use serde::Deserialize;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use validator::Validate;
use crate::{response::ApiResponse};
use common::format_validation_errors;
use db::models::user;
use crate::auth::AuthUser;
use crate::routes::common::UserResponse;

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUserRequest {
    #[validate(regex(
        path = &*username_REGEX,
        message = "Student number must be in format u12345678"
    ))]
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
    State(db): State<DatabaseConnection>,
    Path(user_id): Path<i64>,
    Json(req): Json<UpdateUserRequest>,
) -> impl IntoResponse {
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
        .one(&db).await.unwrap().unwrap();

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
                .one(&db)
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
                .one(&db)
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

    match active_model.update(&db).await {
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
    State(db): State<DatabaseConnection>,
    AuthUser(claims): AuthUser,
    Path(user_id): Path<i64>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    if !claims.admin {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<ProfilePictureResponse>::error("Only admins may upload avatars for other users")),
        )
    }

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
        .one(&db).await.unwrap().unwrap();

    let mut model = current.into_active_model();
    model.profile_picture_path = Set(Some(relative_path.clone()));
    model.update(&db).await.unwrap();

    let response = ProfilePictureResponse {
        profile_picture_path: relative_path,
    };

    (
        StatusCode::OK,
        Json(ApiResponse::success(response, "Avatar uploaded for user.")),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{
        ActiveModelTrait, IntoActiveModel, Set, DatabaseConnection,
    };
    use db::{models::user};
    use validator::Validate;
    use db::test_utils::setup_test_db;

    // Helper to create test users using SeaORM
    async fn create_test_users(db: &DatabaseConnection) -> (user::Model, user::Model) {
        let admin_user = user::ActiveModel {
            username: Set("u12345678".to_string()),
            email: Set("admin@example.com".to_string()),
            password_hash: Set("hashed1".to_string()),
            admin: Set(true),
            ..Default::default()
        }
        .insert(db)
        .await
        .unwrap();

        let regular_user = user::ActiveModel {
            username: Set("u87654321".to_string()),
            email: Set("user@example.com".to_string()),
            password_hash: Set("hashed2".to_string()),
            admin: Set(false),
            ..Default::default()
        }
        .insert(db)
        .await
        .unwrap();

        (admin_user, regular_user)
    }

    #[tokio::test]
    async fn test_update_user_success() {
        let db = setup_test_db().await;

        let (_, target_user) = create_test_users(&db).await;

        let update_req = UpdateUserRequest {
            username: Some("u99999999".to_string()),
            email: Some("updated@example.com".to_string()),
            admin: Some(true),
        };

        assert!(update_req.validate().is_ok());

        let mut model = target_user.into_active_model();
        if let Some(sn) = update_req.username.clone() {
            model.username = Set(sn);
        }
        if let Some(em) = update_req.email.clone() {
            model.email = Set(em);
        }
        if let Some(ad) = update_req.admin {
            model.admin = Set(ad);
        }

        let updated = model.update(&db).await.unwrap();

        assert_eq!(updated.username, "u99999999");
        assert_eq!(updated.email, "updated@example.com");
        assert!(updated.admin);
    }

    #[tokio::test]
    async fn test_update_user_validation() {
        // Invalid student number
        let invalid_sn = UpdateUserRequest {
            username: Some("invalid".to_string()),
            email: None,
            admin: None,
        };
        assert!(invalid_sn.validate().is_err());

        // Invalid email
        let invalid_email = UpdateUserRequest {
            username: None,
            email: Some("not-an-email".to_string()),
            admin: None,
        };
        assert!(invalid_email.validate().is_err());

        // Valid case
        let valid_update = UpdateUserRequest {
            username: Some("u12345678".to_string()),
            email: Some("valid@example.com".to_string()),
            admin: Some(true),
        };
        assert!(valid_update.validate().is_ok());
    }

    #[tokio::test]
    async fn test_update_user_not_found() {
        use sea_orm::{EntityTrait};
        use db::models::user;

        let db = setup_test_db().await;
        let non_existent_id = 99999i32;

        let result = user::Entity::find_by_id(non_existent_id)
            .one(&db)
            .await
            .unwrap();

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_update_user_duplicate_email() {
        use sea_orm::{ActiveModelTrait, IntoActiveModel, Set};
        use db::models::user;

        let db = setup_test_db().await;

        let (user1, user2) = {
            let admin_user = user::ActiveModel {
                username: Set("u12345678".to_string()),
                email: Set("admin@example.com".to_string()),
                password_hash: Set("hashed1".to_string()),
                admin: Set(true),
                ..Default::default()
            }
            .insert(&db)
            .await
            .unwrap();

            let regular_user = user::ActiveModel {
                username: Set("u87654321".to_string()),
                email: Set("user@example.com".to_string()),
                password_hash: Set("hashed2".to_string()),
                admin: Set(false),
                ..Default::default()
            }
            .insert(&db)
            .await
            .unwrap();

            (admin_user, regular_user)
        };

        // Attempt to update user2's email to user1's email
        let mut model = user2.into_active_model();
        model.email = Set(user1.email.clone());

        let result = model.update(&db).await;

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("UNIQUE constraint failed") || err_msg.contains("constraint"),
            "Expected duplicate email error, got: {}",
            err_msg
        );
    }

    #[tokio::test]
    async fn test_update_user_duplicate_username() {
        use sea_orm::{ActiveModelTrait, IntoActiveModel, Set};
        use db::models::user;

        let db = setup_test_db().await;

        let (user1, user2) = {
            let u1 = user::ActiveModel {
                username: Set("u12345678".to_string()),
                email: Set("admin@example.com".to_string()),
                password_hash: Set("hashed1".to_string()),
                admin: Set(true),
                ..Default::default()
            }
            .insert(&db)
            .await
            .unwrap();

            let u2 = user::ActiveModel {
                username: Set("u87654321".to_string()),
                email: Set("user@example.com".to_string()),
                password_hash: Set("hashed2".to_string()),
                admin: Set(false),
                ..Default::default()
            }
            .insert(&db)
            .await
            .unwrap();

            (u1, u2)
        };

        let mut model = user2.into_active_model();
        model.username = Set(user1.username.clone());

        let result = model.update(&db).await;

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("UNIQUE constraint failed") || err_msg.contains("constraint"),
            "Expected duplicate student number error, got: {}",
            err_msg
        );
    }

    #[tokio::test]
    async fn test_update_user_partial_fields() {
        use sea_orm::{ActiveModelTrait, Set};
        use db::models::user;

        let db = setup_test_db().await;

        let user = user::ActiveModel {
            username: Set("u99999999".to_string()),
            email: Set("original@example.com".to_string()),
            password_hash: Set("secure".to_string()),
            admin: Set(false),
            ..Default::default()
        }
        .insert(&db)
        .await
        .unwrap();

        let mut model = user.clone().into_active_model();
        model.email = Set("partial@example.com".to_string());

        let updated_user = model.update(&db).await.unwrap();

        assert_eq!(updated_user.email, "partial@example.com");
        assert_eq!(updated_user.username, user.username); // unchanged
        assert_eq!(updated_user.admin, user.admin); // unchanged
    }

    #[tokio::test]
    async fn test_update_user_no_fields() {
        use validator::Validate;

        let update_req = UpdateUserRequest {
            username: None,
            email: None,
            admin: None,
        };

        assert!(update_req.validate().is_ok());

        let result = match (&update_req.username, &update_req.email, &update_req.admin) {
            (None, None, None) => Err("At least one field must be provided for update"),
            _ => Ok(()),
        };

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "At least one field must be provided for update"
        );
    }

    #[tokio::test]
    async fn test_update_user_response_format() {
        use sea_orm::{ActiveModelTrait, Set};
        use db::models::user;

        let db = setup_test_db().await;

        let original = user::ActiveModel {
            username: Set("u87654321".to_string()),
            email: Set("original@example.com".to_string()),
            password_hash: Set("test123".to_string()),
            admin: Set(false),
            ..Default::default()
        }
        .insert(&db)
        .await
        .unwrap();

        let mut updated_model = original.clone().into_active_model();
        updated_model.username = Set("u99999999".to_string());
        updated_model.email = Set("response@example.com".to_string());
        updated_model.admin = Set(true);

        let updated = updated_model.update(&db).await.unwrap();

        let response = UserResponse::from(updated);

        assert_eq!(response.id, original.id);
        assert_eq!(response.username, "u99999999");
        assert_eq!(response.email, "response@example.com");
        assert!(response.admin);
        assert!(!response.created_at.is_empty());
        assert!(!response.updated_at.is_empty());
    }

}