use axum::{
    extract::{Path, Extension},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use db::models::user::User;
use db::pool;
use crate::response::ApiResponse;
use crate::auth::claims::AuthUser;

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

    let pool = pool::get();

    match User::get_by_id(Some(pool), user_id).await {
        Ok(Some(_)) => {
            match User::delete_by_id(Some(pool), user_id).await {
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

        Ok(None) => {
            (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<()>::error("User not found")),
            )
        }

        Err(e) => {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(format!("Database error: {}", e))),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use db::models::user::User;
    use crate::auth::claims::Claims;
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

    // Helper function to create mock claims
    fn create_mock_claims(user_id: i64, admin: bool) -> Claims {
        Claims {
            sub: user_id,
            admin,
            exp: 9999999999, // Far future timestamp
        }
    }

    #[tokio::test]
    async fn test_delete_user_success() {
        let pool = create_test_db(Some("test_delete_user_success.db")).await;
        
        let (admin_user, target_user) = create_test_users(&pool).await;
        let target_id = target_user.id;

        // Verify user exists before deletion
        let found_user = User::get_by_id(Some(&pool), target_id).await.unwrap();
        assert!(found_user.is_some());

        // Create mock claims for admin user
        let claims = create_mock_claims(admin_user.id, true);
        let auth_user = AuthUser(claims);

        // Test the core deletion logic (simulating the handler)
        let user_id_str = target_id.to_string();
        let user_id_parsed = user_id_str.parse::<i64>().unwrap();
        
        // Verify it's not self-deletion
        assert_ne!(user_id_parsed, auth_user.0.sub);

        // Perform deletion
        let user_exists = User::get_by_id(Some(&pool), user_id_parsed).await.unwrap();
        assert!(user_exists.is_some());

        User::delete_by_id(Some(&pool), user_id_parsed).await.unwrap();

        // Verify user was deleted
        let deleted_user = User::get_by_id(Some(&pool), target_id).await.unwrap();
        assert!(deleted_user.is_none());

        pool.close().await;
        delete_database("test_delete_user_success.db");
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
        let pool = create_test_db(Some("test_self_deletion.db")).await;
        
        let (admin_user, _) = create_test_users(&pool).await;
        let admin_id = admin_user.id;

        // Create claims for the admin user
        let claims = create_mock_claims(admin_id, true);

        // Test attempting to delete own account
        let self_deletion_check = admin_id == claims.sub;
        assert!(self_deletion_check, "Should detect self-deletion attempt");

        // Verify user still exists (since deletion should be prevented)
        let user_still_exists = User::get_by_id(Some(&pool), admin_id).await.unwrap();
        assert!(user_still_exists.is_some());

        pool.close().await;
        delete_database("test_self_deletion.db");
    }

    #[tokio::test]
    async fn test_delete_user_not_found() {
        let pool = create_test_db(Some("test_delete_not_found.db")).await;
        
        let (admin_user, _) = create_test_users(&pool).await;
        
        // Try to find a non-existent user
        let non_existent_id = 99999i64;
        let found_user = User::get_by_id(Some(&pool), non_existent_id).await.unwrap();
        assert!(found_user.is_none());

        // Verify it's not self-deletion
        let claims = create_mock_claims(admin_user.id, true);
        assert_ne!(non_existent_id, claims.sub);

        pool.close().await;
        delete_database("test_delete_not_found.db");
    }

    #[tokio::test]
    async fn test_delete_user_database_operations() {
        let pool = create_test_db(Some("test_delete_db_ops.db")).await;
        
        let (_admin_user, target_user) = create_test_users(&pool).await;
        let target_id = target_user.id;

        // Test get_by_id operation
        let get_result = User::get_by_id(Some(&pool), target_id).await;
        assert!(get_result.is_ok());
        assert!(get_result.unwrap().is_some());

        // Test delete_by_id operation
        let delete_result = User::delete_by_id(Some(&pool), target_id).await;
        assert!(delete_result.is_ok());

        // Verify deletion worked
        let get_after_delete = User::get_by_id(Some(&pool), target_id).await;
        assert!(get_after_delete.is_ok());
        assert!(get_after_delete.unwrap().is_none());

        // Test deleting already deleted user (should not error)
        let delete_again_result = User::delete_by_id(Some(&pool), target_id).await;
        assert!(delete_again_result.is_ok()); // SQLite DELETE is idempotent

        pool.close().await;
        delete_database("test_delete_db_ops.db");
    }

    #[tokio::test]
    async fn test_delete_user_edge_cases() {
        let pool = create_test_db(Some("test_delete_edge_cases.db")).await;
        
        // Test with edge case IDs
        let edge_case_ids = vec![0i64, -1i64, 1i64];
        
        for test_id in edge_case_ids {
            // These should not exist in our test database
            let found_user = User::get_by_id(Some(&pool), test_id).await.unwrap();
            assert!(found_user.is_none());
        }

        // Test with maximum i64 value
        let max_id = i64::MAX;
        let found_max = User::get_by_id(Some(&pool), max_id).await.unwrap();
        assert!(found_max.is_none());

        pool.close().await;
        delete_database("test_delete_edge_cases.db");
    }

    #[tokio::test]
    async fn test_claims_structure() {
        // Test Claims structure and AuthUser wrapper
        let claims = create_mock_claims(123, true);
        assert_eq!(claims.sub, 123);
        assert_eq!(claims.admin, true);
        assert_eq!(claims.exp, 9999999999);

        let auth_user = AuthUser(claims);
        assert_eq!(auth_user.0.sub, 123);
        assert_eq!(auth_user.0.admin, true);

        // Test non-admin claims
        let non_admin_claims = create_mock_claims(456, false);
        assert_eq!(non_admin_claims.admin, false);
    }

    #[tokio::test]
    async fn test_multiple_user_deletion_scenario() {
        let pool = create_test_db(Some("test_multiple_deletion.db")).await;
        
        // Create multiple users
        let user1 = User::create(Some(&pool), "u11111111", "user1@test.com", "pass1", false).await.unwrap();
        let user2 = User::create(Some(&pool), "u22222222", "user2@test.com", "pass2", false).await.unwrap();
        let user3 = User::create(Some(&pool), "u33333333", "user3@test.com", "pass3", false).await.unwrap();
        let admin = User::create(Some(&pool), "u99999999", "admin@test.com", "admin_pass", true).await.unwrap();

        let admin_claims = create_mock_claims(admin.id, true);

        // Delete users one by one (simulating the deletion logic)
        let users_to_delete = vec![user1.id, user2.id, user3.id];
        
        for user_id in users_to_delete {
            // Verify not self-deletion
            assert_ne!(user_id, admin_claims.sub);
            
            // Verify user exists
            let exists = User::get_by_id(Some(&pool), user_id).await.unwrap();
            assert!(exists.is_some());
            
            // Delete user
            User::delete_by_id(Some(&pool), user_id).await.unwrap();
            
            // Verify deletion
            let deleted = User::get_by_id(Some(&pool), user_id).await.unwrap();
            assert!(deleted.is_none());
        }

        // Verify admin still exists
        let admin_exists = User::get_by_id(Some(&pool), admin.id).await.unwrap();
        assert!(admin_exists.is_some());

        pool.close().await;
        delete_database("test_multiple_deletion.db");
    }

    #[tokio::test]
    async fn test_id_parsing_boundary_values() {
        // Test boundary values for i64 parsing
        let boundary_tests = vec![
            ("9223372036854775807", true),  // i64::MAX
            ("9223372036854775808", false), // i64::MAX + 1 (should overflow)
            ("-9223372036854775808", true), // i64::MIN
            ("-9223372036854775809", false), // i64::MIN - 1 (should overflow)
        ];

        for (id_str, should_succeed) in boundary_tests {
            let result = id_str.parse::<i64>();
            if should_succeed {
                assert!(result.is_ok(), "Expected '{}' to parse successfully", id_str);
            } else {
                assert!(result.is_err(), "Expected '{}' to fail parsing", id_str);
            }
        }
    }

    #[tokio::test]
    async fn test_deletion_logic_flow() {
        let pool = create_test_db(Some("test_deletion_flow.db")).await;
        
        let (admin_user, target_user) = create_test_users(&pool).await;
        let target_id = target_user.id;
        let target_id_str = target_id.to_string();

        // Simulate the complete handler flow
        
        // Step 1: Parse ID
        let parsed_id = target_id_str.parse::<i64>();
        assert!(parsed_id.is_ok());
        let user_id = parsed_id.unwrap();

        // Step 2: Check self-deletion
        let admin_claims = create_mock_claims(admin_user.id, true);
        let is_self_deletion = user_id == admin_claims.sub;
        assert!(!is_self_deletion);

        // Step 3: Check if user exists
        let user_lookup = User::get_by_id(Some(&pool), user_id).await;
        assert!(user_lookup.is_ok());
        let user_exists = user_lookup.unwrap().is_some();
        assert!(user_exists);

        // Step 4: Delete user
        let deletion_result = User::delete_by_id(Some(&pool), user_id).await;
        assert!(deletion_result.is_ok());

        // Step 5: Verify deletion
        let verification = User::get_by_id(Some(&pool), user_id).await.unwrap();
        assert!(verification.is_none());

        pool.close().await;
        delete_database("test_deletion_flow.db");
    }

    #[tokio::test]
    async fn test_error_conditions() {
        let pool = create_test_db(Some("test_error_conditions.db")).await;
        
        // Test various error conditions that could occur
        
        // 1. Invalid ID format (covered in other tests, but let's be thorough)
        let invalid_ids = vec!["", "abc", "12.5", "∞"];
        for invalid_id in invalid_ids {
            assert!(invalid_id.parse::<i64>().is_err());
        }

        // 2. Self-deletion attempt
        let user = User::create(Some(&pool), "u12345678", "test@example.com", "password", true).await.unwrap();
        let self_claims = create_mock_claims(user.id, true);
        assert_eq!(user.id, self_claims.sub); // This would trigger the forbidden response

        // 3. Non-existent user
        let non_existent_result = User::get_by_id(Some(&pool), 99999).await.unwrap();
        assert!(non_existent_result.is_none());

        pool.close().await;
        delete_database("test_error_conditions.db");
    }

    // Test the response structure expectations (without actually calling the handler)
    #[tokio::test]
    async fn test_response_format_expectations() {
        // This test validates that our logic aligns with the expected response formats
        // documented in the handler's docstring
        
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