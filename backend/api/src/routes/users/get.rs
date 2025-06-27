use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use sea_orm::{
    ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
};

use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::response::ApiResponse;

use db::{
    connect,
    models::user::{Entity as UserEntity, Model as UserModel, Column as UserColumn},
};

#[derive(Debug, Deserialize, Validate)]
pub struct ListUsersQuery {
    #[validate(range(min = 1))]
    pub page: Option<u64>,
    #[validate(range(min = 1, max = 100))]
    pub per_page: Option<u64>,
    pub sort: Option<String>,
    pub query: Option<String>,
    pub email: Option<String>,
    pub username: Option<String>,
    pub admin: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct UserListItem {
    pub id: String,
    pub email: String,
    pub username: String,
    pub admin: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct UsersListResponse {
    pub users: Vec<UserListItem>,
    pub page: u64,
    pub per_page: u64,
    pub total: u64,
}

impl From<UserModel> for UserListItem {
    fn from(user: UserModel) -> Self {
        Self {
            id: user.id.to_string(),
            email: user.email,
            username: user.username,
            admin: user.admin,
            created_at: user.created_at.to_string(),
            updated_at: user.updated_at.to_string(),
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
/// - `query` (optional): Case-insensitive partial match against email OR username
/// - `email` (optional): Case-insensitive partial match on email
/// - `username` (optional): Case-insensitive partial match on student number
/// - `admin` (optional): Filter by admin status (true/false)
/// - `sort` (optional): Comma-separated sort fields. Use `-` prefix for descending
///
/// ### Examples
/// ```http
/// GET /api/users?page=2&per_page=10
/// GET /api/users?query=u1234
/// GET /api/users?email=@example.com
/// GET /api/users?username=u1234
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
///         "username": "u12345678",
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
pub async fn list_users(Query(query): Query<ListUsersQuery>) -> impl IntoResponse {
    if let Err(e) = query.validate() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<UsersListResponse>::error(
                common::format_validation_errors(&e),
            )),
        );
    }

    let db = connect().await;
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);

    let mut condition = Condition::all();

    if let Some(q) = &query.query {
        let pattern = format!("%{}%", q.to_lowercase());
        condition = condition.add(
            Condition::any()
                .add(UserColumn::Email.contains(&pattern))
                .add(UserColumn::Username.contains(&pattern)),
        );
    }

    if let Some(email) = &query.email {
        condition = condition.add(UserColumn::Email.contains(&format!("%{}%", email)));
    }

    if let Some(sn) = &query.username {
        condition = condition.add(UserColumn::Username.contains(&format!("%{}%", sn)));
    }

    if let Some(admin) = query.admin {
        condition = condition.add(UserColumn::Admin.eq(admin));
    }

    let mut query_builder = UserEntity::find().filter(condition);

    if let Some(sort_param) = &query.sort {
        for sort_field in sort_param.split(',') {
            let (field, desc) = if sort_field.starts_with('-') {
                (&sort_field[1..], true)
            } else {
                (sort_field, false)
            };

            match field {
                "email" => {
                    query_builder = if desc {
                        query_builder.order_by_desc(UserColumn::Email)
                    } else {
                        query_builder.order_by_asc(UserColumn::Email)
                    };
                }
                "username" => {
                    query_builder = if desc {
                        query_builder.order_by_desc(UserColumn::Username)
                    } else {
                        query_builder.order_by_asc(UserColumn::Username)
                    };
                }
                "created_at" => {
                    query_builder = if desc {
                        query_builder.order_by_desc(UserColumn::CreatedAt)
                    } else {
                        query_builder.order_by_asc(UserColumn::CreatedAt)
                    };
                }
                "admin" => {
                    query_builder = if desc {
                        query_builder.order_by_desc(UserColumn::Admin)
                    } else {
                        query_builder.order_by_asc(UserColumn::Admin)
                    };
                }
                _ => {}
            }
        }
    } else {
        query_builder = query_builder.order_by_asc(UserColumn::Id);
    }

    let paginator = query_builder.paginate(&db, per_page);
    let total = paginator.num_items().await.unwrap_or(0);
    let users = paginator.fetch_page(page - 1).await.unwrap_or_default();
    let users = users.into_iter().map(UserListItem::from).collect();

    (
        StatusCode::OK,
        Json(ApiResponse::success(
            UsersListResponse {
                users,
                page,
                per_page,
                total,
            },
            "Users retrieved successfully",
        )),
    )
}

/// GET /api/users/:id
///
/// Fetch a single user by ID. Requires admin privileges.
///
/// ### Path Parameters
/// - `id`: The user ID (integer)
///
/// ### Responses
/// - `200 OK`: User found
/// - `400 Bad Request`: Invalid ID format
/// - `404 Not Found`: User does not exist
/// - `500 Internal Server Error`: DB error
pub async fn get_user(Path(id): Path<String>) -> impl IntoResponse {
    let user_id: i32 = match id.parse() {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<UserListItem>::error("Invalid user ID format")),
            );
        }
    };

    let db = connect().await;

    match UserEntity::find_by_id(user_id).one(&db).await {
        Ok(Some(user)) => {
            let user_item = UserListItem::from(user);
            (
                StatusCode::OK,
                Json(ApiResponse::success(user_item, "User retrieved successfully")),
            )
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<UserListItem>::error("User not found")),
        ),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<UserListItem>::error(format!("Database error: {}", err))),
        ),
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
pub async fn get_user_modules(Path(id): Path<String>) -> impl IntoResponse {
    let user_id: i64 = match id.parse() {
        Ok(val) => val,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<Vec<UserModule>>::error("Invalid user ID format")),
            );
        }
    };

    let db = connect().await;

    let user = match UserEntity::find_by_id(user_id).one(&db).await {
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

    let roles = match UserModel::get_module_roles(&db, user.id).await {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<Vec<UserModule>>::error(format!("Database error: {}", e))),
            );
        }
    };

    let modules = roles
        .into_iter()
        .map(|r| UserModule {
            id: r.module_id,
            code: r.module_code,
            year: r.module_year,
            description: r.module_description.unwrap_or_default(),
            credits: r.module_credits,
            role: r.role,
            created_at: r.module_created_at,
            updated_at: r.module_updated_at,
        })
        .collect::<Vec<_>>();

    (
        StatusCode::OK,
        Json(ApiResponse::success(
            modules,
            "Modules for user retrieved successfully",
        )),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use db::models::user::Model as UserModel;
    use sea_orm::DatabaseConnection;

    // Helper to insert multiple test users into SeaORM DB
    async fn create_test_users(db: &DatabaseConnection) -> Vec<UserModel> {
        let mut users = Vec::new();

        let user1 = UserModel::create(db, "u12345678", "alice@example.com", "password1", false)
            .await
            .unwrap();
        users.push(user1);

        let user2 = UserModel::create(db, "u87654321", "bob@test.com", "password2", true)
            .await
            .unwrap();
        users.push(user2);

        let user3 = UserModel::create(db, "u11111111", "charlie@example.com", "password3", false)
            .await
            .unwrap();
        users.push(user3);

        let user4 = UserModel::create(db, "u22222222", "diana@university.edu", "password4", true)
            .await
            .unwrap();
        users.push(user4);

        users
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
            username: None,
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
            username: None,
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
            username: None,
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
            username: None,
            admin: None,
        };
        assert!(invalid_per_page_low.validate().is_err());
    }

    #[tokio::test]
    async fn test_user_list_item_from_user() {
        use db::test_utils::setup_test_db;
        use db::models::user::Model as UserModel;

        let db = setup_test_db().await;

        let user = UserModel::create(&db, "u12345678", "test@example.com", "password", true)
            .await
            .unwrap();

        // Store the values we need before converting
        let id = user.id.to_string();
        let email = user.email.clone();
        let username = user.username.clone();
        let admin = user.admin;
        let created_at = user.created_at.to_string();
        let updated_at = user.updated_at.to_string();

        let list_item = UserListItem::from(user);

        assert_eq!(list_item.id, id);
        assert_eq!(list_item.email, email);
        assert_eq!(list_item.username, username);
        assert_eq!(list_item.admin, admin);
        assert_eq!(list_item.created_at, created_at);
        assert_eq!(list_item.updated_at, updated_at);
    }

    #[tokio::test]
    async fn test_list_users_default_parameters() {
        use db::test_utils::setup_test_db;

        let db = setup_test_db().await;

        // Create some test users
        create_test_users(&db).await;

        // Simulate an empty query (should use defaults)
        let query_params = ListUsersQuery {
            page: None,
            per_page: None,
            sort: None,
            query: None,
            email: None,
            username: None,
            admin: None,
        };

        let page = query_params.page.unwrap_or(1);
        let per_page = query_params.per_page.unwrap_or(20);
        let offset = (page - 1) * per_page;

        assert_eq!(page, 1);
        assert_eq!(per_page, 20);
        assert_eq!(offset, 0);
    }

    #[tokio::test]
    async fn test_where_clause_building_logic() {
        let query_params = ListUsersQuery {
            page: Some(1),
            per_page: Some(10),
            sort: None,
            query: Some("test".to_string()),
            email: Some("should_be_ignored".to_string()),
            username: Some("also_ignored".to_string()),
            admin: Some(true),
        };

        let mut where_conditions = Vec::new();
        let mut params = Vec::new();

        // Simulate the where clause building logic from list_users
        if let Some(query) = &query_params.query {
            where_conditions.push("(LOWER(email) LIKE LOWER(?) OR LOWER(username) LIKE LOWER(?))");
            let query_pattern = format!("%{}%", query);
            params.push(query_pattern.clone());
            params.push(query_pattern);
        } else {
            if let Some(email) = &query_params.email {
                where_conditions.push("LOWER(email) LIKE LOWER(?)");
                params.push(format!("%{}%", email));
            }

            if let Some(username) = &query_params.username {
                where_conditions.push("LOWER(username) LIKE LOWER(?)");
                params.push(format!("%{}%", username));
            }
        }

        if let Some(admin_filter) = query_params.admin {
            where_conditions.push("admin = ?");
            params.push(if admin_filter { "1" } else { "0" }.to_string());
        }

        assert_eq!(where_conditions.len(), 2);
        assert_eq!(params.len(), 3);
        assert_eq!(params[0], "%test%");
        assert_eq!(params[1], "%test%");
        assert_eq!(params[2], "1");
    }


    #[tokio::test]
    async fn test_where_clause_building_without_query() {
        // Test email and username filters when query is not provided
        let query_params = ListUsersQuery {
            page: Some(1),
            per_page: Some(10),
            sort: None,
            query: None,
            email: Some("example.com".to_string()),
            username: Some("u1234".to_string()),
            admin: Some(false),
        };

        let mut where_conditions = Vec::new();
        let mut params = Vec::new();
        
        // Simulate the where clause building logic
        if let Some(query) = &query_params.query {
            where_conditions.push("(LOWER(email) LIKE LOWER(?) OR LOWER(username) LIKE LOWER(?))");
            let query_pattern = format!("%{}%", query);
            params.push(query_pattern.clone());
            params.push(query_pattern);
        } else {
            if let Some(email) = &query_params.email {
                where_conditions.push("LOWER(email) LIKE LOWER(?)");
                params.push(format!("%{}%", email));
            }
            
            if let Some(username) = &query_params.username {
                where_conditions.push("LOWER(username) LIKE LOWER(?)");
                params.push(format!("%{}%", username));
            }
        }
        
        if let Some(admin_filter) = query_params.admin {
            where_conditions.push("admin = ?");
            params.push(if admin_filter { "1" } else { "0" }.to_string());
        }

        // Verify the logic worked correctly
        assert_eq!(where_conditions.len(), 3); // email + username + admin
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
        use db::{test_utils::setup_test_db, models::user::Entity as UserEntity};
        use sea_orm::{
            ColumnTrait, EntityTrait, QueryFilter, Condition,
            sea_query::Expr, DatabaseConnection
        };

        // Fully qualified column type alias to disambiguate `Column`
        type UserColumn = <UserEntity as EntityTrait>::Column;

        let db: DatabaseConnection = setup_test_db().await;

        // Seed 4 users
        create_test_users(&db).await;

        // Count total users
        let total = UserEntity::find().count(&db).await.unwrap();
        assert_eq!(total, 4);

        // Count users with email like '%example.com%'
        let email_like = UserEntity::find()
            .filter(Expr::col(UserColumn::Email).like("%example.com%"))
            .count(&db)
            .await
            .unwrap();
        assert_eq!(email_like, 2); // alice@example.com and charlie@example.com

        // Count users where admin = true
        let admin_count = UserEntity::find()
            .filter(UserColumn::Admin.eq(true))
            .count(&db)
            .await
            .unwrap();
        assert_eq!(admin_count, 2); // bob and diana

        // Count users where username or email contains 'u1'
        let combined = UserEntity::find()
            .filter(
                Condition::any()
                    .add(Expr::col(UserColumn::Email).like("%u1%"))
                    .add(Expr::col(UserColumn::Username).like("%u1%")),
            )
            .count(&db)
            .await
            .unwrap();
        assert!(combined >= 2);
    }

    #[tokio::test]
    async fn test_ordering_queries() {
        use db::{test_utils::setup_test_db, models::user::Entity as UserEntity};
        use sea_orm::{EntityTrait, QueryOrder, DatabaseConnection};

        type UserColumn = <UserEntity as EntityTrait>::Column;

        let db: DatabaseConnection = setup_test_db().await;

        // Seed users
        create_test_users(&db).await;

        // Test ordering by email ascending
        let ordered_users = UserEntity::find()
            .order_by_asc(UserColumn::Email)
            .all(&db)
            .await
            .unwrap();

        assert!(ordered_users.len() >= 4);
        for i in 1..ordered_users.len() {
            assert!(ordered_users[i - 1].email <= ordered_users[i].email);
        }

        // Test ordering by created_at descending
        let desc_users = UserEntity::find()
            .order_by_desc(UserColumn::CreatedAt)
            .all(&db)
            .await
            .unwrap();

        assert!(desc_users.len() >= 4);
        // Optionally: Add stricter timestamp checks if needed
    }

    #[tokio::test]
    async fn test_pagination_queries() {
        use db::{test_utils::setup_test_db, models::user::Entity as UserEntity};
        use sea_orm::{EntityTrait, QueryOrder, PaginatorTrait, DatabaseConnection};

        type UserColumn = <UserEntity as EntityTrait>::Column;

        let db: DatabaseConnection = setup_test_db().await;

        // Seed users
        create_test_users(&db).await;

        // Page size
        let per_page = 2;

        // Page 1
        let page1 = UserEntity::find()
            .order_by_asc(UserColumn::Id)
            .paginate(&db, per_page)
            .fetch_page(0)
            .await
            .unwrap();

        // Page 2
        let page2 = UserEntity::find()
            .order_by_asc(UserColumn::Id)
            .paginate(&db, per_page)
            .fetch_page(1)
            .await
            .unwrap();

        assert_eq!(page1.len(), 2);
        assert_eq!(page2.len(), 2);

        let page1_ids: Vec<i64> = page1.iter().map(|u| u.id).collect();
        let page2_ids: Vec<i64> = page2.iter().map(|u| u.id).collect();

        for id in &page1_ids {
            assert!(!page2_ids.contains(id), "Pages should not overlap");
        }
    }
}