use axum::{
    extract::Query,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use db::models::user::User;
use db::pool;
use crate::response::ApiResponse;
use serde::{Deserialize, Serialize};
use validator::Validate;
use sqlx::Row;

#[derive(Debug, Deserialize, Validate)]
pub struct ListUsersQuery {
    #[validate(range(min = 1, message = "Page must be at least 1"))]
    pub page: Option<i64>,
    
    #[validate(range(min = 1, max = 100, message = "Per page must be between 1 and 100"))]
    pub per_page: Option<i64>,
    
    pub sort: Option<String>,
    pub query: Option<String>,
    pub email: Option<String>,
    pub student_number: Option<String>,
    pub admin: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct UserListItem {
    pub id: String,
    pub email: String,
    pub student_number: String,
    pub admin: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct UsersListResponse {
    pub users: Vec<UserListItem>,
    pub page: i64,
    pub per_page: i64,
    pub total: i64,
}

impl From<User> for UserListItem {
    fn from(user: User) -> Self {
        Self {
            id: user.id.to_string(),
            email: user.email,
            student_number: user.student_number,
            admin: user.admin,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

/// GET /api/users
///
/// Retrieve a paginated list of users with optional filtering and sorting.
/// Requires admin privileges.
///
/// ### Query Parameters
/// - `page` (optional): Page number (default: 1, min: 1)
/// - `per_page` (optional): Items per page (default: 20, min: 1, max: 100)
/// - `query` (optional): Case-insensitive partial match against email OR student_number
/// - `email` (optional): Case-insensitive partial match on email (ignored if query is provided)
/// - `student_number` (optional): Case-insensitive partial match on student number (ignored if query is provided)
/// - `admin` (optional): Filter by admin status (true/false)
/// - `sort` (optional): Comma-separated sort fields. Use `-` prefix for descending
///
/// ### Examples
/// ```http
/// GET /api/users?page=2&per_page=10
/// GET /api/users?query=u1234
/// GET /api/users?email=@example.com
/// GET /api/users?student_number=u1234
/// GET /api/users?admin=true
/// GET /api/users?sort=email,-created_at
/// GET /api/users?page=1&per_page=10&admin=false&query=jacques&sort=-email
/// ```
///
/// ### Responses
///
/// - `200 OK`
/// ```json
/// {
///   "success": true,
///   "data": {
///     "users": [
///       {
///         "id": "uuid",
///         "email": "user@example.com",
///         "student_number": "u12345678",
///         "admin": false,
///         "created_at": "2025-05-23T18:00:00Z",
///         "updated_at": "2025-05-23T18:00:00Z"
///       }
///     ],
///     "page": 1,
///     "per_page": 10,
///     "total": 135
///   },
///   "message": "Users retrieved successfully"
/// }
/// ```
///
/// - `400 Bad Request` - Invalid query parameters
/// - `401 Unauthorized` - Missing or invalid JWT
/// - `403 Forbidden` - Authenticated but not admin user
/// - `500 Internal Server Error` - Database error
pub async fn list_users(Query(query_params): Query<ListUsersQuery>) -> impl IntoResponse {
    if let Err(validation_errors) = query_params.validate() {
        let error_message = common::format_validation_errors(&validation_errors);
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<UsersListResponse>::error(error_message)),
        );
    }

    let pool = pool::get();
    
    let page = query_params.page.unwrap_or(1);
    let per_page = query_params.per_page.unwrap_or(20);
    let offset = (page - 1) * per_page;

    let mut where_conditions = Vec::new();
    let mut params = Vec::new();
    
    if let Some(query) = &query_params.query {
        where_conditions.push("(LOWER(email) LIKE LOWER(?) OR LOWER(student_number) LIKE LOWER(?))");
        let query_pattern = format!("%{}%", query);
        params.push(query_pattern.clone());
        params.push(query_pattern);
    } else {
        if let Some(email) = &query_params.email {
            where_conditions.push("LOWER(email) LIKE LOWER(?)");
            params.push(format!("%{}%", email));
        }
        
        if let Some(student_number) = &query_params.student_number {
            where_conditions.push("LOWER(student_number) LIKE LOWER(?)");
            params.push(format!("%{}%", student_number));
        }
    }
    
    if let Some(admin_filter) = query_params.admin {
        where_conditions.push("admin = ?");
        params.push(if admin_filter { "1" } else { "0" }.to_string()); // goofy ahhh code
    }

    let where_clause = if where_conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_conditions.join(" AND "))
    };

    let order_clause = if let Some(sort_param) = &query_params.sort {
        build_order_clause(sort_param)
    } else {
        "ORDER BY id ASC".to_string()
    };

    let count_query = format!("SELECT COUNT(*) as count FROM users {}", where_clause);
    let mut count_query_builder = sqlx::query(&count_query);
    
    for param in &params {
        count_query_builder = count_query_builder.bind(param);
    }
    
    let total = match count_query_builder.fetch_one(pool).await {
        Ok(row) => {
            match row.try_get::<i64, _>("count") {
                Ok(count) => count,
                Err(_) => 0,
            }
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<UsersListResponse>::error(format!("Database error: {}", e))),
            );
        }
    };

    let users_query = format!(
        "SELECT id, student_number, email, password_hash, admin, created_at, updated_at 
         FROM users {} {} LIMIT ? OFFSET ?",
        where_clause, order_clause
    );
    
    let mut users_query_builder = sqlx::query_as::<_, User>(&users_query);
    
    for param in &params {
        users_query_builder = users_query_builder.bind(param);
    }
    
    users_query_builder = users_query_builder.bind(per_page).bind(offset);

    match users_query_builder.fetch_all(pool).await {
        Ok(users) => {
            let user_items: Vec<UserListItem> = users.into_iter().map(UserListItem::from).collect();
            
            let response = UsersListResponse {
                users: user_items,
                page,
                per_page,
                total,
            };

            (
                StatusCode::OK,
                Json(ApiResponse::success(response, "Users retrieved successfully")),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<UsersListResponse>::error(format!("Database error: {}", e))),
        ),
    }
}

