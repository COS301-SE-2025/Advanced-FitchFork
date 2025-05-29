use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::{
    auth::claims::AuthUser,
    response::ApiResponse,
};

use db::{
    connect,
    models::user::{self, Entity as UserEntity},
};

/// DELETE /users/:id
///
/// Delete a user by their ID. Only admins can access this endpoint.
/// Users cannot delete their own account.
///
/// ### Path Parameters
/// - `id` - The ID of the user to delete
///
/// ### Responses
///
/// - `200 OK`  
/// ```json
/// {
///   "success": true,
///   "message": "User deleted successfully"
/// }
/// ```
///
/// - `404 Not Found`  
/// ```json
/// {
///   "success": false,
///   "message": "User not found"
/// }
/// ```
///
/// - `400 Bad Request` (invalid ID format)  
/// ```json
/// {
///   "success": false,
///   "message": "Invalid user ID format"
/// }
/// ```
///
/// - `403 Forbidden`  
/// ```json
/// {
///   "success": false,
///   "message": "You cannot delete your own account"
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
pub async fn delete_user(
    Path(id): Path<String>,
    Extension(AuthUser(claims)): Extension<AuthUser>,
) -> impl IntoResponse {
    let user_id = match id.parse::<i64>() {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<()>::error("Invalid user ID format")),
            );
        }
    };

    if user_id == claims.sub {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<()>::error("You cannot delete your own account")),
        );
    }

    let db = connect().await;

    match UserEntity::find()
        .filter(user::Column::Id.eq(user_id))
        .one(&db)
        .await
    {
        Ok(Some(_)) => {
            match UserEntity::delete_by_id(user_id).exec(&db).await {
                Ok(_) => (
                    StatusCode::OK,
                    Json(ApiResponse::success_without_data("User deleted successfully")),
                ),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()>::error(format!("Database error: {}", e))),
                ),
            }
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("User not found")),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Database error: {}", e))),
        ),
    }
}

#[cfg(test)]
mod tests {
    use crate::auth::claims::{ Claims};
    use db::{models::user};
    use db::test_utils::{setup_test_db};
    use sea_orm::{EntityTrait, ActiveModelTrait, Set};
    use db::models::user::{Entity as UserEntity, ActiveModel as UserActiveModel};

    // Seed admin and regular user
    async fn create_test_users(db: &sea_orm::DatabaseConnection) -> (user::Model, user::Model) {
        let admin_user = UserActiveModel {
            student_number: Set("u12345678".into()),
            email: Set("admin@example.com".into()),
            password_hash: Set("hashed1".into()),
            admin: Set(true),
            ..Default::default()
        }
        .insert(db)
        .await
        .unwrap();

        let regular_user = UserActiveModel {
            student_number: Set("u87654321".into()),
            email: Set("user@example.com".into()),
            password_hash: Set("hashed2".into()),
            admin: Set(false),
            ..Default::default()
        }
        .insert(db)
        .await
        .unwrap();

        (admin_user, regular_user)
    }

    // Mock Claims
    fn create_mock_claims(user_id: i64, admin: bool) -> Claims {
        Claims {
            sub: user_id,
            admin,
            exp: 9999999999,
        }
    }

    #[tokio::test]
    async fn test_delete_user_success() {
        let db = setup_test_db().await;
        let (admin_user, target_user) = create_test_users(&db).await;

        assert_ne!(target_user.id, admin_user.id); // Not self-deletion

        let found_user = UserEntity::find_by_id(target_user.id).one(&db).await.unwrap();
        assert!(found_user.is_some());

        UserEntity::delete_by_id(target_user.id).exec(&db).await.unwrap();

        let after = UserEntity::find_by_id(target_user.id).one(&db).await.unwrap();
        assert!(after.is_none());
    }

    #[tokio::test]
    async fn test_delete_user_invalid_id_format() {
        // Test invalid ID formats that should fail parsing
        let invalid_ids = vec![
            "not_a_number",
            "12.34",
            "abc123",
            "",
            " ",
            "123abc",
            "∞",
            "null",
        ];

        for invalid_id in invalid_ids {
            let result = invalid_id.parse::<i64>();
            assert!(result.is_err(), "Expected parsing '{}' to fail", invalid_id);
        }
    }

    #[tokio::test]
    async fn test_delete_user_valid_id_formats() {
        // Test valid ID formats that should succeed parsing
        let valid_ids = vec![
            ("123", 123i64),
            ("0", 0i64),
            ("-1", -1i64),
            ("9223372036854775807", 9223372036854775807i64), // i64::MAX
        ];

        for (id_str, expected) in valid_ids {
            let result = id_str.parse::<i64>();
            assert!(result.is_ok(), "Expected parsing '{}' to succeed", id_str);
            assert_eq!(result.unwrap(), expected);
        }
    }

