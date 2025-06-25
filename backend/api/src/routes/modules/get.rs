use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};

use serde::{Deserialize, Serialize};

use sea_orm::{
    ColumnTrait, Condition, DatabaseConnection, EntityTrait, JoinType, Order,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};

use crate::{
    auth::AuthUser,
    response::ApiResponse,
    routes::modules::post::PersonnelResponse,
};

use db::{
    connect,
    models::{
        module::{Column as ModuleCol, Entity as ModuleEntity, Model as Module},
        user::{self, Column as UserCol, Entity as UserEntity, Model as UserModel},
        user_module_role::{self, Column as RoleCol, Entity as RoleEntity, Role},
    },
};

#[derive(Debug, Serialize, Deserialize)]
pub struct ModuleResponse {
    pub id: i64,
    pub code: String,
    pub year: i32,
    pub description: Option<String>,
    pub credits: i32,
    pub created_at: String,
    pub updated_at: String,
    pub lecturers: Vec<UserResponse>,
    pub tutors: Vec<UserResponse>,
    pub students: Vec<UserResponse>,
}

impl From<db::models::module::Model> for ModuleResponse {
    fn from(m: db::models::module::Model) -> Self {
        Self {
            id: m.id,
            code: m.code,
            year: m.year,
            description: m.description,
            credits: m.credits,
            created_at: m.created_at.to_rfc3339(),
            updated_at: m.updated_at.to_rfc3339(),
            lecturers: vec![],
            tutors: vec![],
            students: vec![],
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub admin: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<db::models::user::Model> for UserResponse {
    fn from(user: db::models::user::Model) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            admin: user.admin,
            created_at: user.created_at.to_rfc3339(),
            updated_at: user.updated_at.to_rfc3339(),
        }
    }
}


#[derive(Debug, Deserialize)]
pub struct LecturerQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub query: Option<String>,
    pub email: Option<String>,
    pub username: Option<String>,
    pub sort: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LecturerListResponse {
    pub users: Vec<PersonnelResponse>,
    pub page: u32,
    pub per_page: u32,
    pub total: u64,
}

/// GET /api/modules/:module_id/lecturers
///
/// Retrieve a paginated, filtered, and sorted list of users assigned as lecturers to the specified module.
///
/// ### Access Control
/// This endpoint is accessible to:
/// - Admin users (`claims.admin == true`)
/// - Users assigned to the module (as Lecturer, Tutor, or Student)
///
/// ### Path Parameters
/// - `module_id` (integer): The ID of the module whose lecturers are being queried.
///
/// ### Query Parameters (All Optional)
/// - `page` (integer): Page number, default is `1`, must be ≥ 1.
/// - `per_page` (integer): Number of results per page, default is `20`, max is `100`.
/// - `query` (string): Fuzzy search term for `email` or `username` (case-insensitive).
/// - `email` (string): Filter by email (case-insensitive, ignored if `query` is present).
/// - `username` (string): Filter by student number (ignored if `query` is present).
/// - `sort` (string): Sort by field. Prefix with `-` for descending order. Allowed fields:
///   - `email`
///   - `username`
///   - `created_at`
///
/// ### Authentication
/// Requires a valid JWT with appropriate permissions. Returns `403` if the user is not an admin and not assigned to the module.
///
/// ### Example Requests
/// ```http
/// GET /api/modules/42/lecturers?page=2&per_page=10
/// GET /api/modules/42/lecturers?query=example
/// GET /api/modules/42/lecturers?email=@up.ac.za
/// GET /api/modules/42/lecturers?sort=-created_at
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
///         "id": 1,
///         "username": "u12345678",
///         "email": "lecturer@example.com",
///         "admin": false,
///         "created_at": "2025-05-23T18:00:00Z",
///         "updated_at": "2025-05-23T18:00:00Z"
///       }
///     ],
///     "page": 1,
///     "per_page": 20,
///     "total": 57
///   },
///   "message": "Lecturers retrieved successfully"
/// }
/// ```
///
/// - `403 Forbidden`  
/// ```json
/// {
///   "success": false,
///   "message": "You do not have permission to view this module's users"
/// }
/// ```
///
/// - `404 Not Found`  
/// ```json
/// {
///   "success": false,
///   "message": "Module not found"
/// }
/// ```
///
/// - `500 Internal Server Error`  
/// ```json
/// {
///   "success": false,
///   "message": "An internal server error occurred"
/// }
/// ```
pub async fn get_lecturers(
    Path(module_id): Path<i32>,
    Query(params): Query<LecturerQuery>,
) -> Response {
    let db: DatabaseConnection = connect().await;

    // Check module existence
    let module_exists = user_module_role::Entity::find()
        .filter(RoleCol::ModuleId.eq(module_id))
        .one(&db)
        .await;

    if let Ok(None) | Err(_) = module_exists {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Module not found")),
        )
            .into_response();
    }

