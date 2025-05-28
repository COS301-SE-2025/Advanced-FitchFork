use crate::auth::AuthUser;
use crate::response::ApiResponse;
use crate::routes::modules::post::PersonnelResponse;
use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use db::models::module::Module;
use db::models::module_lecturer::ModuleLecturer;
use db::models::module_student::ModuleStudent;
use db::models::module_tutor::ModuleTutor;
use db::models::user::User;
use db::pool;
use serde::{Deserialize, Serialize};
use sqlx::Arguments;
use sqlx::sqlite::SqliteArguments;
use sqlx::Row;


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

impl From<Module> for ModuleResponse {
    fn from(m: Module) -> Self {
        Self {
            id: m.id,
            code: m.code,
            year: m.year,
            description: m.description,
            credits: m.credits,
            created_at: m.created_at,
            updated_at: m.updated_at,
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

#[derive(Debug, Deserialize)]
pub struct LecturerQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub query: Option<String>,         // fuzzy match on email or student_number
    pub email: Option<String>,         // filter on email (ignored if query is set)
    pub student_number: Option<String>,// filter on student_number (ignored if query is set)
    pub sort: Option<String>,          // e.g. "-email"
}

#[derive(Debug, serde::Serialize)]
pub struct LecturerListResponse {
    pub users: Vec<PersonnelResponse>,
    pub page: u32,
    pub per_page: u32,
    pub total: i64,
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
/// - `query` (string): Fuzzy search term for `email` or `student_number` (case-insensitive).
/// - `email` (string): Filter by email (case-insensitive, ignored if `query` is present).
/// - `student_number` (string): Filter by student number (ignored if `query` is present).
/// - `sort` (string): Sort by field. Prefix with `-` for descending order. Allowed fields:
///   - `email`
///   - `student_number`
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
///         "student_number": "u12345678",
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
    Path(module_id): Path<i64>,
    Query(params): Query<LecturerQuery>,
) -> Response {
    let pool = pool::get();

    // Validate module exists
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

    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;

    let mut base_sql = String::from(
        "FROM users u
         INNER JOIN module_lecturers ml ON u.id = ml.user_id
         WHERE ml.module_id = ?",
    );
    let mut args = SqliteArguments::default();
    args.add(module_id);

    // Filtering
    if let Some(ref q) = params.query {
        base_sql.push_str(" AND (LOWER(u.email) LIKE ? OR LOWER(u.student_number) LIKE ?)");
        let pattern = format!("%{}%", q.to_lowercase());
        args.add(pattern.clone());
        args.add(pattern);
    } else {
        if let Some(ref email) = params.email {
            base_sql.push_str(" AND LOWER(u.email) LIKE ?");
            args.add(format!("%{}%", email.to_lowercase()));
        }
        if let Some(ref sn) = params.student_number {
            base_sql.push_str(" AND LOWER(u.student_number) LIKE ?");
            args.add(format!("%{}%", sn.to_lowercase()));
        }
    }

    // Sorting
    let mut order_sql = String::from(" ORDER BY u.id ASC");
    if let Some(ref sort) = params.sort {
        let (field, dir) = if sort.starts_with('-') {
            (&sort[1..], "DESC")
        } else {
            (sort.as_str(), "ASC")
        };
        let allowed_fields = ["email", "student_number", "created_at"];
        if allowed_fields.contains(&field) {
            order_sql = format!(" ORDER BY u.{} {}", field, dir);
        }
    }

    // Count total
    let count_sql = format!("SELECT COUNT(*) {}", base_sql);
    let total = sqlx::query_scalar_with::<_, i64, _>(&count_sql, args.clone())
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    // Final query
    let data_sql = format!("SELECT u.* {}{} LIMIT ? OFFSET ?", base_sql, order_sql);
    args.add(per_page as i64);
    args.add(offset as i64);

    let users: Vec<User> = sqlx::query_as_with(&data_sql, args)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

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
    pub student_number: Option<String>,
    pub sort: Option<String>,
}

#[derive(serde::Serialize)]
pub struct PaginatedPersonnelResponse {
    pub users: Vec<PersonnelResponse>,
    pub page: u32,
    pub per_page: u32,
    pub total: i64,
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
/// - `query` (string): Fuzzy search term for `email` or `student_number` (case-insensitive).
/// - `email` (string): Filter by email (case-insensitive, ignored if `query` is present).
/// - `student_number` (string): Filter by student number (ignored if `query` is present).
/// - `sort` (string): Sort by field. Prefix with `-` for descending order. Allowed fields:
///   - `email`
///   - `student_number`
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
///         "student_number": "u22222222",
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
    Path(module_id): Path<i64>,
    Query(params): Query<TutorQuery>,
) -> Response {
    let pool = pool::get();
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).min(100);
    let offset = (page - 1) * per_page;

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

    let mut base_sql = String::from(
        "FROM users u
         INNER JOIN module_tutors mt ON u.id = mt.user_id
         WHERE mt.module_id = ?",
    );
    let mut args = SqliteArguments::default();
    args.add(module_id);

    if let Some(ref q) = params.query {
        base_sql.push_str(" AND (LOWER(u.email) LIKE ? OR LOWER(u.student_number) LIKE ?)");
        let pattern = format!("%{}%", q.to_lowercase());
        args.add(pattern.clone());
        args.add(pattern);
    } else {
        if let Some(ref email) = params.email {
            base_sql.push_str(" AND LOWER(u.email) LIKE ?");
            args.add(format!("%{}%", email.to_lowercase()));
        }
        if let Some(ref sn) = params.student_number {
            base_sql.push_str(" AND LOWER(u.student_number) LIKE ?");
            args.add(format!("%{}%", sn.to_lowercase()));
        }
    }

    let count_sql = format!("SELECT COUNT(*) as count {}", base_sql);
    let total = sqlx::query_with(&count_sql, args.clone())
        .fetch_one(pool)
        .await
        .and_then(|row| row.try_get::<i64, _>("count"))
        .unwrap_or(0);

    let mut data_sql = format!("SELECT u.* {}", base_sql);

    if let Some(ref sort) = params.sort {
        let (field, dir) = if sort.starts_with('-') {
            (&sort[1..], "DESC")
        } else {
            (sort.as_str(), "ASC")
        };

        let allowed = ["email", "student_number", "created_at"];
        if allowed.contains(&field) {
            data_sql.push_str(&format!(" ORDER BY u.{} {}", field, dir));
        }
    } else {
        data_sql.push_str(" ORDER BY u.id ASC");
    }

    data_sql.push_str(" LIMIT ? OFFSET ?");
    args.add(per_page as i64);
    args.add(offset as i64);

    let users = sqlx::query_as_with::<_, User, _>(&data_sql, args)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

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
    pub student_number: Option<String>,
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
/// - `student_number` (optional): Partial match on student number (used only if `query` is not provided)
/// - `sort` (optional): Sort by field. Prefix with `-` for descending. Allowed fields: `email`, `student_number`, `created_at`
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
    Path(module_id): Path<i64>,
    Query(params): Query<StudentQuery>,
) -> axum::response::Response {
    let pool = db::pool::get();
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).min(100);
    let offset = (page - 1) * per_page;

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

