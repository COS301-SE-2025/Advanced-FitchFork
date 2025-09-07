//! Module query routes.
//!
//! Provides endpoints to retrieve modules:
//! - `GET /api/modules/{id}` → Get details of a single module with assigned users.
//! - `GET /api/modules` → Paginated and optionally filtered list of modules.
//! - `GET /api/modules/me` → Retrieve modules for the authenticated user, grouped by role.
//!
//! All responses follow the standard `ApiResponse` format.

use crate::routes::common::UserResponse;
use crate::{auth::AuthUser, response::ApiResponse};
use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use db::models::user_module_role;
use db::models::{
    module::{Column as ModuleCol, Entity as ModuleEntity, Model as Module},
    user::{Column as UserCol, Entity as UserEntity, Model as UserModel},
    user_module_role::{Column as RoleCol, Entity as RoleEntity, Role},
};
use sea_orm::{
    ColumnTrait, Condition, DatabaseConnection, EntityTrait, JoinType, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect,
};
use serde::{Deserialize, Serialize};
use util::state::AppState;

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

/// GET /api/modules/{module_id}
///
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
///
/// # Example Response
///
/// - `200 OK`  
/// ```json
/// {
///   "success": true,
///   "data": {
///     "id": 1,
///     "code": "CS101",
///     "year": 2024,
///     "description": "Introduction to Computer Science",
///     "credits": 15,
///     "created_at": "2024-01-15T10:00:00Z",
///     "updated_at": "2024-01-15T10:00:00Z",
///     "lecturers": [
///       {
///         "id": 1,
///         "username": "lecturer1",
///         "email": "lecturer1@example.com",
///         "admin": false,
///         "created_at": "2024-01-01T00:00:00Z",
///         "updated_at": "2024-01-01T00:00:00Z"
///       }
///     ],
///     "tutors": [
///       {
///         "id": 2,
///         "username": "tutor1",
///         "email": "tutor1@example.com",
///         "admin": false,
///         "created_at": "2024-01-01T00:00:00Z",
///         "updated_at": "2024-01-01T00:00:00Z"
///       }
///     ],
///     "students": [
///       {
///         "id": 3,
///         "username": "student1",
///         "email": "student1@example.com",
///         "admin": false,
///         "created_at": "2024-01-01T00:00:00Z",
///         "updated_at": "2024-01-01T00:00:00Z"
///       }
///     ]
///   },
///   "message": "Module retrieved successfully"
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
///   "message": "Database error retrieving module"
/// }
/// ```
pub async fn get_module(State(state): State<AppState>, Path(module_id): Path<i64>) -> Response {
    let db = state.db();

    let module = ModuleEntity::find_by_id(module_id)
        .one(db)
        .await
        .unwrap()
        .unwrap();

    let (lecturers, tutors, students) = tokio::join!(
        get_users_by_role(db, module_id, Role::Lecturer),
        get_users_by_role(db, module_id, Role::Tutor),
        get_users_by_role(db, module_id, Role::Student),
    );

    if lecturers.is_err() || tutors.is_err() || students.is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(
                "Failed to retrieve assigned personnel",
            )),
        )
            .into_response();
    }

    let mut response = ModuleResponse::from(module);
    response.lecturers = lecturers
        .unwrap()
        .into_iter()
        .map(UserResponse::from)
        .collect();
    response.tutors = tutors
        .unwrap()
        .into_iter()
        .map(UserResponse::from)
        .collect();
    response.students = students
        .unwrap()
        .into_iter()
        .map(UserResponse::from)
        .collect();

    (
        StatusCode::OK,
        Json(ApiResponse::success(
            response,
            "Module retrieved successfully",
        )),
    )
        .into_response()
}