    // Pagination
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).clamp(1, 100);

    // Build base filter condition
    let mut condition = Condition::all()
        .add(RoleCol::ModuleId.eq(module_id))
        .add(RoleCol::Role.eq(Role::Lecturer));

    // Search query
    if let Some(ref q) = params.query {
        let q_lower = q.to_lowercase();
        condition = condition.add(
            Condition::any()
                .add(user::Column::Email.contains(&q_lower))
                .add(user::Column::Username.contains(&q_lower)),
        );
    } else {
        if let Some(ref email) = params.email {
            condition = condition.add(user::Column::Email.contains(email));
        }
        if let Some(ref sn) = params.username {
            condition = condition.add(user::Column::Username.contains(sn));
        }
    }

    // Build query
    let mut query = user::Entity::find()
        .join(
            JoinType::InnerJoin,
            user::Entity::belongs_to(user_module_role::Entity)
                .from(user::Column::Id)
                .to(RoleCol::UserId)
                .into(),
        )
        .filter(condition.clone());

    // Sorting
    match params.sort.as_deref() {
        Some("-email") => query = query.order_by_desc(user::Column::Email),
        Some("email") => query = query.order_by_asc(user::Column::Email),
        Some("-username") => query = query.order_by_desc(user::Column::Username),
        Some("username") => query = query.order_by_asc(user::Column::Username),
        Some("-created_at") => query = query.order_by_desc(user::Column::CreatedAt),
        Some("created_at") => query = query.order_by_asc(user::Column::CreatedAt),
        _ => query = query.order_by_asc(user::Column::Id),
    }

    let paginator = query.paginate(&db, per_page.into());
    let total = paginator.num_items().await.unwrap_or(0);
    let users = paginator.fetch_page((page - 1).into()).await.unwrap_or_default();
    let result = users.into_iter().map(PersonnelResponse::from).collect();

    (
        StatusCode::OK,
        Json(ApiResponse::success(
            LecturerListResponse {
                users: result,
                page,
                per_page,
                total,
            },
            "Lecturers retrieved successfully",
        )),
    )
        .into_response()
}


#[derive(Debug, Deserialize)]
pub struct TutorQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub query: Option<String>,
    pub email: Option<String>,
    pub username: Option<String>,
    pub sort: Option<String>,
}

#[derive(serde::Serialize)]
pub struct PaginatedPersonnelResponse {
    pub users: Vec<PersonnelResponse>,
    pub page: u32,
    pub per_page: u32,
    pub total: u64,
}