    let mut base_sql = String::from(
        "FROM users u
         INNER JOIN module_students ms ON u.id = ms.user_id
         WHERE ms.module_id = ?",
    );
    let mut args = SqliteArguments::default();
    args.add(module_id);

    if let Some(ref q) = params.query {
        base_sql.push_str(" AND (LOWER(u.email) LIKE ? OR LOWER(u.student_number) LIKE ?)");
        let pattern = format!("%{}%", q.to_lowercase());
        args.add(pattern.clone());
        args.add(pattern);
    } else {
        if let Some(ref email) = params.email {
            base_sql.push_str(" AND LOWER(u.email) LIKE ?");
            args.add(format!("%{}%", email.to_lowercase()));
        }
        if let Some(ref sn) = params.student_number {
            base_sql.push_str(" AND LOWER(u.student_number) LIKE ?");
            args.add(format!("%{}%", sn.to_lowercase()));
        }
    }

    let count_sql = format!("SELECT COUNT(*) as count {}", base_sql);
    let total = sqlx::query_with(&count_sql, args.clone())
        .fetch_one(pool)
        .await
        .and_then(|row| row.try_get::<i64, _>("count"))
        .unwrap_or(0);

    let mut data_sql = format!("SELECT u.* {}", base_sql);

    if let Some(ref sort) = params.sort {
        let (field, dir) = if sort.starts_with('-') {
            (&sort[1..], "DESC")
        } else {
            (sort.as_str(), "ASC")
        };

        let allowed_fields = ["email", "student_number", "created_at"];
        if allowed_fields.contains(&field) {
            data_sql.push_str(&format!(" ORDER BY u.{} {}", field, dir));
        }
    } else {
        data_sql.push_str(" ORDER BY u.id ASC");
    }

    data_sql.push_str(" LIMIT ? OFFSET ?");
    args.add(per_page as i64);
    args.add(offset as i64);