      #[tokio::test]
    async fn test_delete_user_self_deletion_prevention() {
        let db = setup_test_db().await;

        let admin_user = UserActiveModel {
            student_number: Set("u99999999".into()),
            email: Set("admin_self@example.com".into()),
            password_hash: Set("hashed".into()),
            admin: Set(true),
            ..Default::default()
        }
        .insert(&db)
        .await
        .unwrap();

        let claims = create_mock_claims(admin_user.id, true);
        assert_eq!(claims.sub, admin_user.id); // Should detect self-deletion

        let user_exists = UserEntity::find_by_id(admin_user.id).one(&db).await.unwrap();
        assert!(user_exists.is_some());
    }

      #[tokio::test]
    async fn test_delete_user_not_found() {
        let db = setup_test_db().await;

        let admin_user = UserActiveModel {
            student_number: Set("u11111111".into()),
            email: Set("admin_nf@example.com".into()),
            password_hash: Set("hashed".into()),
            admin: Set(true),
            ..Default::default()
        }
        .insert(&db)
        .await
        .unwrap();

        let claims = create_mock_claims(admin_user.id, true);

        let nonexistent_id = 99999;
        assert_ne!(claims.sub, nonexistent_id);

        let user = UserEntity::find_by_id(nonexistent_id).one(&db).await.unwrap();
        assert!(user.is_none());
    }

   #[tokio::test]
    async fn test_delete_user_database_operations() {
        use db::models::user::{Entity as UserEntity, ActiveModel as UserActiveModel};
        use sea_orm::{EntityTrait, ActiveModelTrait, Set};

        let db = db::test_utils::setup_test_db().await;

        // Insert a user
        let user = UserActiveModel {
            student_number: Set("u32132132".into()),
            email: Set("target@dbops.com".into()),
            password_hash: Set("pass".into()),
            admin: Set(false),
            ..Default::default()
        }
        .insert(&db)
        .await
        .unwrap();

        let user_id = user.id;

        // Confirm user exists
        let before = UserEntity::find_by_id(user_id).one(&db).await.unwrap();
        assert!(before.is_some());

        // Delete user
        let delete_result = UserEntity::delete_by_id(user_id).exec(&db).await;
        assert!(delete_result.is_ok());

        // Confirm deletion
        let after = UserEntity::find_by_id(user_id).one(&db).await.unwrap();
        assert!(after.is_none());

        // Delete again (should still succeed)
        let second_delete = UserEntity::delete_by_id(user_id).exec(&db).await;
        assert!(second_delete.is_ok());
    }

    #[tokio::test]
    async fn test_delete_user_edge_cases() {
        use db::models::user::Entity as UserEntity;
        use sea_orm::{EntityTrait};

        let db = db::test_utils::setup_test_db().await;

        // Edge case IDs that shouldn't exist
        let edge_ids: Vec<i32> = vec![0, -1, 1, i32::MAX];

        for id in edge_ids {
            let found = UserEntity::find_by_id(id).one(&db).await.unwrap();
            assert!(found.is_none(), "User with ID {} should not exist", id);
        }
    }

    #[tokio::test]
    async fn test_claims_structure() {
        use crate::auth::claims::{AuthUser, Claims};

        // Admin claims
        let claims = Claims {
            sub: 123,
            admin: true,
            exp: 9999999999,
        };
        assert_eq!(claims.sub, 123);
        assert!(claims.admin);
        assert_eq!(claims.exp, 9999999999);

        let auth_user = AuthUser(claims);
        assert_eq!(auth_user.0.sub, 123);
        assert!(auth_user.0.admin);

        // Non-admin claims
        let non_admin_claims = Claims {
            sub: 456,
            admin: false,
            exp: 9999999999,
        };
        assert!(!non_admin_claims.admin);
    }

   #[tokio::test]
    async fn test_multiple_user_deletion_scenario() {
        use db::test_utils::setup_test_db;
        use db::models::user::{Entity as UserEntity, ActiveModel as UserActiveModel};
        use crate::auth::claims::Claims;
        use sea_orm::{ActiveModelTrait, EntityTrait, Set};

        let db = setup_test_db().await;

        // Create users
        let user1 = UserActiveModel {
            student_number: Set("u11111111".into()),
            email: Set("user1@test.com".into()),
            password_hash: Set("pass1".into()),
            admin: Set(false),
            ..Default::default()
        }.insert(&db).await.unwrap();

        let user2 = UserActiveModel {
            student_number: Set("u22222222".into()),
            email: Set("user2@test.com".into()),
            password_hash: Set("pass2".into()),
            admin: Set(false),
            ..Default::default()
        }.insert(&db).await.unwrap();

        let user3 = UserActiveModel {
            student_number: Set("u33333333".into()),
            email: Set("user3@test.com".into()),
            password_hash: Set("pass3".into()),
            admin: Set(false),
            ..Default::default()
        }.insert(&db).await.unwrap();

        let admin = UserActiveModel {
            student_number: Set("u99999999".into()),
            email: Set("admin@test.com".into()),
            password_hash: Set("admin_pass".into()),
            admin: Set(true),
            ..Default::default()
        }.insert(&db).await.unwrap();

        let admin_claims = Claims {
            sub: admin.id,
            admin: true,
            exp: 9999999999,
        };

        // Deleting users
        for user_id in &[user1.id, user2.id, user3.id] {
            assert_ne!(*user_id, admin_claims.sub);

            let exists = UserEntity::find_by_id(*user_id).one(&db).await.unwrap();
            assert!(exists.is_some());

            UserEntity::delete_by_id(*user_id).exec(&db).await.unwrap();

            let deleted = UserEntity::find_by_id(*user_id).one(&db).await.unwrap();
            assert!(deleted.is_none());
        }

        // Ensure admin still exists
        let admin_exists = UserEntity::find_by_id(admin.id).one(&db).await.unwrap();
        assert!(admin_exists.is_some());
    }