/// GET /api/modules/:module_id/tutors
///
/// Retrieve a paginated, filtered, and sorted list of users assigned as tutors to the specified module.
///
/// ### Access Control
/// This endpoint is accessible to:
/// - Admin users (`claims.admin == true`)
/// - Users assigned to the module (as Lecturer, Tutor, or Student)
///
/// ### Path Parameters
/// - `module_id` (integer): The ID of the module whose tutors are being queried.
///
/// ### Query Parameters (All Optional)
/// - `page` (integer): Page number. Default is `1`. Must be ≥ 1.
/// - `per_page` (integer): Number of results per page. Default is `20`. Maximum is `100`.
/// - `query` (string): Fuzzy search term for `email` or `username` (case-insensitive).
/// - `email` (string): Filter by email (case-insensitive, ignored if `query` is present).
/// - `username` (string): Filter by student number (ignored if `query` is present).
/// - `sort` (string): Sort by field. Prefix with `-` for descending order. Allowed fields:
///   - `email`
///   - `username`
///   - `created_at`
///
/// ### Authentication
/// Requires a valid JWT. Returns `403 Forbidden` if the user is not an admin and not assigned to the module.
///
/// ### Example Requests
/// ```http
/// GET /api/modules/42/tutors?page=2&per_page=10
/// GET /api/modules/42/tutors?query=up.ac.za
/// GET /api/modules/42/tutors?email=tutor@example.com
/// GET /api/modules/42/tutors?sort=-created_at
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
///         "id": 7,
///         "username": "u22222222",
///         "email": "tutor@example.com",
///         "admin": false,
///         "created_at": "2025-05-23T18:00:00Z",
///         "updated_at": "2025-05-23T18:00:00Z"
///       }
///     ],
///     "page": 1,
///     "per_page": 20,
///     "total": 42
///   },
///   "message": "Tutors retrieved successfully"
/// }
/// ```
///
/// - `403 Forbidden`  
/// ```json
/// {
///   "success": false,
///   "message": "You do not have permission to view this module's users"
/// }
/// ```
///
/// - `404 Not Found`  
/// ```json
/// {
///   "success": false,
///   "message": "Module not found"
/// }
/// ```
///
/// - `500 Internal Server Error`  
/// ```json
/// {
///   "success": false,
///   "message": "An internal server error occurred"
/// }
/// ```
pub async fn get_tutors(
    Path(module_id): Path<i32>,
    Query(params): Query<TutorQuery>,
) -> Response {
    let db = connect().await;

    // Check if module exists
    let exists = db::models::module::Entity::find_by_id(module_id)
        .one(&db)
        .await
        .unwrap_or(None)
        .is_some();

    if !exists {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Module not found")),
        )
            .into_response();
    }

    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).clamp(1, 100);

    // Build filter condition
    let mut condition = Condition::all()
        .add(RoleCol::ModuleId.eq(module_id))
        .add(RoleCol::Role.eq(Role::Tutor));

    if let Some(ref q) = params.query {
        let pattern = format!("%{}%", q.to_lowercase());
        condition = condition.add(
            Condition::any()
                .add(user::Column::Email.contains(&pattern))
                .add(user::Column::Username.contains(&pattern)),
        );
    } else {
        if let Some(ref email) = params.email {
            condition = condition.add(user::Column::Email.contains(email));
        }
        if let Some(ref sn) = params.username {
            condition = condition.add(user::Column::Username.contains(sn));
        }
    }

    // Build query
    let mut query = user::Entity::find()
        .join(
            JoinType::InnerJoin,
            user::Entity::belongs_to(RoleEntity)
                .from(user::Column::Id)
                .to(RoleCol::UserId)
                .into(),
        )
        .filter(condition.clone());

    // Sorting
    if let Some(ref sort) = params.sort {
        let (field, dir) = if sort.starts_with('-') {
            (&sort[1..], sea_orm::Order::Desc)
        } else {
            (sort.as_str(), sea_orm::Order::Asc)
        };

        match field {
            "email" => query = query.order_by(user::Column::Email, dir),
            "username" => query = query.order_by(user::Column::Username, dir),
            "created_at" => query = query.order_by(user::Column::CreatedAt, dir),
            _ => {}
        }
    } else {
        query = query.order_by(user::Column::Id, sea_orm::Order::Asc);
    }

    // Pagination
    let paginator = query.paginate(&db, per_page.into());
    let total = paginator.num_items().await.unwrap_or(0);
    let users = paginator.fetch_page((page - 1).into()).await.unwrap_or_default();

    let result = users.into_iter().map(PersonnelResponse::from).collect();

    (
        StatusCode::OK,
        Json(ApiResponse::success(
            PaginatedPersonnelResponse {
                users: result,
                page,
                per_page,
                total,
            },
            "Tutors retrieved successfully",
        )),
    )
        .into_response()
}

