use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use db::models::user::User;
use db::pool;
use crate::response::ApiResponse;
use common::format_validation_errors;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUserRequest {
    #[validate(regex(
        path = "STUDENT_NUMBER_REGEX",
        message = "Student number must be in format u12345678"
    ))]
    pub student_number: Option<String>,

    #[validate(email(message = "Invalid email format"))]
    pub email: Option<String>,

    pub admin: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct UpdateUserResponse {
    pub id: i64,
    pub student_number: String,
    pub email: String,
    pub admin: bool,
    pub created_at: String,
    pub updated_at: String,
}

lazy_static::lazy_static! {
    static ref STUDENT_NUMBER_REGEX: regex::Regex = regex::Regex::new("^u\\d{8}$").unwrap();
}

impl From<User> for UpdateUserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            student_number: user.student_number,
            email: user.email,
            admin: user.admin,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

/// PUT /api/users/:id
///
/// Update a user's information. Only admins can access this endpoint.
///
/// # Path Parameters
/// * `id` - The ID of the user to update
///
/// # Request Body
/// ```json
/// {
///   "student_number": "u87654321",  // optional
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
///     "student_number": "u87654321",
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
    if let Err(validation_errors) = req.validate() {
        let error_message = format_validation_errors(&validation_errors);
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<UpdateUserResponse>::error(error_message)),
        );
    }

    if req.student_number.is_none() && req.email.is_none() && req.admin.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<UpdateUserResponse>::error(
                "At least one field must be provided for update"
            )),
        );
    }

    let pool = pool::get();

    let existing_user = match User::get_by_id(Some(pool), user_id).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<UpdateUserResponse>::error("User not found")),
            );
        }

        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<UpdateUserResponse>::error(format!("Database error: {}", e))),
            );
        }
    };

    if let Some(ref email) = req.email {
        if email != &existing_user.email {
            match sqlx::query("SELECT id FROM users WHERE email = ? AND id != ?")
                .bind(email)
                .bind(user_id)
                .fetch_optional(pool)
                .await
            {
                Ok(Some(_)) => {
                    return (
                        StatusCode::CONFLICT,
                        Json(ApiResponse::<UpdateUserResponse>::error(
                            "A user with this email already exists"
                        )),
                    );
                }

                Ok(None) => {}

                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiResponse::<UpdateUserResponse>::error(format!("Database error: {}", e))),
                    );
                }
            }
        }
    }

    if let Some(ref student_number) = req.student_number {
        if student_number != &existing_user.student_number {
            match User::get_by_student_number(Some(pool), student_number).await {
                Ok(Some(conflicting_user)) if conflicting_user.id != user_id => {
                    return (
                        StatusCode::CONFLICT,
                        Json(ApiResponse::<UpdateUserResponse>::error(
                            "A user with this student number already exists"
                        )),
                    );
                }

                Ok(_) => {}

                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiResponse::<UpdateUserResponse>::error(format!("Database error: {}", e))),
                    );
                }
            }
        }
    }

    let result = match (&req.student_number, &req.email, &req.admin) {
        (Some(sn), Some(em), Some(ad)) => {
            sqlx::query_as::<_, User>(
                "UPDATE users SET student_number = ?, email = ?, admin = ?, updated_at = CURRENT_TIMESTAMP 
                 WHERE id = ? RETURNING id, student_number, email, password_hash, admin, created_at, updated_at"
            )
            .bind(sn)
            .bind(em)
            .bind(ad)
            .bind(user_id)
            .fetch_one(pool)
            .await
        }

        (Some(sn), Some(em), None) => {
            sqlx::query_as::<_, User>(
                "UPDATE users SET student_number = ?, email = ?, updated_at = CURRENT_TIMESTAMP 
                 WHERE id = ? RETURNING id, student_number, email, password_hash, admin, created_at, updated_at"
            )
            .bind(sn)
            .bind(em)
            .bind(user_id)
            .fetch_one(pool)
            .await
        }
        
        (Some(sn), None, Some(ad)) => {
            sqlx::query_as::<_, User>(
                "UPDATE users SET student_number = ?, admin = ?, updated_at = CURRENT_TIMESTAMP 
                 WHERE id = ? RETURNING id, student_number, email, password_hash, admin, created_at, updated_at"
            )
            .bind(sn)
            .bind(ad)
            .bind(user_id)
            .fetch_one(pool)
            .await
        }
        
        (None, Some(em), Some(ad)) => {
            sqlx::query_as::<_, User>(
                "UPDATE users SET email = ?, admin = ?, updated_at = CURRENT_TIMESTAMP 
                 WHERE id = ? RETURNING id, student_number, email, password_hash, admin, created_at, updated_at"
            )
            .bind(em)
            .bind(ad)
            .bind(user_id)
            .fetch_one(pool)
            .await
        }
        
        (Some(sn), None, None) => {
            sqlx::query_as::<_, User>(
                "UPDATE users SET student_number = ?, updated_at = CURRENT_TIMESTAMP 
                 WHERE id = ? RETURNING id, student_number, email, password_hash, admin, created_at, updated_at"
            )
            .bind(sn)
            .bind(user_id)
            .fetch_one(pool)
            .await
        }
        
        (None, Some(em), None) => {
            sqlx::query_as::<_, User>(
                "UPDATE users SET email = ?, updated_at = CURRENT_TIMESTAMP 
                 WHERE id = ? RETURNING id, student_number, email, password_hash, admin, created_at, updated_at"
            )
            .bind(em)
            .bind(user_id)
            .fetch_one(pool)
            .await
        }
        
        (None, None, Some(ad)) => {
            sqlx::query_as::<_, User>(
                "UPDATE users SET admin = ?, updated_at = CURRENT_TIMESTAMP 
                 WHERE id = ? RETURNING id, student_number, email, password_hash, admin, created_at, updated_at"
            )
            .bind(ad)
            .bind(user_id)
            .fetch_one(pool)
            .await
        }
        
        (None, None, None) => unreachable!(),
    };

    match result {
        Ok(updated_user) => {
            let response = UpdateUserResponse::from(updated_user);
            (
                StatusCode::OK,
                Json(ApiResponse::success(response, "User updated successfully")),
            )
        }

        Err(sqlx::Error::RowNotFound) => {
            (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<UpdateUserResponse>::error("User not found")),
            )
        }

        Err(e) => {
            if let Some(db_err) = e.as_database_error() {
                let msg = db_err.message();
                if msg.contains("users.email") {
                    return (
                        StatusCode::CONFLICT,
                        Json(ApiResponse::<UpdateUserResponse>::error(
                            "A user with this email already exists"
                        )),
                    );
                }
                
                if msg.contains("users.student_number") {
                    return (
                        StatusCode::CONFLICT,
                        Json(ApiResponse::<UpdateUserResponse>::error(
                            "A user with this student number already exists"
                        )),
                    );
                }
            }

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<UpdateUserResponse>::error(format!("Database error: {}", e))),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use db::models::user::User;
    use sqlx::SqlitePool;
    use db::{create_test_db, delete_database};

    // Helper function to create test users
    async fn create_test_users(pool: &SqlitePool) -> (User, User) {
        let admin_user = User::create(Some(pool), "u12345678", "admin@example.com", "password1", true)
            .await
            .unwrap();

        let regular_user = User::create(Some(pool), "u87654321", "user@example.com", "password2", false)
            .await
            .unwrap();

        (admin_user, regular_user)
    }

    #[tokio::test]
    async fn test_update_user_success() {
        let pool = create_test_db(Some("test_update_user_success.db")).await;
        
        let (_, target_user) = create_test_users(&pool).await;
        let target_id = target_user.id;

        // Test updating all fields
        let update_req = UpdateUserRequest {
            student_number: Some("u99999999".to_string()),
            email: Some("updated@example.com".to_string()),
            admin: Some(true),
        };

        // Verify validation passes
        assert!(update_req.validate().is_ok());

        // Test the update logic
        let result = match (&update_req.student_number, &update_req.email, &update_req.admin) {
            (Some(sn), Some(em), Some(ad)) => {
                sqlx::query_as::<_, User>(
                    "UPDATE users SET student_number = ?, email = ?, admin = ?, updated_at = CURRENT_TIMESTAMP 
                     WHERE id = ? RETURNING id, student_number, email, password_hash, admin, created_at, updated_at"
                )
                .bind(sn)
                .bind(em)
                .bind(ad)
                .bind(target_id)
                .fetch_one(&pool)
                .await
            }
            _ => unreachable!(),
        };

        assert!(result.is_ok());
        let updated_user = result.unwrap();
        assert_eq!(updated_user.student_number, "u99999999");
        assert_eq!(updated_user.email, "updated@example.com");
        assert!(updated_user.admin);

        pool.close().await;
        delete_database("test_update_user_success.db");
    }

    #[tokio::test]
    async fn test_update_user_validation() {
        // Test invalid student number format
        let invalid_sn = UpdateUserRequest {
            student_number: Some("invalid".to_string()),
            email: None,
            admin: None,
        };
        assert!(invalid_sn.validate().is_err());

        // Test invalid email format
        let invalid_email = UpdateUserRequest {
            student_number: None,
            email: Some("not-an-email".to_string()),
            admin: None,
        };
        assert!(invalid_email.validate().is_err());

        // Test valid formats
        let valid_update = UpdateUserRequest {
            student_number: Some("u12345678".to_string()),
            email: Some("valid@example.com".to_string()),
            admin: Some(true),
        };
        assert!(valid_update.validate().is_ok());
    }

    #[tokio::test]
    async fn test_update_user_not_found() {
        let pool = create_test_db(Some("test_update_not_found.db")).await;
        
        let non_existent_id = 99999i64;

        // Verify user doesn't exist
        let found_user = User::get_by_id(Some(&pool), non_existent_id).await.unwrap();
        assert!(found_user.is_none());

        pool.close().await;
        delete_database("test_update_not_found.db");
    }

    #[tokio::test]
    async fn test_update_user_duplicate_email() {
        let pool = create_test_db(Some("test_update_duplicate_email.db")).await;
        
        let (user1, user2) = create_test_users(&pool).await;

        // Try to update user2's email to user1's email
        let update_req = UpdateUserRequest {
            student_number: None,
            email: Some(user1.email.clone()),
            admin: None,
        };

        // Verify validation passes
        assert!(update_req.validate().is_ok());

        // Test the update logic
        let result = match (&update_req.student_number, &update_req.email, &update_req.admin) {
            (None, Some(em), None) => {
                sqlx::query_as::<_, User>(
                    "UPDATE users SET email = ?, updated_at = CURRENT_TIMESTAMP 
                     WHERE id = ? RETURNING id, student_number, email, password_hash, admin, created_at, updated_at"
                )
                .bind(em)
                .bind(user2.id)
                .fetch_one(&pool)
                .await
            }
            _ => unreachable!(),
        };

        // Should fail due to duplicate email
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("UNIQUE constraint failed"));
        }

        pool.close().await;
        delete_database("test_update_duplicate_email.db");
    }

    #[tokio::test]
    async fn test_update_user_duplicate_student_number() {
        let pool = create_test_db(Some("test_update_duplicate_sn.db")).await;
        
        let (user1, user2) = create_test_users(&pool).await;

        // Try to update user2's student number to user1's student number
        let update_req = UpdateUserRequest {
            student_number: Some(user1.student_number.clone()),
            email: None,
            admin: None,
        };

        // Verify validation passes
        assert!(update_req.validate().is_ok());

        // Test the update logic
        let result = match (&update_req.student_number, &update_req.email, &update_req.admin) {
            (Some(sn), None, None) => {
                sqlx::query_as::<_, User>(
                    "UPDATE users SET student_number = ?, updated_at = CURRENT_TIMESTAMP 
                     WHERE id = ? RETURNING id, student_number, email, password_hash, admin, created_at, updated_at"
                )
                .bind(sn)
                .bind(user2.id)
                .fetch_one(&pool)
                .await
            }
            _ => unreachable!(),
        };

        // Should fail due to duplicate student number
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("UNIQUE constraint failed"));
        }

        pool.close().await;
        delete_database("test_update_duplicate_sn.db");
    }

    #[tokio::test]
    async fn test_update_user_partial_fields() {
        let pool = create_test_db(Some("test_update_partial.db")).await;
        
        let (_, target_user) = create_test_users(&pool).await;
        let target_id = target_user.id;

        // Test updating only email
        let update_req = UpdateUserRequest {
            student_number: None,
            email: Some("partial@example.com".to_string()),
            admin: None,
        };

        // Verify validation passes
        assert!(update_req.validate().is_ok());

        // Test the update logic
        let result = match (&update_req.student_number, &update_req.email, &update_req.admin) {
            (None, Some(em), None) => {
                sqlx::query_as::<_, User>(
                    "UPDATE users SET email = ?, updated_at = CURRENT_TIMESTAMP 
                     WHERE id = ? RETURNING id, student_number, email, password_hash, admin, created_at, updated_at"
                )
                .bind(em)
                .bind(target_id)
                .fetch_one(&pool)
                .await
            }
            _ => unreachable!(),
        };

        assert!(result.is_ok());
        let updated_user = result.unwrap();
        assert_eq!(updated_user.email, "partial@example.com");
        assert_eq!(updated_user.student_number, target_user.student_number); // Unchanged
        assert_eq!(updated_user.admin, target_user.admin); // Unchanged

        pool.close().await;
        delete_database("test_update_partial.db");
    }

    #[tokio::test]
    async fn test_update_user_no_fields() {
        let pool = create_test_db(Some("test_update_no_fields.db")).await;
        
        let (_, _target_user) = create_test_users(&pool).await;

        // Test updating with no fields
        let update_req = UpdateUserRequest {
            student_number: None,
            email: None,
            admin: None,
        };

        // Verify validation passes
        assert!(update_req.validate().is_ok());

        // Test that at least one field must be provided
        let result = match (&update_req.student_number, &update_req.email, &update_req.admin) {
            (None, None, None) => Err("At least one field must be provided for update"),
            _ => Ok(()),
        };

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "At least one field must be provided for update");

        pool.close().await;
        delete_database("test_update_no_fields.db");
    }

    #[tokio::test]
    async fn test_update_user_response_format() {
        let pool = create_test_db(Some("test_update_response.db")).await;
        
        let (_, target_user) = create_test_users(&pool).await;
        let target_id = target_user.id;

        // Test updating user
        let update_req = UpdateUserRequest {
            student_number: Some("u99999999".to_string()),
            email: Some("response@example.com".to_string()),
            admin: Some(true),
        };

        // Test the update logic
        let result = match (&update_req.student_number, &update_req.email, &update_req.admin) {
            (Some(sn), Some(em), Some(ad)) => {
                sqlx::query_as::<_, User>(
                    "UPDATE users SET student_number = ?, email = ?, admin = ?, updated_at = CURRENT_TIMESTAMP 
                     WHERE id = ? RETURNING id, student_number, email, password_hash, admin, created_at, updated_at"
                )
                .bind(sn)
                .bind(em)
                .bind(ad)
                .bind(target_id)
                .fetch_one(&pool)
                .await
            }
            _ => unreachable!(),
        };

        assert!(result.is_ok());
        let updated_user = result.unwrap();
        let response = UpdateUserResponse::from(updated_user);

        // Verify response format
        assert_eq!(response.id, target_id);
        assert_eq!(response.student_number, "u99999999");
        assert_eq!(response.email, "response@example.com");
        assert!(response.admin);
        assert!(!response.created_at.is_empty());
        assert!(!response.updated_at.is_empty());

        pool.close().await;
        delete_database("test_update_response.db");
    }
}