    let users = sqlx::query_as_with::<_, User, _>(&data_sql, args)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

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
    pub student_number: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct EligibleUserListResponse {
    pub users: Vec<User>,
    pub page: u32,
    pub per_page: u32,
    pub total: i64,
}

pub async fn get_eligible_users_for_module(
    Path(module_id): Path<i64>,
    Query(params): Query<EligibleUserQuery>,
) -> impl IntoResponse {
    let pool = pool::get();
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;

    if !["Lecturer", "Tutor", "Student"].contains(&params.role.as_str()) {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<EligibleUserListResponse>::error("Invalid role")),
        );
    }

    let mut base_sql = r#"
        FROM users
        WHERE id NOT IN (
            SELECT user_id FROM module_lecturers WHERE module_id = ?
            UNION
            SELECT user_id FROM module_tutors WHERE module_id = ?
            UNION
            SELECT user_id FROM module_students WHERE module_id = ?
        )
    "#.to_string();

    let mut args = sqlx::sqlite::SqliteArguments::default();
    args.add(module_id);
    args.add(module_id);
    args.add(module_id);

    if let Some(ref q) = params.query {
        if !q.trim().is_empty() {
            base_sql.push_str(" AND (LOWER(email) LIKE ? OR LOWER(student_number) LIKE ?)");
            let pattern = format!("%{}%", q.to_lowercase());
            args.add(pattern.clone());
            args.add(pattern);
        }
    }

    // These apply regardless of query
    if let Some(ref email) = params.email {
        if !email.trim().is_empty() {
            base_sql.push_str(" AND LOWER(email) LIKE ?");
            args.add(format!("%{}%", email.to_lowercase()));
        }
    }

    if let Some(ref student_number) = params.student_number {
        if !student_number.trim().is_empty() {
            base_sql.push_str(" AND LOWER(student_number) LIKE ?");
            args.add(format!("%{}%", student_number.to_lowercase()));
        }
    }

    let count_sql = format!("SELECT COUNT(*) as count {}", base_sql);
    let total = match sqlx::query_with(&count_sql, args.clone())
        .fetch_one(pool)
        .await
    {
        Ok(row) => row.try_get("count").unwrap_or(0),
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<EligibleUserListResponse>::error(format!(
                    "Count query failed: {}",
                    e
                ))),
            )
        }
    };

    let mut data_sql = format!("SELECT * {}", base_sql);
    if let Some(sort) = &params.sort {
        let (field, direction) = if sort.starts_with('-') {
            (&sort[1..], "DESC")
        } else {
            (&sort[..], "ASC")
        };
        data_sql.push_str(&format!(" ORDER BY {} {}", field, direction));
    } else {
        data_sql.push_str(" ORDER BY id ASC");
    }

    data_sql.push_str(" LIMIT ? OFFSET ?");
    args.add(per_page as i64);
    args.add(offset as i64);

    match sqlx::query_as_with::<_, User, _>(&data_sql, args)
        .fetch_all(pool)
        .await
    {
        Ok(users) => (
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
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<EligibleUserListResponse>::error(format!(
                "Failed to fetch users: {}",
                e
            ))),
        ),
    }
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
    let offset = (page - 1) * length;

    // Validate sort fields
    if let Some(sort) = &params.sort {
        let valid_fields: [&'static str; 5] = ["code", "created_at", "year", "credits", "description"];
        for field in sort.split(',') {
            let field = field.trim_start_matches('-');
            if !valid_fields.contains(&field) {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::<FilterResponse>::error("Invalid field used")),
                );
            }
        }
    }

    let pool = pool::get();
    let mut args = sqlx::sqlite::SqliteArguments::default();
    let mut base_sql = String::from("FROM modules WHERE 1=1");

    if let Some(ref q) = params.query {
        base_sql.push_str(" AND (LOWER(code) LIKE ? OR LOWER(description) LIKE ?)");
        let q_like = format!("%{}%", q.to_lowercase());
        args.add(q_like.clone());
        args.add(q_like);
    }

    if let Some(ref c) = params.code {
        base_sql.push_str(" AND LOWER(code) LIKE ?");
        args.add(format!("%{}%", c.to_lowercase()));
    }

    if let Some(y) = params.year {
        base_sql.push_str(" AND year = ?");
        args.add(y);
    }

    // Count total
    let total_query = format!("SELECT COUNT(*) {}", base_sql);
    let total: i32 = sqlx::query_scalar_with(&total_query, args.clone())
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    // Add sorting
    let mut final_sql = format!("SELECT * {}", base_sql);
    if let Some(sort_str) = params.sort {
        let mut order_clauses = Vec::new();
        for field in sort_str.split(',') {
            let trimmed = field.trim();
            if trimmed.is_empty() {
                continue;
            }
            let (field_name, direction) = if trimmed.starts_with('-') {
                (&trimmed[1..], "DESC")
            } else {
                (trimmed, "ASC")
            };
            order_clauses.push(format!("{} {}", field_name, direction));
        }
        if !order_clauses.is_empty() {
            final_sql.push_str(" ORDER BY ");
            final_sql.push_str(&order_clauses.join(", "));
        }
    }

    // Pagination
    final_sql.push_str(" LIMIT ? OFFSET ?");
    args.add(length);
    args.add(offset);

    let modules: Vec<Module> = sqlx::query_as_with(&final_sql, args)
        .fetch_all(pool)
        .await
        .unwrap_or_else(|_| vec![]);

    let response = FilterResponse::from((modules, page, length, total));
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
        use std::convert::From;
        MyDetailsResponse {
            as_student: as_student.into_iter().map(From::from).collect(),
            as_tutor: as_tutor.into_iter().map(From::from).collect(),
            as_lecturer: as_lecturer.into_iter().map(From::from).collect(),
        }
    }
}

pub async fn get_my_details(Extension(AuthUser(claims)): Extension<AuthUser>) -> impl IntoResponse {
    let id = claims.sub;
    let pool = pool::get();
    let as_student = ModuleStudent::get_by_user_id(Some(pool), id).await;
    let as_tutor = ModuleTutor::get_by_user_id(Some(pool), id).await;
    let as_lecturer = ModuleLecturer::get_by_user_id(Some(pool), id).await;

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
