use crate::auth::AuthUser;
use crate::response::ApiResponse;
use crate::routes::modules::post::PersonnelResponse;
use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use db::models::module::Module;
use db::models::user::User;
use db::pool;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ModuleResponse {
    pub module: ModuleDetailsResponse,
    pub lecturers: Vec<UserResponse>,
    pub tutors: Vec<UserResponse>,
    pub students: Vec<UserResponse>,
}

impl From<Module> for ModuleResponse {
    fn from(m: Module) -> Self {
        Self {
            module: ModuleDetailsResponse {
                id: m.id,
                code: m.code,
                year: m.year,
                description: m.description,
                credits: m.credits,
                created_at: m.created_at,
                updated_at: m.updated_at,
            },
            lecturers: Vec::new(),
            tutors: Vec::new(),
            students: Vec::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: i64,
    pub student_number: String,
    pub email: String,
    pub admin: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// GET /api/modules/:module_id/lecturers
///
/// Retrieve a list of users assigned as lecturers to the specified module.
///
/// ### Access Control
/// This endpoint is accessible to:
/// - Admin users (claims.admin == true)
///
/// ### Path Parameter
/// - `module_id` (integer): The ID of the module to retrieve lecturers for.
///
/// ### Authentication
/// Requires a valid JWT with the appropriate permissions. Returns 403 if the user
/// is not assigned to the module or is not an admin.
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
///       "student_number": "u12345678",
///       "email": "lecturer@example.com",
///       "admin": false,
///       "created_at": "2025-05-23T18:00:00Z",
///       "updated_at": "2025-05-23T18:00:00Z"
///     }
///   ],
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
pub async fn get_lecturers(Path(module_id): Path<i64>, AuthUser(claims): AuthUser) -> Response {
    let pool = pool::get();

    let module_exists = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM modules WHERE id = ?)")
        .bind(module_id)
        .fetch_one(pool)
        .await
        .unwrap_or(false);

    if !module_exists {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Module not found")),
        )
            .into_response();
    }

    let is_admin = claims.admin;
    let is_assigned = sqlx::query_scalar(
        "SELECT EXISTS(
            SELECT 1 FROM module_lecturers WHERE module_id = ? AND user_id = ?
            UNION
            SELECT 1 FROM module_tutors WHERE module_id = ? AND user_id = ?
            UNION
            SELECT 1 FROM module_students WHERE module_id = ? AND user_id = ?
        )",
    )
    .bind(module_id)
    .bind(claims.sub)
    .bind(module_id)
    .bind(claims.sub)
    .bind(module_id)
    .bind(claims.sub)
    .fetch_one(pool)
    .await
    .unwrap_or(false);

    if !is_admin && !is_assigned {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<()>::error(
                "You do not have permission to view this module's users",
            )),
        )
            .into_response();
    }

    let users = sqlx::query_as::<_, User>(
        "SELECT u.* FROM users u
         INNER JOIN module_lecturers ml ON u.id = ml.user_id
         WHERE ml.module_id = ?",
    )
    .bind(module_id)
    .fetch_all(pool)
    .await
    .unwrap_or_else(|_| vec![]);

    let result: Vec<PersonnelResponse> = users.into_iter().map(Into::into).collect();

    (
        StatusCode::OK,
        Json(ApiResponse::success(
            result,
            "Lecturers retrieved successfully",
        )),
    )
        .into_response()
}

/// GET /api/modules/:module_id/tutors
///
/// Retrieve a list of users assigned as tutors to the specified module.
///
/// ### Access Control
/// This endpoint is accessible to:
/// - Admin users
///
///
/// ### Path Parameter
/// - `module_id` (integer): The ID of the module to retrieve tutors for.
///
/// ### Authentication
/// Requires a valid JWT. Users must be either an admin or assigned to the module.
///
/// ### Responses
///
/// - `200 OK`  
/// ```json
/// {
///   "success": true,
///   "data": [
///     {
///       "id": 7,
///       "student_number": "u22222222",
///       "email": "tutor@example.com",
///       "admin": false,
///       "created_at": "2025-05-23T18:00:00Z",
///       "updated_at": "2025-05-23T18:00:00Z"
///     }
///   ],
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

