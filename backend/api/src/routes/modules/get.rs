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
    extract::{Path, Query},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use services::module::{Module, ModuleService};
use services::service::{AppError, Service};
use services::user_module_role::UserModuleRoleService;
use util::filters::{FilterParam, QueryParam};

#[derive(Debug, Serialize, Deserialize)]
pub struct ModuleResponse {
    pub id: i64,
    pub code: String,
    pub year: i32,
    pub description: Option<String>,
    pub credits: i64,
    pub created_at: String,
    pub updated_at: String,
    pub lecturers: Vec<UserResponse>,
    pub assistant_lecturers: Vec<UserResponse>,
    pub tutors: Vec<UserResponse>,
    pub students: Vec<UserResponse>,
}

impl From<Module> for ModuleResponse {
    fn from(m: Module) -> Self {
        Self {
            id: m.id,
            code: m.code,
            year: m.year,
            description: m.description,
            credits: m.credits,
            created_at: m.created_at.to_rfc3339(),
            updated_at: m.updated_at.to_rfc3339(),
            lecturers: vec![],
            assistant_lecturers: vec![],
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
/// - `500 INTERNAL SERVER ERROR` if a database error occurs or if related personnel data (lecturers, assistant_lecturers, tutors, or students) fails to load.
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
///     "assitant_lecturers": [
///       {
///         "id": 1,
///         "username": "assistant_lecturer1",
///         "email": "assistant_lecturer1@example.com",
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
pub async fn get_module(Path(module_id): Path<i64>) -> Response {
    let module = match ModuleService::find_by_id(module_id).await {
        Ok(Some(module)) => module,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<()>::error("Module not found")),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(format!(
                    "Database error retrieving module: {}",
                    e
                ))),
            )
                .into_response();
        }
    };

    let (lecturers, assistant_lecturers, tutors, students) = tokio::join!(
        get_users_by_role(module_id, "lecturer".to_string()),
        get_users_by_role(module_id, "assistant_lecturer".to_string()),
        get_users_by_role(module_id, "tutor".to_string()),
        get_users_by_role(module_id, "student".to_string()),
    );

    if lecturers.is_err() || assistant_lecturers.is_err() || tutors.is_err() || students.is_err() {
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
    response.assistant_lecturers = assistant_lecturers
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

async fn get_users_by_role(module_id: i64, role: String) -> Result<Vec<User>, AppError> {
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
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub query: Option<String>,
    pub code: Option<String>,
    pub year: Option<i64>,
    pub sort: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ModuleDetailsResponse {
    pub id: i64,
    pub code: String,
    pub year: i32,
    pub description: Option<String>,
    pub credits: i64,
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
    pub page: u64,
    pub per_page: u64,
    pub total: u64,
}

impl From<(Vec<Module>, u64, u64, u64)> for FilterResponse {
    fn from(data: (Vec<Module>, u64, u64, u64)) -> Self {
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
    Extension(AuthUser(claims)): Extension<AuthUser>,
    Query(query): Query<FilterReq>,
) -> impl IntoResponse {
    let user_id = claims.sub;
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100).max(1);
    let sort = query.sort.clone();

    let mut filters = Vec::new();
    let mut queries = Vec::new();

    if let Some(code) = query.code {
        filters.push(FilterParam::like("code", code));
    }
    if let Some(year) = query.year {
        filters.push(FilterParam::eq("year", year));
    }
    if let Some(query_text) = query.query {
        queries.push(QueryParam::new(
            vec!["code".to_string(), "description".to_string()],
            query_text,
        ));
    }

    let module_ids = if !claims.admin {
        match UserModuleRoleService::find_all(
            &vec![FilterParam::eq("user_id", user_id)],
            &vec![],
            None,
        )
        .await
        {
            Ok(memberships) => {
                let ids: Vec<i64> = memberships.into_iter().map(|m| m.module_id).collect();
                Some(ids)
            }
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<FilterResponse>::error(format!(
                        "Database error: {}",
                        e
                    ))),
                );
            }
        }
    } else {
        None
    };

    if let Some(ids) = module_ids {
        if ids.is_empty() {
            let response = FilterResponse::from((Vec::<Module>::new(), page, per_page, 0));
            return (
                StatusCode::OK,
                Json(ApiResponse::success(
                    response,
                    "Modules retrieved successfully",
                )),
            );
        }
        filters.push(FilterParam::eq("id", ids));
    }

    let (modules, total) =
        match ModuleService::filter(&filters, &queries, page, per_page, sort).await {
            Ok(result) => result,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<FilterResponse>::error(format!(
                        "Database error: {}",
                        e
                    ))),
                );
            }
        };

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
pub async fn get_my_details(Extension(AuthUser(claims)): Extension<AuthUser>) -> impl IntoResponse {
    let user_id = claims.sub;

    let (as_student, as_tutor, as_lecturer, as_assistant_lecturer) = tokio::join!(
        get_modules_by_user_and_role(user_id, "student".to_string()),
        get_modules_by_user_and_role(user_id, "tutor".to_string()),
        get_modules_by_user_and_role(user_id, "lecturer".to_string()),
        get_modules_by_user_and_role(user_id, "assistant_lecturer".to_string()),
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
async fn get_modules_by_user_and_role(user_id: i64, role: String) -> Result<Vec<Module>, AppError> {
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