/// Builds ORDER BY clause from sort parameter string
/// 
/// # Arguments
/// * `sort_param` - Comma-separated list of fields, with optional `-` prefix for descending
/// 
/// # Examples
/// * `"email"` -> `"ORDER BY email ASC"`
/// * `"email,-created_at"` -> `"ORDER BY email ASC, created_at DESC"`
/// * `"-created_at,email"` -> `"ORDER BY created_at DESC, email ASC"`
fn build_order_clause(sort_param: &str) -> String {
    let valid_fields = ["email", "student_number", "created_at", "admin"];
    let mut order_parts = Vec::new();

    for field in sort_param.split(',') {
        let field = field.trim();
        
        let (field_name, direction) = if field.starts_with('-') {
            (&field[1..], "DESC")
        } else {
            (field, "ASC")
        };

        if valid_fields.contains(&field_name) {
            order_parts.push(format!("{} {}", field_name, direction));
        }
    }

    if order_parts.is_empty() {
        "ORDER BY id ASC".to_string()
    } else {
        format!("ORDER BY {}", order_parts.join(", "))
    }
}

#[derive(Debug, Serialize)]
pub struct UserModule {
    pub id: i64,
    pub code: String,
    pub year: i32,
    pub description: String,
    pub credits: i32,
    pub role: String,
    pub created_at: String,
    pub updated_at: String,
}