async fn get_users_by_role(
    db: &DatabaseConnection,
    module_id: i64,
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

/// GET /api/modules
///
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
/// Allowed sort fields: `"code"`, `"created_at"`, `"year"`, `"credits"`, `"description"`.
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
///
/// # Example Response
///
/// - `200 OK`  
/// ```json
/// {
///   "success": true,
///   "data": {
///     "modules": [
///       {
///         "id": 1,
///         "code": "CS101",
///         "year": 2024,
///         "description": "Introduction to Computer Science",
///         "credits": 15,
///         "created_at": "2024-01-15T10:00:00Z",
///         "updated_at": "2024-01-15T10:00:00Z"
///       }
///     ],
///     "page": 1,
///     "per_page": 20,
///     "total": 57
///   },
///   "message": "Modules retrieved successfully"
/// }
/// ```
///
/// - `400 Bad Request`  
/// ```json
/// {
///   "success": false,
///   "message": "Invalid field used for sorting"
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
pub async fn get_modules(
    State(state): State<AppState>,
    Extension(AuthUser(claims)): Extension<AuthUser>,
    Query(params): Query<FilterReq>,
) -> impl IntoResponse {
    let db = state.db();
    let user_id = claims.sub;
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).min(100);

    let build_query = |query: sea_orm::Select<ModuleEntity>| -> sea_orm::Select<ModuleEntity> {
        let mut query = query;

        if let Some(ref q) = params.query {
            let q = q.to_lowercase();
            query = query.filter(
                ModuleCol::Code
                    .contains(&q)
                    .or(ModuleCol::Description.contains(&q)),
            );
        }
        if let Some(ref code) = params.code {
            query = query.filter(ModuleCol::Code.contains(&code.to_lowercase()));
        }
        if let Some(year) = params.year {
            query = query.filter(ModuleCol::Year.eq(year));
        }
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
                query = match column {
                    "code" => {
                        if descending {
                            query.order_by_desc(ModuleCol::Code)
                        } else {
                            query.order_by_asc(ModuleCol::Code)
                        }
                    }
                    "created_at" => {
                        if descending {
                            query.order_by_desc(ModuleCol::CreatedAt)
                        } else {
                            query.order_by_asc(ModuleCol::CreatedAt)
                        }
                    }
                    "year" => {
                        if descending {
                            query.order_by_desc(ModuleCol::Year)
                        } else {
                            query.order_by_asc(ModuleCol::Year)
                        }
                    }
                    "credits" => {
                        if descending {
                            query.order_by_desc(ModuleCol::Credits)
                        } else {
                            query.order_by_asc(ModuleCol::Credits)
                        }
                    }
                    "description" => {
                        if descending {
                            query.order_by_desc(ModuleCol::Description)
                        } else {
                            query.order_by_asc(ModuleCol::Description)
                        }
                    }
                    _ => query,
                };
            }
        }

        query
    };

    // If admin, fetch all modules
    let query = if claims.admin {
        build_query(ModuleEntity::find())
    } else {
        // Otherwise, filter by membership
        let memberships = user_module_role::Entity::find()
            .filter(user_module_role::Column::UserId.eq(user_id))
            .all(db)
            .await
            .unwrap_or_default();

        if memberships.is_empty() {
            let response = FilterResponse::from((Vec::<Module>::new(), page, per_page, 0));
            return (
                StatusCode::OK,
                Json(ApiResponse::success(
                    response,
                    "Modules retrieved successfully",
                )),
            );
        }

        let module_ids: Vec<i64> = memberships.iter().map(|m| m.module_id).collect();
        build_query(ModuleEntity::find().filter(ModuleCol::Id.is_in(module_ids)))
    };

    let paginator = query.paginate(db, per_page as u64);
    let total = paginator.num_items().await.unwrap_or(0) as i32;
    let modules: Vec<Module> = paginator
        .fetch_page((page - 1) as u64)
        .await
        .unwrap_or_default();

    let response = FilterResponse::from((modules, page, per_page, total));
    (
        StatusCode::OK,
        Json(ApiResponse::success(
            response,
            "Modules retrieved successfully",
        )),
    )
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MyDetailsResponse {
    pub as_student: Vec<ModuleDetailsResponse>,
    pub as_tutor: Vec<ModuleDetailsResponse>,
    pub as_lecturer: Vec<ModuleDetailsResponse>,
    pub as_assistant_lecturer: Vec<ModuleDetailsResponse>,
}