#[derive(Debug, Deserialize)]
pub struct StudentQuery {
    pub query: Option<String>,
    pub email: Option<String>,
    pub username: Option<String>,
    pub sort: Option<String>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

/// GET /api/modules/:module_id/students
///
/// Retrieve a paginated, filtered, and sortable list of users enrolled as students in the specified module.
///
/// ### Access Control
/// This endpoint is accessible to:
/// - Admin users
///
/// ### Path Parameter
/// - `module_id` (integer): The ID of the module to retrieve students for.
///
/// ### Query Parameters
/// - `page` (optional): Page number (default: 1)
/// - `per_page` (optional): Items per page (default: 20, max: 100)
/// - `query` (optional): Case-insensitive partial match against email or student number
/// - `email` (optional): Partial match on email (used only if `query` is not provided)
/// - `username` (optional): Partial match on student number (used only if `query` is not provided)
/// - `sort` (optional): Sort by field. Prefix with `-` for descending. Allowed fields: `email`, `username`, `created_at`
///
/// ### Responses
/// - `200 OK`
/// ```json
/// {
///   "success": true,
///   "data": {
///     "users": [ { "id": 1, "email": "...", ... } ],
///     "page": 1,
///     "per_page": 20,
///     "total": 87
///   },
///   "message": "Students retrieved successfully"
/// }
/// ```
/// - `403 Forbidden` – if user is not admin or assigned to module
/// - `404 Not Found` – if the module does not exist
pub async fn get_students(
    Path(module_id): Path<i32>,
    Query(params): Query<StudentQuery>,
) -> Response {
    let db = connect().await;

    let module_exists = db::models::module::Entity::find_by_id(module_id)
        .one(&db)
        .await
        .unwrap_or(None)
        .is_some();

    if !module_exists {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Module not found")),
        )
            .into_response();
    }

    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).clamp(1, 100);

    // Build base filter
    let mut condition = Condition::all()
        .add(RoleCol::ModuleId.eq(module_id))
        .add(RoleCol::Role.eq(Role::Student));

    if let Some(ref q) = params.query {
        let pattern = format!("%{}%", q.to_lowercase());
        condition = condition.add(
            Condition::any()
                .add(user::Column::Email.contains(&pattern))
                .add(user::Column::Username.contains(&pattern)),
        );
    } else {
        if let Some(ref email) = params.email {
            condition = condition.add(user::Column::Email.contains(email));
        }
        if let Some(ref sn) = params.username {
            condition = condition.add(user::Column::Username.contains(sn));
        }
    }

    let mut query = user::Entity::find()
        .join(
            JoinType::InnerJoin,
            user::Entity::belongs_to(RoleEntity)
                .from(user::Column::Id)
                .to(RoleCol::UserId)
                .into(),
        )
        .filter(condition);

    // Sorting
    if let Some(ref sort) = params.sort {
        let (field, dir) = if sort.starts_with('-') {
            (&sort[1..], Order::Desc)
        } else {
            (sort.as_str(), Order::Asc)
        };

        match field {
            "email" => query = query.order_by(user::Column::Email, dir),
            "username" => query = query.order_by(user::Column::Username, dir),
            "created_at" => query = query.order_by(user::Column::CreatedAt, dir),
            _ => {}
        }
    } else {
        query = query.order_by(user::Column::Id, Order::Asc);
    }

    // Pagination
    let paginator = query.paginate(&db, per_page.into());
    let total = paginator.num_items().await.unwrap_or(0);
    let users = paginator.fetch_page((page - 1).into()).await.unwrap_or_default();

    let result = users.into_iter().map(PersonnelResponse::from).collect();

    (
        StatusCode::OK,
        Json(ApiResponse::success(
            PaginatedPersonnelResponse {
                users: result,
                page,
                per_page,
                total,
            },
            "Students retrieved successfully",
        )),
    )
        .into_response()
}