pub async fn get_tutors(
    Path(module_id): Path<i64>,
    AuthUser(claims): AuthUser,
) -> axum::response::Response {
    let pool = db::pool::get();

    let module_exists = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM modules WHERE id = ?)")
        .bind(module_id)
        .fetch_one(pool)
        .await
        .unwrap_or(false);

    if !module_exists {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Module not found")),
        )
            .into_response();
    }

    let is_admin = claims.admin;
    let is_assigned = sqlx::query_scalar(
        "SELECT EXISTS(
            SELECT 1 FROM module_lecturers WHERE module_id = ? AND user_id = ?
            UNION
            SELECT 1 FROM module_tutors WHERE module_id = ? AND user_id = ?
            UNION
            SELECT 1 FROM module_students WHERE module_id = ? AND user_id = ?
        )",
    )
    .bind(module_id)
    .bind(claims.sub)
    .bind(module_id)
    .bind(claims.sub)
    .bind(module_id)
    .bind(claims.sub)
    .fetch_one(pool)
    .await
    .unwrap_or(false);

    if !is_admin && !is_assigned {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<()>::error(
                "You do not have permission to view this module's users",
            )),
        )
            .into_response();
    }

    let users = sqlx::query_as::<_, db::models::user::User>(
        "SELECT u.* FROM users u
         INNER JOIN module_tutors mt ON u.id = mt.user_id
         WHERE mt.module_id = ?",
    )
    .bind(module_id)
    .fetch_all(pool)
    .await
    .unwrap_or_else(|_| vec![]);

    let result: Vec<PersonnelResponse> = users.into_iter().map(Into::into).collect();

    (
        StatusCode::OK,
        Json(ApiResponse::success(
            result,
            "Tutors retrieved successfully",
        )),
    )
        .into_response()
}