impl From<(Vec<Module>, Vec<Module>, Vec<Module>, Vec<Module>)> for MyDetailsResponse {
    fn from(
        (as_student, as_tutor, as_lecturer, as_assistant_lecturer): (
            Vec<Module>,
            Vec<Module>,
            Vec<Module>,
            Vec<Module>,
        ),
    ) -> Self {
        MyDetailsResponse {
            as_student: as_student
                .into_iter()
                .map(ModuleDetailsResponse::from)
                .collect(),
            as_tutor: as_tutor
                .into_iter()
                .map(ModuleDetailsResponse::from)
                .collect(),
            as_lecturer: as_lecturer
                .into_iter()
                .map(ModuleDetailsResponse::from)
                .collect(),
            as_assistant_lecturer: as_assistant_lecturer
                .into_iter()
                .map(ModuleDetailsResponse::from)
                .collect(),
        }
    }
}

/// GET /api/modules/me
///
/// Retrieves detailed information about the modules the authenticated user is assigned to.
///
/// # Arguments
///
/// This endpoint requires authentication. The user ID is automatically extracted from the JWT token.
///
/// # Returns
///
/// Returns an HTTP response indicating the result:
/// - `200 OK` with the user's module assignments organized by role if successful.
/// - `500 INTERNAL SERVER ERROR` if a database error occurs while retrieving the module details.
///
/// The response body contains:
/// - `as_student`: List of modules where the user is assigned as a student.
/// - `as_tutor`: List of modules where the user is assigned as a tutor.
/// - `as_lecturer`: List of modules where the user is assigned as a lecturer.
/// - `as_assistant_lecturer`: List of modules where the user is assigned as an assistant lecturer.
///
/// # Example Response
///
/// - `200 OK`  
/// ```json
/// {
///   "success": true,
///   "data": {
///     "as_student": [
///       { "id": 1, "code": "CS101", "year": 2024, "description": "...", "credits": 15, "created_at": "...", "updated_at": "..." }
///     ],
///     "as_tutor": [
///       { "id": 2, "code": "CS201", "year": 2024, "description": "...", "credits": 20, "created_at": "...", "updated_at": "..." }
///     ],
///     "as_lecturer": [],
///     "as_assistant_lecturer": []
///   },
///   "message": "My module details retrieved successfully"
/// }
/// ```
///
/// - `500 Internal Server Error`  
/// ```json
/// {
///   "success": false,
///   "message": "An error occurred while retrieving module details"
/// }
/// ```
pub async fn get_my_details(
    Extension(AuthUser(claims)): Extension<AuthUser>,
) -> impl IntoResponse {
    let db = state.db();

    let user_id = claims.sub;

    let (as_student, as_tutor, as_lecturer, as_assistant_lecturer) = tokio::join!(
        get_modules_by_user_and_role(db, user_id, Role::Student),
        get_modules_by_user_and_role(db, user_id, Role::Tutor),
        get_modules_by_user_and_role(db, user_id, Role::Lecturer),
        get_modules_by_user_and_role(db, user_id, Role::AssistantLecturer),
    );

    match (as_student, as_tutor, as_lecturer, as_assistant_lecturer) {
        (Ok(students), Ok(tutors), Ok(lecturers), Ok(assistants)) => {
            let response = MyDetailsResponse::from((students, tutors, lecturers, assistants));
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
        .find_also_related(ModuleEntity)
        .all(db)
        .await
        .map(|results| {
            results
                .into_iter()
                .filter_map(|(_, module)| module)
                .collect()
        })
}