#[derive(Debug, Deserialize)]
pub struct EligibleUserQuery {
    pub role: String,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub sort: Option<String>,
    pub query: Option<String>,
    pub email: Option<String>,
    pub username: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct EligibleUserListResponse {
    pub users: Vec<db::models::user::Model>,
    pub page: u32,
    pub per_page: u32,
    pub total: u64,
}

pub async fn get_eligible_users_for_module(
    Path(module_id): Path<i64>,
    Query(params): Query<EligibleUserQuery>,
) -> Response {
    let db = connect().await;

    if !["Lecturer", "Tutor", "Student"].contains(&params.role.as_str()) {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<EligibleUserListResponse>::error("Invalid role")),
        )
            .into_response();
    }

    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).clamp(1, 100);

    // Find all user IDs already assigned to the module
    let assigned_ids: Vec<i32> = user_module_role::Entity::find()
        .select_only()
        .column(user_module_role::Column::UserId)
        .filter(user_module_role::Column::ModuleId.eq(module_id))
        .into_tuple::<i32>()
        .all(&db)
        .await
        .unwrap_or_default();

    let mut condition = Condition::all();

    if !assigned_ids.is_empty() {
    condition = condition.add(user::Column::Id.is_not_in(assigned_ids));
    }

    if let Some(ref q) = params.query {
        let pattern = format!("%{}%", q.to_lowercase());
        condition = condition.add(
            Condition::any()
                .add(user::Column::Email.contains(&pattern))
                .add(user::Column::Username.contains(&pattern)),
        );
    } else {
        if let Some(ref email) = params.email {
            condition = condition.add(user::Column::Email.contains(email));
        }
        if let Some(ref sn) = params.username {
            condition = condition.add(user::Column::Username.contains(sn));
        }
    }

    let mut query = user::Entity::find().filter(condition);

    if let Some(sort) = &params.sort {
        let (field, dir) = if sort.starts_with('-') {
            (&sort[1..], Order::Desc)
        } else {
            (sort.as_str(), Order::Asc)
        };

        match field {
            "email" => query = query.order_by(user::Column::Email, dir),
            "username" => query = query.order_by(user::Column::Username, dir),
            "created_at" => query = query.order_by(user::Column::CreatedAt, dir),
            _ => {}
        }
    } else {
        query = query.order_by(user::Column::Id, Order::Asc);
    }

    let paginator = query.paginate(&db, per_page.into());
    let total = paginator.num_items().await.unwrap_or(0);
    let users = paginator.fetch_page((page - 1).into()).await.unwrap_or_default();

    (
        StatusCode::OK,
        Json(ApiResponse::success(
            EligibleUserListResponse {
                users,
                page,
                per_page,
                total,
            },
            "Eligible users fetched",
        )),
    )
        .into_response()
}

/// Retrieves detailed information about a specific module, including assigned lecturers, tutors, and students.
///
/// # Arguments
///
/// The argument is extracted automatically from the HTTP route:
/// - Path parameter `module_id`: The ID of the module to retrieve.
///
/// # Returns
///
/// Returns an HTTP response indicating the result:
/// - `200 OK` with the full module details (including associated lecturers, tutors, and students) if successful.
/// - `404 NOT FOUND` if no module is found with the given `module_id`.
/// - `500 INTERNAL SERVER ERROR` if a database error occurs or if related personnel data (lecturers, tutors, or students) fails to load.
///
/// The response body is a JSON object using a standardized API response format, containing:
/// - Module information.
/// - Lists of users for each role (lecturers, tutors, students), each mapped to `UserResponse`.
pub async fn get_module(Path(module_id): Path<i32>) -> Response {
    let db: DatabaseConnection = connect().await;

    // Get the module by ID using match properly
    let module = match ModuleEntity::find_by_id(module_id).one(&db).await {
        Ok(Some(m)) => m,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<()>::error("Module not found")),
            )
                .into_response();
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("Database error retrieving module")),
            )
                .into_response();
        }
    };

    // Concurrently fetch users by role
    let (lecturers, tutors, students) = tokio::join!(
        get_users_by_role(&db, module_id, Role::Lecturer),
        get_users_by_role(&db, module_id, Role::Tutor),
        get_users_by_role(&db, module_id, Role::Student),
    );

    if lecturers.is_err() || tutors.is_err() || students.is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error("Failed to retrieve assigned personnel")),
        )
            .into_response();
    }

    let mut response = ModuleResponse::from(module);
    response.lecturers = lecturers.unwrap().into_iter().map(UserResponse::from).collect();
    response.tutors = tutors.unwrap().into_iter().map(UserResponse::from).collect();
    response.students = students.unwrap().into_iter().map(UserResponse::from).collect();

    (
        StatusCode::OK,
        Json(ApiResponse::success(response, "Module retrieved successfully")),
    )
        .into_response()
}


