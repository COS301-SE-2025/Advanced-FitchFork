use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter, Set,
};

use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{response::ApiResponse};
use common::format_validation_errors;
use db::{connect, models::user};

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

impl From<user::Model> for UpdateUserResponse {
    fn from(user: user::Model) -> Self {
        Self {
            id: user.id,
            student_number: user.student_number,
            email: user.email,
            admin: user.admin,
            created_at: user.created_at.to_string(),
            updated_at: user.updated_at.to_string(),
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
    Path(user_id): Path<i32>,
    Json(req): Json<UpdateUserRequest>,
) -> impl IntoResponse {
    if let Err(e) = req.validate() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<UpdateUserResponse>::error(format_validation_errors(&e))),
        );
    }

    if req.student_number.is_none() && req.email.is_none() && req.admin.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<UpdateUserResponse>::error("At least one field must be provided")),
        );
    }

    let db: DatabaseConnection = connect().await;

    let user_entity = user::Entity::find_by_id(user_id)
        .one(&db)
        .await;

    let current_user = match user_entity {
        Ok(Some(u)) => u,
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

    // TODO: Should probably make a more robust system with a super admin
    // Prevent changing your own admin status or changing others' admin status
    if let Some(_) = req.admin {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<UpdateUserResponse>::error("Changing admin status is not allowed")),
        );
    }

    // Check email conflict
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
                        Json(ApiResponse::<UpdateUserResponse>::error("A user with this email already exists")),
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

    // Check student_number conflict
    if let Some(sn) = &req.student_number {
        if sn != &current_user.student_number {
            let exists_result = user::Entity::find()
                .filter(
                    Condition::all()
                        .add(user::Column::StudentNumber.eq(sn.clone()))
                        .add(user::Column::Id.ne(user_id)),
                )
                .one(&db)
                .await;

            match exists_result {
                Ok(Some(_)) => {
                    return (
                        StatusCode::CONFLICT,
                        Json(ApiResponse::<UpdateUserResponse>::error(
                            "A user with this student number already exists",
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

    let mut active_model: user::ActiveModel = current_user.into();
    if let Some(sn) = req.student_number {
        active_model.student_number = Set(sn);
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
                UpdateUserResponse::from(updated),
                "User updated successfully",
            )),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<UpdateUserResponse>::error(format!("Database error: {}", e))),
        ),
    }
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
            student_number: Set("u12345678".to_string()),
            email: Set("admin@example.com".to_string()),
            password_hash: Set("hashed1".to_string()),
            admin: Set(true),
            ..Default::default()
        }
        .insert(db)
        .await
        .unwrap();

        let regular_user = user::ActiveModel {
            student_number: Set("u87654321".to_string()),
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
            student_number: Some("u99999999".to_string()),
            email: Some("updated@example.com".to_string()),
            admin: Some(true),
        };

        assert!(update_req.validate().is_ok());

        let mut model = target_user.into_active_model();
        if let Some(sn) = update_req.student_number.clone() {
            model.student_number = Set(sn);
        }
        if let Some(em) = update_req.email.clone() {
            model.email = Set(em);
        }
        if let Some(ad) = update_req.admin {
            model.admin = Set(ad);
        }

        let updated = model.update(&db).await.unwrap();

        assert_eq!(updated.student_number, "u99999999");
        assert_eq!(updated.email, "updated@example.com");
        assert!(updated.admin);
    }

    #[tokio::test]
    async fn test_update_user_validation() {
        // Invalid student number
        let invalid_sn = UpdateUserRequest {
            student_number: Some("invalid".to_string()),
            email: None,
            admin: None,
        };
        assert!(invalid_sn.validate().is_err());

        // Invalid email
        let invalid_email = UpdateUserRequest {
            student_number: None,
            email: Some("not-an-email".to_string()),
            admin: None,
        };
        assert!(invalid_email.validate().is_err());

        // Valid case
        let valid_update = UpdateUserRequest {
            student_number: Some("u12345678".to_string()),
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
                student_number: Set("u12345678".to_string()),
                email: Set("admin@example.com".to_string()),
                password_hash: Set("hashed1".to_string()),
                admin: Set(true),
                ..Default::default()
            }
            .insert(&db)
            .await
            .unwrap();

            let regular_user = user::ActiveModel {
                student_number: Set("u87654321".to_string()),
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
    async fn test_update_user_duplicate_student_number() {
        use sea_orm::{ActiveModelTrait, IntoActiveModel, Set};
        use db::models::user;

        let db = setup_test_db().await;

        let (user1, user2) = {
            let u1 = user::ActiveModel {
                student_number: Set("u12345678".to_string()),
                email: Set("admin@example.com".to_string()),
                password_hash: Set("hashed1".to_string()),
                admin: Set(true),
                ..Default::default()
            }
            .insert(&db)
            .await
            .unwrap();

            let u2 = user::ActiveModel {
                student_number: Set("u87654321".to_string()),
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
        model.student_number = Set(user1.student_number.clone());

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
            student_number: Set("u99999999".to_string()),
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
        assert_eq!(updated_user.student_number, user.student_number); // unchanged
        assert_eq!(updated_user.admin, user.admin); // unchanged
    }

    #[tokio::test]
    async fn test_update_user_no_fields() {
        use validator::Validate;

        let update_req = UpdateUserRequest {
            student_number: None,
            email: None,
            admin: None,
        };

        assert!(update_req.validate().is_ok());

        let result = match (&update_req.student_number, &update_req.email, &update_req.admin) {
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
            student_number: Set("u87654321".to_string()),
            email: Set("original@example.com".to_string()),
            password_hash: Set("test123".to_string()),
            admin: Set(false),
            ..Default::default()
        }
        .insert(&db)
        .await
        .unwrap();

        let mut updated_model = original.clone().into_active_model();
        updated_model.student_number = Set("u99999999".to_string());
        updated_model.email = Set("response@example.com".to_string());
        updated_model.admin = Set(true);

        let updated = updated_model.update(&db).await.unwrap();

        let response = UpdateUserResponse::from(updated);

        assert_eq!(response.id, original.id);
        assert_eq!(response.student_number, "u99999999");
        assert_eq!(response.email, "response@example.com");
        assert!(response.admin);
        assert!(!response.created_at.is_empty());
        assert!(!response.updated_at.is_empty());
    }

}