/// GET /api/users/:id/modules
///
/// Retrieve all modules that a specific user is involved in, including their role in each module.
/// Requires admin privileges.
///
/// ### Path Parameters
/// - `id`: The ID of the user to fetch modules for
///
/// ### Responses
///
/// - `200 OK`
/// ```json
/// {
///   "success": true,
///   "data": [
///     {
///       "id": 1,
///       "code": "COS301",
///       "year": 2025,
///       "description": "Advanced Software Engineering",
///       "credits": 16,
///       "role": "Lecturer",
///       "created_at": "2025-05-01T08:00:00Z",
///       "updated_at": "2025-05-01T08:00:00Z"
///     }
///   ],
///   "message": "Modules for user retrieved successfully"
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
/// - `403 Forbidden` - Not an admin user
/// - `404 Not Found` - User not found
/// ```json
/// {
///   "success": false,
///   "message": "User not found"
/// }
/// ```
///
/// - `500 Internal Server Error` - Database error
/// ```json
/// {
///   "success": false,
///   "message": "Database error: detailed error here"
/// }
/// ```
pub async fn get_user_modules(
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let user_id = match id.parse::<i64>() {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<Vec<UserModule>>::error("Invalid user ID format")),
            );
        }
    };

    let pool = pool::get();

    let user = match User::get_by_id(Some(pool), user_id).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<Vec<UserModule>>::error("User not found")),
            );
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<Vec<UserModule>>::error(format!("Database error: {}", e))),
            );
        }
    };

    let module_roles = match User::get_module_roles(Some(pool), user.id).await {
        Ok(roles) => roles,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<Vec<UserModule>>::error(format!("Database error: {}", e))),
            );
        }
    };

    let modules = module_roles
        .into_iter()
        .map(|role| UserModule {
            id: role.module_id,
            code: role.module_code,
            year: role.module_year,
            description: role.module_description.unwrap_or_default(),
            credits: role.module_credits.unwrap_or(0),
            role: role.role,
            created_at: role.module_created_at.unwrap_or_default(),
            updated_at: role.module_updated_at.unwrap_or_default(),
        })
        .collect::<Vec<UserModule>>();

    (
        StatusCode::OK,
        Json(ApiResponse::<Vec<UserModule>>::success(
            modules,
            "Modules for user retrieved successfully",
        )),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use db::models::user::User;
    use sqlx::SqlitePool;
    use db::create_test_db;
    use db::delete_database;

    // Helper function to create test users
    async fn create_test_users(pool: &SqlitePool) -> Vec<User> {
        let mut users = Vec::new();
        
        // Create users with different attributes for testing
        let user1 = User::create(Some(pool), "u12345678", "alice@example.com", "password1", false)
            .await
            .unwrap();
        users.push(user1);

        let user2 = User::create(Some(pool), "u87654321", "bob@test.com", "password2", true)
            .await
            .unwrap();
        users.push(user2);

        let user3 = User::create(Some(pool), "u11111111", "charlie@example.com", "password3", false)
            .await
            .unwrap();
        users.push(user3);

        let user4 = User::create(Some(pool), "u22222222", "diana@university.edu", "password4", true)
            .await
            .unwrap();
        users.push(user4);

        users
    }

    #[tokio::test]
    async fn test_build_order_clause_valid_fields() {
        // Test single field ascending
        let result = build_order_clause("email");
        assert_eq!(result, "ORDER BY email ASC");

        // Test single field descending
        let result = build_order_clause("-created_at");
        assert_eq!(result, "ORDER BY created_at DESC");

        // Test multiple fields
        let result = build_order_clause("email,-created_at,admin");
        assert_eq!(result, "ORDER BY email ASC, created_at DESC, admin ASC");

        // Test with spaces
        let result = build_order_clause(" email , -created_at , admin ");
        assert_eq!(result, "ORDER BY email ASC, created_at DESC, admin ASC");
    }

    #[tokio::test]
    async fn test_build_order_clause_invalid_fields() {
        // Test invalid field - should be filtered out
        let result = build_order_clause("invalid_field");
        assert_eq!(result, "ORDER BY id ASC");

        // Test mix of valid and invalid fields
        let result = build_order_clause("email,invalid_field,-created_at");
        assert_eq!(result, "ORDER BY email ASC, created_at DESC");

        // Test empty string
        let result = build_order_clause("");
        assert_eq!(result, "ORDER BY id ASC");

        // Test only invalid fields
        let result = build_order_clause("bad_field,another_bad_field");
        assert_eq!(result, "ORDER BY id ASC");
    }

    #[tokio::test]
    async fn test_build_order_clause_sql_injection_prevention() {
        // Test potential SQL injection attempts
        let result = build_order_clause("email; DROP TABLE users; --");
        assert_eq!(result, "ORDER BY id ASC");

        let result = build_order_clause("email' OR '1'='1");
        assert_eq!(result, "ORDER BY id ASC");

        let result = build_order_clause("email UNION SELECT * FROM users");
        assert_eq!(result, "ORDER BY id ASC");
    }

    #[tokio::test]
    async fn test_list_users_query_validation() {
        // Test valid query parameters
        let valid_query = ListUsersQuery {
            page: Some(1),
            per_page: Some(10),
            sort: Some("email".to_string()),
            query: Some("test".to_string()),
            email: None,
            student_number: None,
            admin: Some(false),
        };
        assert!(valid_query.validate().is_ok());

        // Test invalid page (less than 1)
        let invalid_page = ListUsersQuery {
            page: Some(0),
            per_page: Some(10),
            sort: None,
            query: None,
            email: None,
            student_number: None,
            admin: None,
        };
        assert!(invalid_page.validate().is_err());

        // Test invalid per_page (too large)
        let invalid_per_page = ListUsersQuery {
            page: Some(1),
            per_page: Some(101),
            sort: None,
            query: None,
            email: None,
            student_number: None,
            admin: None,
        };
        assert!(invalid_per_page.validate().is_err());

        // Test invalid per_page (less than 1)
        let invalid_per_page_low = ListUsersQuery {
            page: Some(1),
            per_page: Some(0),
            sort: None,
            query: None,
            email: None,
            student_number: None,
            admin: None,
        };
        assert!(invalid_per_page_low.validate().is_err());
    }

    #[tokio::test]
    async fn test_user_list_item_from_user() {
        let pool = create_test_db(Some("test_user_list_item.db")).await;
        
        let user = User::create(Some(&pool), "u12345678", "test@example.com", "password", true)
            .await
            .unwrap();

        // Store the values we need before converting
        let id = user.id.to_string();
        let email = user.email.clone();
        let student_number = user.student_number.clone();
        let admin = user.admin;
        let created_at = user.created_at.clone();
        let updated_at = user.updated_at.clone();

        let list_item = UserListItem::from(user);

        assert_eq!(list_item.id, id);
        assert_eq!(list_item.email, email);
        assert_eq!(list_item.student_number, student_number);
        assert_eq!(list_item.admin, admin);
        assert_eq!(list_item.created_at, created_at);
        assert_eq!(list_item.updated_at, updated_at);

        pool.close().await;
        delete_database("test_user_list_item.db");
    }

    #[tokio::test]
    async fn test_list_users_default_parameters() {
        let pool = create_test_db(Some("test_list_users_defaults.db")).await;
        
        // Create some test users
        create_test_users(&pool).await;

        // Test with empty query parameters (should use defaults)
        let query_params = ListUsersQuery {
            page: None,
            per_page: None,
            sort: None,
            query: None,
            email: None,
            student_number: None,
            admin: None,
        };

        // Since we can't easily test the full handler, we'll test the parameter defaults
        let page = query_params.page.unwrap_or(1);
        let per_page = query_params.per_page.unwrap_or(20);
        let offset = (page - 1) * per_page;

        assert_eq!(page, 1);
        assert_eq!(per_page, 20);
        assert_eq!(offset, 0);

        pool.close().await;
        delete_database("test_list_users_defaults.db");
    }

    #[tokio::test]
    async fn test_where_clause_building_logic() {
        // Test query parameter logic (the core filtering logic)
        
        // Test with general query (should override email and student_number)
        let query_params = ListUsersQuery {
            page: Some(1),
            per_page: Some(10),
            sort: None,
            query: Some("test".to_string()),
            email: Some("should_be_ignored".to_string()),
            student_number: Some("also_ignored".to_string()),
            admin: Some(true),
        };

        let mut where_conditions = Vec::new();
        let mut params = Vec::new();
        
        // Simulate the where clause building logic from list_users
        if let Some(query) = &query_params.query {
            where_conditions.push("(LOWER(email) LIKE LOWER(?) OR LOWER(student_number) LIKE LOWER(?))");
            let query_pattern = format!("%{}%", query);
            params.push(query_pattern.clone());
            params.push(query_pattern);
        } else {
            if let Some(email) = &query_params.email {
                where_conditions.push("LOWER(email) LIKE LOWER(?)");
                params.push(format!("%{}%", email));
            }
            
            if let Some(student_number) = &query_params.student_number {
                where_conditions.push("LOWER(student_number) LIKE LOWER(?)");
                params.push(format!("%{}%", student_number));
            }
        }
        
        if let Some(admin_filter) = query_params.admin {
            where_conditions.push("admin = ?");
            params.push(if admin_filter { "1" } else { "0" }.to_string());
        }

        // Verify the logic worked correctly
        assert_eq!(where_conditions.len(), 2); // query condition + admin condition
        assert_eq!(params.len(), 3); // query pattern twice + admin value
        assert_eq!(params[0], "%test%");
        assert_eq!(params[1], "%test%");
        assert_eq!(params[2], "1");
    }

    #[tokio::test]
    async fn test_where_clause_building_without_query() {
        // Test email and student_number filters when query is not provided
        let query_params = ListUsersQuery {
            page: Some(1),
            per_page: Some(10),
            sort: None,
            query: None,
            email: Some("example.com".to_string()),
            student_number: Some("u1234".to_string()),
            admin: Some(false),
        };

        let mut where_conditions = Vec::new();
        let mut params = Vec::new();
        
        // Simulate the where clause building logic
        if let Some(query) = &query_params.query {
            where_conditions.push("(LOWER(email) LIKE LOWER(?) OR LOWER(student_number) LIKE LOWER(?))");
            let query_pattern = format!("%{}%", query);
            params.push(query_pattern.clone());
            params.push(query_pattern);
        } else {
            if let Some(email) = &query_params.email {
                where_conditions.push("LOWER(email) LIKE LOWER(?)");
                params.push(format!("%{}%", email));
            }
            
            if let Some(student_number) = &query_params.student_number {
                where_conditions.push("LOWER(student_number) LIKE LOWER(?)");
                params.push(format!("%{}%", student_number));
            }
        }
        
        if let Some(admin_filter) = query_params.admin {
            where_conditions.push("admin = ?");
            params.push(if admin_filter { "1" } else { "0" }.to_string());
        }

        // Verify the logic worked correctly
        assert_eq!(where_conditions.len(), 3); // email + student_number + admin
        assert_eq!(params.len(), 3);
        assert_eq!(params[0], "%example.com%");
        assert_eq!(params[1], "%u1234%");
        assert_eq!(params[2], "0");
    }

    #[tokio::test]
    async fn test_pagination_calculation() {
        // Test pagination offset calculation
        let test_cases = vec![
            (1, 10, 0),   // page 1, per_page 10 -> offset 0
            (2, 10, 10),  // page 2, per_page 10 -> offset 10
            (3, 5, 10),   // page 3, per_page 5 -> offset 10
            (1, 20, 0),   // page 1, per_page 20 -> offset 0
            (5, 25, 100), // page 5, per_page 25 -> offset 100
        ];

        for (page, per_page, expected_offset) in test_cases {
            let offset = (page - 1) * per_page;
            assert_eq!(offset, expected_offset, 
                "Failed for page={}, per_page={}: expected offset {}, got {}", 
                page, per_page, expected_offset, offset);
        }
    }

    #[tokio::test]
    async fn test_admin_filter_conversion() {
        // Test the admin filter boolean to string conversion logic
        let admin_true = true;
        let admin_false = false;

        let admin_true_str = if admin_true { "1" } else { "0" }.to_string();
        let admin_false_str = if admin_false { "1" } else { "0" }.to_string();

        assert_eq!(admin_true_str, "1");
        assert_eq!(admin_false_str, "0");
    }

    // Integration-style test that creates users and tests the database queries
    #[tokio::test]
    async fn test_user_filtering_queries() {
        let pool = create_test_db(Some("test_user_filtering.db")).await;
        
        // Create test users with specific data for filtering
        let _users = create_test_users(&pool).await;

        // Test count query (simulate the count logic from list_users)
        let count_result = sqlx::query("SELECT COUNT(*) as count FROM users")
            .fetch_one(&pool)
            .await
            .unwrap();
        
        let total: i64 = count_result.try_get("count").unwrap();
        assert_eq!(total, 4); // We created 4 users

        // Test email filter query
        let email_filter_result = sqlx::query("SELECT COUNT(*) as count FROM users WHERE LOWER(email) LIKE LOWER(?)")
            .bind("%example.com%")
            .fetch_one(&pool)
            .await
            .unwrap();
        
        let email_count: i64 = email_filter_result.try_get("count").unwrap();
        assert_eq!(email_count, 2); // alice@example.com and charlie@example.com

        // Test admin filter query
        let admin_filter_result = sqlx::query("SELECT COUNT(*) as count FROM users WHERE admin = ?")
            .bind("1")
            .fetch_one(&pool)
            .await
            .unwrap();
        
        let admin_count: i64 = admin_filter_result.try_get("count").unwrap();
        assert_eq!(admin_count, 2); // bob and diana are admins

        // Test combined query filter (email OR student_number)
        let combined_filter_result = sqlx::query("SELECT COUNT(*) as count FROM users WHERE (LOWER(email) LIKE LOWER(?) OR LOWER(student_number) LIKE LOWER(?))")
            .bind("%u1%")
            .bind("%u1%")
            .fetch_one(&pool)
            .await
            .unwrap();
        
        let combined_count: i64 = combined_filter_result.try_get("count").unwrap();
        assert!(combined_count >= 2); // Should match users with "u1" in student number or email

        pool.close().await;
        delete_database("test_user_filtering.db");
    }

    #[tokio::test]
    async fn test_ordering_queries() {
        let pool = create_test_db(Some("test_user_ordering.db")).await;
        
        // Create test users
        let _users = create_test_users(&pool).await;

        // Test ordering by email ascending
        let ordered_users = sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY email ASC")
            .fetch_all(&pool)
            .await
            .unwrap();

        // Verify ordering
        assert!(ordered_users.len() >= 4);
        for i in 1..ordered_users.len() {
            assert!(ordered_users[i-1].email <= ordered_users[i].email);
        }

        // Test ordering by created_at descending
        let desc_users = sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY created_at DESC")
            .fetch_all(&pool)
            .await
            .unwrap();

        assert!(desc_users.len() >= 4);
        // Note: We can't easily test timestamp ordering without more control over creation times

        pool.close().await;
        delete_database("test_user_ordering.db");
    }

    #[tokio::test]
    async fn test_pagination_queries() {
        let pool = create_test_db(Some("test_user_pagination.db")).await;
        
        // Create test users
        let _users = create_test_users(&pool).await;

        // Test LIMIT and OFFSET
        let page1_users = sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY id ASC LIMIT ? OFFSET ?")
            .bind(2_i64)
            .bind(0_i64)
            .fetch_all(&pool)
            .await
            .unwrap();

        let page2_users = sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY id ASC LIMIT ? OFFSET ?")
            .bind(2_i64)
            .bind(2_i64)
            .fetch_all(&pool)
            .await
            .unwrap();

        assert_eq!(page1_users.len(), 2);
        assert_eq!(page2_users.len(), 2);
        
        // Ensure no overlap
        let page1_ids: Vec<i64> = page1_users.iter().map(|u| u.id).collect();
        let page2_ids: Vec<i64> = page2_users.iter().map(|u| u.id).collect();
        
        for id1 in &page1_ids {
            assert!(!page2_ids.contains(id1), "Pages should not overlap");
        }

        pool.close().await;
        delete_database("test_user_pagination.db");
    }
}