/// GET /api/modules/:module_id/students
///
/// Retrieve a list of users enrolled as students in the specified module.
///
/// ### Access Control
/// This endpoint is accessible to:
/// - Admin users
///
///
/// ### Path Parameter
/// - `module_id` (integer): The ID of the module to retrieve students for.
///
/// ### Authentication
/// Requires a valid JWT. Access is denied to unauthorized users.
///
/// ### Responses
///
/// - `200 OK`  
/// ```json
/// {
///   "success": true,
///   "data": [
///     {
///       "id": 11,
///       "student_number": "u33333333",
///       "email": "student@example.com",
///       "admin": false,
///       "created_at": "2025-05-23T18:00:00Z",
///       "updated_at": "2025-05-23T18:00:00Z"
///     }
///   ],
///   "message": "Students retrieved successfully"
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
pub async fn get_students(
    Path(module_id): Path<i64>,
    AuthUser(claims): AuthUser,
) -> axum::response::Response {
    let pool = db::pool::get();

    let module_exists = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM modules WHERE id = ?)")
        .bind(module_id)
        .fetch_one(pool)
        .await
        .unwrap_or(false);

    if !module_exists {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Module not found")),
        )
            .into_response();
    }

    let is_admin = claims.admin;
    let is_assigned = sqlx::query_scalar(
        "SELECT EXISTS(
            SELECT 1 FROM module_lecturers WHERE module_id = ? AND user_id = ?
            UNION
            SELECT 1 FROM module_tutors WHERE module_id = ? AND user_id = ?
            UNION
            SELECT 1 FROM module_students WHERE module_id = ? AND user_id = ?
        )",
    )
    .bind(module_id)
    .bind(claims.sub)
    .bind(module_id)
    .bind(claims.sub)
    .bind(module_id)
    .bind(claims.sub)
    .fetch_one(pool)
    .await
    .unwrap_or(false);

    if !is_admin && !is_assigned {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<()>::error(
                "You do not have permission to view this module's users",
            )),
        )
            .into_response();
    }

    let users = sqlx::query_as::<_, db::models::user::User>(
        "SELECT u.* FROM users u
         INNER JOIN module_students ms ON u.id = ms.user_id
         WHERE ms.module_id = ?",
    )
    .bind(module_id)
    .fetch_all(pool)
    .await
    .unwrap_or_else(|_| vec![]);

    let result: Vec<PersonnelResponse> = users.into_iter().map(Into::into).collect();

    (
        StatusCode::OK,
        Json(ApiResponse::success(
            result,
            "Students retrieved successfully",
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


pub async fn get_module(Path(module_id): Path<i64>) -> impl IntoResponse {
    let module_res = Module::get_by_id(Some(pool::get()), module_id).await;
    match module_res {
        Ok(Some(module)) => {
            let lecturers = db::models::module_lecturer::ModuleLecturer::get_details_by_id(
                Some(pool::get()),
                module_id,
            )
            .await;

            let tutors = db::models::module_tutor::ModuleTutor::get_details_by_id(
                Some(pool::get()),
                module_id,
            )
            .await;

            let students = db::models::module_student::ModuleStudent::get_details_by_id(
                Some(pool::get()),
                module_id,
            )
            .await;

            if lecturers.is_err() || tutors.is_err() || students.is_err() {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<ModuleResponse>::error(
                        "Failed to retrieve module personnel".to_string(),
                    )),
                );
            }
            let mut response = ModuleResponse::from(module);
            response.lecturers = lecturers
                .unwrap_or_default()
                .into_iter()
                .map(|u| UserResponse {
                    id: u.id,
                    student_number: u.student_number,
                    email: u.email,
                    admin: u.admin,
                    created_at: u.created_at,
                    updated_at: u.updated_at,
                })
                .collect();
            response.tutors = tutors
                .unwrap_or_default()
                .into_iter()
                .map(|u| UserResponse {
                    id: u.id,
                    student_number: u.student_number,
                    email: u.email,
                    admin: u.admin,
                    created_at: u.created_at,
                    updated_at: u.updated_at,
                })
                .collect();
            response.students = students
                .unwrap_or_default()
                .into_iter()
                .map(|u| UserResponse {
                    id: u.id,
                    student_number: u.student_number,
                    email: u.email,
                    admin: u.admin,
                    created_at: u.created_at,
                    updated_at: u.updated_at,
                })
                .collect();

            (
                StatusCode::OK,
                Json(ApiResponse::success(
                    response,
                    "Module retrieved successfully",
                )),
            )
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<ModuleResponse>::error("Module not found")),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<ModuleResponse>::error(
                "An error occurred in the database",
            )),
        ),
    }
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
            created_at: m.created_at,
            updated_at: m.updated_at,
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
    let page = params.page.unwrap_or(1).max(1);
    let length = params.per_page.unwrap_or(20).min(100).max(1);

    if params.sort.is_some() {
        let valid_fields = ["code", "created_at", "year", "credits"];
        if !valid_fields.contains(&params.sort.as_ref().unwrap().as_str()) {
            return (StatusCode::BAD_REQUEST, Json(ApiResponse::<FilterResponse>::error("Invalid field used")));
        }
    }
    let res = Module::filter(
        Some(pool::get()),
        page,
        length,
        params.query,
        params.code,
        params.year,
        params.sort,
    )
    .await;
    match res {
        Ok(data) => {
            let total = data.len() as i32;
            let response: FilterResponse = FilterResponse::from((data, page, length, total));
            (
                StatusCode::OK,
                Json(ApiResponse::success(
                    response,
                    "Modules retrieved successfully",
                )),
            )
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<FilterResponse>::error(
                "An error occurred while retrieving modules",
            )),
        ),
    }
}