async fn get_users_by_role(
    db: &DatabaseConnection,
    module_id: i32,
    role: Role,
) -> Result<Vec<UserModel>, sea_orm::DbErr> {
    UserEntity::find()
        .join(
            JoinType::InnerJoin,
            UserEntity::belongs_to(RoleEntity)
                .from(UserCol::Id)
                .to(RoleCol::UserId)
                .into(),
        )
        .filter(
            Condition::all()
                .add(RoleCol::ModuleId.eq(module_id))
                .add(RoleCol::Role.eq(role)),
        )
        .all(db)
        .await
}

#[derive(Debug, Deserialize)]
pub struct FilterReq {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub query: Option<String>,
    pub code: Option<String>,
    pub year: Option<i32>,
    pub sort: Option<String>,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct ModuleDetailsResponse {
    pub id: i64,
    pub code: String,
    pub year: i32,
    pub description: Option<String>,
    pub credits: i32,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Module> for ModuleDetailsResponse {
    fn from(m: Module) -> Self {
        Self {
            id: m.id,
            code: m.code,
            year: m.year,
            description: m.description,
            credits: m.credits,
            created_at: m.created_at.to_rfc3339(),
            updated_at: m.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Serialize)]
pub struct FilterResponse {
    pub modules: Vec<ModuleDetailsResponse>,
    pub page: i32,
    pub per_page: i32,
    pub total: i32,
}

impl From<(Vec<Module>, i32, i32, i32)> for FilterResponse {
    fn from(data: (Vec<Module>, i32, i32, i32)) -> Self {
        let (modules, page, per_page, total) = data;
        Self {
            modules: modules
                .into_iter()
                .map(ModuleDetailsResponse::from)
                .collect(),
            page,
            per_page,
            total,
        }
    }
}

/// Retrieves a paginated and optionally filtered list of modules.
///
/// # Arguments
///
/// The arguments are automatically extracted from query parameters via the `FilterReq` struct:
/// - `page`: (Optional) The page number for pagination. Defaults to 1 if not provided. Minimum value is 1.
/// - `per_page`: (Optional) The number of items per page. Defaults to 20. Maximum is 100. Minimum is 1.
/// - `query`: (Optional) A general search string that filters modules by `code` or `description`.
/// - `code`: (Optional) A filter to match specific module codes.
/// - `year`: (Optional) A filter to match modules by academic year.
/// - `sort`: (Optional) A comma-separated list of fields to sort by. Prefix with `-` for descending order (e.g., `-year`).
///
/// Allowed sort fields: `"code"`, `"created_at"`, `"year"`, `"credits"`.
///
/// # Returns
///
/// Returns an HTTP response indicating the result:
/// - `200 OK` with a list of matching modules, paginated and wrapped in a standardized response format.
/// - `400 BAD REQUEST` if an invalid field is used for sorting.
/// - `500 INTERNAL SERVER ERROR` if a database error occurs while retrieving the modules.
///
/// The response body contains:
/// - A paginated list of modules.
/// - Metadata: current page, items per page, and total items.
pub async fn get_modules(Query(params): Query<FilterReq>) -> impl IntoResponse {
    let db: DatabaseConnection = db::connect().await;

    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).min(100).max(1);

    // Validate sort fields
    if let Some(sort) = &params.sort {
        let valid_fields = ["code", "created_at", "year", "credits", "description"];
        for field in sort.split(',') {
            let field = field.trim_start_matches('-');
            if !valid_fields.contains(&field) {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::<FilterResponse>::error("Invalid field used for sorting")),
                );
            }
        }
    }

    // Build filter conditions
    let mut condition = Condition::all();

    if let Some(ref q) = params.query {
        let q = q.to_lowercase();
        condition = condition.add(
            ModuleCol::Code.contains(&q).or(ModuleCol::Description.contains(&q)),
        );
    }

    if let Some(ref code) = params.code {
        condition = condition.add(ModuleCol::Code.contains(&code.to_lowercase()));
    }

    if let Some(year) = params.year {
        condition = condition.add(ModuleCol::Year.eq(year));
    }

    // Base query
    let mut query = ModuleEntity::find().filter(condition);