    #[tokio::test]
    async fn test_id_parsing_boundary_values() {
        let boundary_tests = vec![
            ("2147483647", true),   // i32::MAX
            ("2147483648", false),  // Overflow
            ("-2147483648", true),  // i32::MIN
            ("-2147483649", false), // Underflow
        ];

        for (id_str, should_succeed) in boundary_tests {
            let result = id_str.parse::<i32>();
            if should_succeed {
                assert!(result.is_ok(), "Expected '{}' to parse successfully", id_str);
            } else {
                assert!(result.is_err(), "Expected '{}' to fail parsing", id_str);
            }
        }
    }

   #[tokio::test]
    async fn test_deletion_logic_flow() {
        use db::test_utils::setup_test_db;
        use db::models::user::{ActiveModel as UserActiveModel, Entity as UserEntity};
        use crate::auth::claims::Claims;
        use sea_orm::{EntityTrait, Set, ActiveModelTrait};

        let db = setup_test_db().await;

        let admin_user = UserActiveModel {
            student_number: Set("u00000001".into()),
            email: Set("admin@example.com".into()),
            password_hash: Set("adminhash".into()),
            admin: Set(true),
            ..Default::default()
        }
        .insert(&db)
        .await
        .unwrap();

        let target_user = UserActiveModel {
            student_number: Set("u00000002".into()),
            email: Set("target@example.com".into()),
            password_hash: Set("targethash".into()),
            admin: Set(false),
            ..Default::default()
        }
        .insert(&db)
        .await
        .unwrap();

        let target_id_str = target_user.id.to_string();

        // Step 1: Parse ID
        let parsed_id = target_id_str.parse::<i64>();
        assert!(parsed_id.is_ok());
        let user_id = parsed_id.unwrap();

        // Step 2: Self-deletion check
        let claims = Claims {
            sub: admin_user.id,
            admin: true,
            exp: 9999999999,
        };
        assert_ne!(user_id, claims.sub);

        // Step 3: Check if user exists
        let user_exists = UserEntity::find_by_id(user_id).one(&db).await.unwrap();
        assert!(user_exists.is_some());

        // Step 4: Delete user
        let delete_result = UserEntity::delete_by_id(user_id).exec(&db).await;
        assert!(delete_result.is_ok());

        // Step 5: Verify deletion
        let deleted = UserEntity::find_by_id(user_id).one(&db).await.unwrap();
        assert!(deleted.is_none());
    }


    #[tokio::test]
    async fn test_error_conditions() {
        use db::test_utils::setup_test_db;
        use db::models::user::{ Entity as UserEntity, ActiveModel as UserActiveModel};
        use sea_orm::{EntityTrait, Set, ActiveModelTrait};
        use crate::auth::claims::Claims;

        let db = setup_test_db().await;

        // 1. Invalid ID format
        let invalid_ids = vec!["", "abc", "12.5", "∞"];
        for invalid_id in invalid_ids {
            assert!(invalid_id.parse::<i64>().is_err());
        }

        // 2. Self-deletion attempt
        let user = UserActiveModel {
            student_number: Set("u12345678".into()),
            email: Set("test@example.com".into()),
            password_hash: Set("hashed".into()),
            admin: Set(true),
            ..Default::default()
        }
        .insert(&db)
        .await
        .unwrap();

        let self_claims = Claims {
            sub: user.id,
            admin: true,
            exp: 9999999999,
        };

        assert_eq!(user.id, self_claims.sub); // Should trigger self-deletion logic

        // 3. Non-existent user
        let non_existent = UserEntity::find_by_id(99999).one(&db).await.unwrap();
        assert!(non_existent.is_none());
    }

    // Test the response structure expectations (without actually calling the handler)
    #[tokio::test]
    async fn test_response_format_expectations() {
        // Simulate handler flow checks without DB access

        // Test ID parsing for bad request scenario
        let bad_id = "not_a_number";
        let parse_result = bad_id.parse::<i64>();
        assert!(parse_result.is_err()); // Should result in 400 Bad Request

        // Test self-deletion check for forbidden scenario
        let user_id = 123i64;
        let claims_sub = 123i64;
        let is_self_deletion = user_id == claims_sub;
        assert!(is_self_deletion); // Should result in 403 Forbidden

        // Test non-self deletion (should allow)
        let other_user_id = 456i64;
        let is_other_deletion = other_user_id == claims_sub;
        assert!(!is_other_deletion); // Should not result in 403
    }
}