    // Apply sorting
    if let Some(sort_str) = &params.sort {
        for field in sort_str.split(',') {
            let trimmed = field.trim();
            if trimmed.is_empty() {
                continue;
            }

            let (column, descending) = if trimmed.starts_with('-') {
                (&trimmed[1..], true)
            } else {
                (trimmed, false)
            };

            match column {
                "code" => {
                    query = if descending {
                        query.order_by_desc(ModuleCol::Code)
                    } else {
                        query.order_by_asc(ModuleCol::Code)
                    };
                }
                "created_at" => {
                    query = if descending {
                        query.order_by_desc(ModuleCol::CreatedAt)
                    } else {
                        query.order_by_asc(ModuleCol::CreatedAt)
                    };
                }
                "year" => {
                    query = if descending {
                        query.order_by_desc(ModuleCol::Year)
                    } else {
                        query.order_by_asc(ModuleCol::Year)
                    };
                }
                "credits" => {
                    query = if descending {
                        query.order_by_desc(ModuleCol::Credits)
                    } else {
                        query.order_by_asc(ModuleCol::Credits)
                    };
                }
                "description" => {
                    query = if descending {
                        query.order_by_desc(ModuleCol::Description)
                    } else {
                        query.order_by_asc(ModuleCol::Description)
                    };
                }
                _ => {}
            }
        }
    }

    // Paginate and fetch
    let paginator = query.paginate(&db, per_page as u64);
    let total = paginator.num_items().await.unwrap_or(0) as i32;
    let modules: Vec<Module> = paginator.fetch_page((page - 1) as u64).await.unwrap_or_default();

    let response = FilterResponse::from((modules, page, per_page, total));
    (
        StatusCode::OK,
        Json(ApiResponse::success(response, "Modules retrieved successfully")),
    )
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MyDetailsResponse {
    pub as_student: Vec<ModuleDetailsResponse>,
    pub as_tutor: Vec<ModuleDetailsResponse>,
    pub as_lecturer: Vec<ModuleDetailsResponse>,
}

impl From<(Vec<Module>, Vec<Module>, Vec<Module>)> for MyDetailsResponse {
    fn from((as_student, as_tutor, as_lecturer): (Vec<Module>, Vec<Module>, Vec<Module>)) -> Self {
        MyDetailsResponse {
            as_student: as_student.into_iter().map(ModuleDetailsResponse::from).collect(),
            as_tutor: as_tutor.into_iter().map(ModuleDetailsResponse::from).collect(),
            as_lecturer: as_lecturer.into_iter().map(ModuleDetailsResponse::from).collect(),
        }
    }
}

pub async fn get_my_details(
    Extension(AuthUser(claims)): Extension<AuthUser>,
) -> impl IntoResponse {
    let db: DatabaseConnection = connect().await;
    let user_id = claims.sub;

    let (as_student, as_tutor, as_lecturer) = tokio::join!(
        get_modules_by_user_and_role(&db, user_id, Role::Student),
        get_modules_by_user_and_role(&db, user_id, Role::Tutor),
        get_modules_by_user_and_role(&db, user_id, Role::Lecturer),
    );

    match (as_student, as_tutor, as_lecturer) {
        (Ok(students), Ok(tutors), Ok(lecturers)) => {
            let response = MyDetailsResponse::from((students, tutors, lecturers));
            (
                StatusCode::OK,
                Json(ApiResponse::success(
                    response,
                    "My module details retrieved successfully",
                )),
            )
        }
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<MyDetailsResponse>::error(
                "An error occurred while retrieving module details",
            )),
        ),
    }
}

/// Helper to fetch modules by user_id and role using SeaORM relations
async fn get_modules_by_user_and_role(
    db: &DatabaseConnection,
    user_id: i64,
    role: Role,
) -> Result<Vec<Module>, sea_orm::DbErr> {
    RoleEntity::find()
        .filter(
            Condition::all()
                .add(RoleCol::UserId.eq(user_id))
                .add(RoleCol::Role.eq(role)),
        )
        .find_also_related(ModuleEntity) // this returns tuples (role, Option<module>)
        .all(db)
        .await
        .map(|results| {
            results
                .into_iter()
                .filter_map(|(_, module)| module) // extract just the Some(module)
                .collect()
        })
}
