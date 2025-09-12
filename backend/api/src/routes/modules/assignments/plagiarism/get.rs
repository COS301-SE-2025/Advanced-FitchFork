use axum::{body::Body, extract::{Path, Query, State}, http::{HeaderMap, HeaderValue, StatusCode}, response::IntoResponse, Json};
use chrono::{DateTime, Utc};
use db::models::{
    assignment_submission::{self, Entity as SubmissionEntity},
    plagiarism_case::{self, Entity as PlagiarismEntity, Status},
    user::{self, Entity as UserEntity},
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Condition, QuerySelect, QueryTrait, QueryOrder, PaginatorTrait};
use serde::{Deserialize, Serialize};
use tokio::fs::File;
use tokio_util::io::ReaderStream;
use std::{path::PathBuf, str::FromStr};
use std::collections::HashMap;
use util::{paths::assignment_dir, state::AppState};
use crate::response::ApiResponse;
use std::fs;

#[derive(Serialize)]
pub struct MossReportResponse {
    pub report_url: String,
    pub generated_at: String,
    pub has_archive: bool,
    pub archive_generated_at: Option<String>,
}

/// GET /api/modules/{module_id}/assignments/{assignment_id}/plagiarism/moss
///
/// Retrieves metadata for the **most recent MOSS report** generated for the given assignment.
/// Accessible only to lecturers and assistant lecturers assigned to the module.
///
/// This endpoint does **not** trigger a new MOSS run—it only returns the last stored report URL
/// and its generation timestamp. To generate a new report, use the POST endpoint:
/// `/api/modules/{module_id}/assignments/{assignment_id}/plagiarism/moss`.
///
/// # Path Parameters
///
/// - `module_id`: The ID of the parent module
/// - `assignment_id`: The ID of the assignment whose latest MOSS report should be fetched
///
/// # Request Body
///
/// None.
///
/// # Returns
///
/// - `200 OK` on success with the latest report metadata:
///   - `report_url` — The external URL to the MOSS results page
///   - `generated_at` — RFC 3339 timestamp for when the report file was written
///   - `has_archive` — `true` if a ZIP archive of a mirrored report exists on the server
///   - `archive_generated_at` — RFC 3339 timestamp for when `moss_archive.zip` was last written; `null` if absent
/// - `404 NOT FOUND` if no report has been generated yet
/// - `500 INTERNAL SERVER ERROR` if the report file cannot be read or parsed
///
/// # Example Response (200 OK)
///
/// ```json
/// {
///   "success": true,
///   "message": "MOSS report retrieved successfully",
///   "data": {
///     "report_url": "http://moss.stanford.edu/results/123456789",
///     "generated_at": "2025-05-30T12:34:56Z",
///     "has_archive": true,
///     "archive_generated_at": "2025-05-30T12:40:02Z"
///   }
/// }
/// ```
///
/// # Example Response (404 Not Found)
///
/// ```json
/// {
///   "success": false,
///   "message": "MOSS report not found"
/// }
/// ```
///
/// # Example Response (500 Internal Server Error)
///
/// ```json
/// {
///   "success": false,
///   "message": "Failed to read MOSS report: <reason>"
/// }
/// ```
///
/// # Notes
/// - Internally, the metadata is read from `reports.txt` under the assignment’s storage directory:
///   `.../module_{module_id}/assignment_{assignment_id}/reports.txt`.
/// - `has_archive` checks for the presence of `moss_archive.zip` in the same directory.  
///   `archive_generated_at` is taken from that file’s last-modified timestamp.
/// - The `report_url` is hosted by the MOSS service and may expire per MOSS retention policy.
/// - To refresh the report, run the POST `/plagiarism/moss` endpoint and then call this GET again.
pub async fn get_moss_report(
    Path((module_id, assignment_id)): Path<(i64, i64)>,
) -> impl IntoResponse {
    let base_dir = assignment_dir(module_id, assignment_id);
    let report_path = base_dir.join("reports.txt");
    let archive_zip_path = base_dir.join("moss_archive.zip");

    if !report_path.exists() {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("MOSS report not found")),
        )
            .into_response();
    }

    let content = match fs::read_to_string(&report_path) {
        Ok(content) => content,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(format!("Failed to read MOSS report: {e}"))),
            )
                .into_response();
        }
    };

    let mut report_url = String::new();
    let mut generated_at = String::new();

    for line in content.lines() {
        if let Some(url) = line.strip_prefix("Report URL: ") {
            report_url = url.to_string();
        } else if let Some(date) = line.strip_prefix("Date: ") {
            generated_at = date.to_string();
        }
    }

    if report_url.is_empty() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error("Failed to parse MOSS report")),
        )
            .into_response();
    }

    let has_archive = archive_zip_path.exists();

    // Best-effort read of archive modified time → RFC 3339
    let archive_generated_at = if has_archive {
        fs::metadata(&archive_zip_path)
            .ok()
            .and_then(|m| m.modified().ok())
            .map(|st| DateTime::<Utc>::from(st).to_rfc3339())
    } else {
        None
    };

    (
        StatusCode::OK,
        Json(ApiResponse::success(
            MossReportResponse {
                report_url,
                generated_at,
                has_archive,
                archive_generated_at,
            },
            "MOSS report retrieved successfully",
        )),
    )
        .into_response()
}

#[derive(Debug, Deserialize)]
pub struct ListPlagiarismCaseQueryParams {
    page: Option<u64>,
    per_page: Option<u64>,
    status: Option<String>,
    query: Option<String>,
    sort: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    id: i64,
    username: String,
    email: String,
    profile_picture_path: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SubmissionResponse {
    id: i64,
    filename: String,
    created_at: chrono::DateTime<chrono::Utc>,
    user: UserResponse,
}

#[derive(Debug, Serialize)]
pub struct PlagiarismCaseResponse {
    id: i64,
    status: String,
    description: String,
    similarity: f32, // <-- NEW
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    submission_1: SubmissionResponse,
    submission_2: SubmissionResponse,
}

#[derive(Debug, Serialize)]
pub struct PlagiarismCaseListResponse {
    cases: Vec<PlagiarismCaseResponse>,
    page: u64,
    per_page: u64,
    total: u64,
}

/// GET /api/modules/{module_id}/assignments/{assignment_id}/plagiarism
///
/// Retrieves paginated plagiarism cases for a specific assignment with filtering and sorting.
/// Only accessible to lecturers and assistant lecturers assigned to the module.
///
/// # Path Parameters
/// - `module_id`: The ID of the parent module
/// - `assignment_id`: The ID of the assignment to retrieve plagiarism cases for
///
/// # Query Parameters
/// - `page`: (Optional) Page number (default: 1, min: 1)
/// - `per_page`: (Optional) Items per page (default: 20, max: 100)
/// - `status`: (Optional) Filter by status: `"review"`, `"flagged"`, or `"reviewed"`
/// - `query`: (Optional) Case-insensitive fuzzy search on usernames of either submission’s user
/// - `sort`: (Optional) Comma-separated sorting criteria; prefix with `-` for descending.
///   **Valid fields:** `"created_at"`, `"status"`, `"similarity"`
///
/// # Returns
/// - `200 OK` with paginated cases on success
/// - `400 BAD REQUEST` for invalid params (status/sort/pagination)
/// - `403 FORBIDDEN` if user lacks permissions
/// - `500 INTERNAL SERVER ERROR` for database failures
///
/// # Example Response (200 OK)
/// ```json
/// {
///   "success": true,
///   "message": "Plagiarism cases retrieved successfully",
///   "data": {
///     "cases": [
///       {
///         "id": 12,
///         "status": "flagged",
///         "description": "Very similar submissions",
///         "similarity": 84.3,
///         "created_at": "2024-05-15T08:30:00Z",
///         "updated_at": "2024-05-16T10:15:00Z",
///         "submission_1": {
///           "id": 42,
///           "filename": "main.cpp",
///           "created_at": "2024-05-14T09:00:00Z",
///           "user": { "id": 5, "username": "u12345678", "email": "abc@example.com", "profile_picture_path": null }
///         },
///         "submission_2": {
///           "id": 43,
///           "filename": "main.cpp",
///           "created_at": "2024-05-14T10:30:00Z",
///           "user": { "id": 6, "username": "u98765432", "email": "xyz@example.com", "profile_picture_path": null }
///         }
///       }
///     ],
///     "page": 1,
///     "per_page": 20,
///     "total": 1
///   }
/// }
/// ```
///
/// # Example Errors
/// - `400 Bad Request` — `{ "success": false, "message": "Invalid status parameter" }`
/// - `403 Forbidden` — `{ "success": false, "message": "Forbidden: Insufficient permissions" }`
/// - `500 Internal Server Error` — `{ "success": false, "message": "Failed to retrieve plagiarism cases" }`
pub async fn list_plagiarism_cases(
    State(app_state): State<AppState>,
    Path((_, assignment_id)): Path<(i64, i64)>,
    Query(params): Query<ListPlagiarismCaseQueryParams>,
) -> impl IntoResponse {
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).min(100);

    // Limit cases to this assignment’s submissions
    let submission_models = SubmissionEntity::find()
        .filter(assignment_submission::Column::AssignmentId.eq(assignment_id))
        .all(app_state.db())
        .await
        .unwrap_or_default();

    let submission_ids: Vec<i64> = submission_models.iter().map(|s| s.id).collect();

    let mut query = PlagiarismEntity::find().filter(
        Condition::any()
            .add(plagiarism_case::Column::SubmissionId1.is_in(submission_ids.clone()))
            .add(plagiarism_case::Column::SubmissionId2.is_in(submission_ids)),
    );

    // Filter: status
    if let Some(status_str) = params.status {
        if let Ok(status) = Status::from_str(&status_str) {
            query = query.filter(plagiarism_case::Column::Status.eq(status));
        } else {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<PlagiarismCaseListResponse>::error("Invalid status parameter")),
            );
        }
    }

    // Search: username (fuzzy)
    if let Some(search_query) = params.query {
        let user_ids_subquery = UserEntity::find()
            .select_only()
            .column(user::Column::Id)
            .filter(user::Column::Username.like(format!("%{}%", search_query.to_lowercase())))
            .into_query();

        let submission_ids_subquery = SubmissionEntity::find()
            .select_only()
            .column(assignment_submission::Column::Id)
            .filter(assignment_submission::Column::UserId.in_subquery(user_ids_subquery))
            .into_query();

        query = query.filter(
            Condition::any()
                .add(plagiarism_case::Column::SubmissionId1.in_subquery(submission_ids_subquery.clone()))
                .add(plagiarism_case::Column::SubmissionId2.in_subquery(submission_ids_subquery)),
        );
    }

    // Sort: created_at, status, similarity
    if let Some(sort) = params.sort {
        for s in sort.split(',') {
            let (order, column) = if s.starts_with('-') {
                (sea_orm::Order::Desc, &s[1..])
            } else {
                (sea_orm::Order::Asc, s)
            };
            match column {
                "created_at" => query = query.order_by(plagiarism_case::Column::CreatedAt, order),
                "status" => query = query.order_by(plagiarism_case::Column::Status, order),
                "similarity" => query = query.order_by(plagiarism_case::Column::Similarity, order),
                _ => {} // silently ignore unknown sort fields
            }
        }
    }

    let paginator = query.paginate(app_state.db(), per_page);
    let total_items = paginator.num_items().await.unwrap_or(0);
    let cases = paginator.fetch_page(page - 1).await.unwrap_or_default();

    // Pull submissions & users for the cases we fetched
    let submission_ids: Vec<i64> = cases
        .iter()
        .flat_map(|c| [c.submission_id_1, c.submission_id_2])
        .collect();

    let submissions = SubmissionEntity::find()
        .filter(assignment_submission::Column::Id.is_in(submission_ids))
        .all(app_state.db())
        .await
        .unwrap_or_default();

    let user_ids: Vec<i64> = submissions.iter().map(|s| s.user_id).collect();
    let users = UserEntity::find()
        .filter(user::Column::Id.is_in(user_ids))
        .all(app_state.db())
        .await
        .unwrap_or_default();

    let user_map: HashMap<i64, user::Model> = users.into_iter().map(|u| (u.id, u)).collect();
    let submission_map: HashMap<i64, (assignment_submission::Model, user::Model)> = submissions
        .into_iter()
        .filter_map(|s| user_map.get(&s.user_id).cloned().map(|u| (s.id, (s, u))))
        .collect();

    let response_cases: Vec<PlagiarismCaseResponse> = cases
        .into_iter()
        .filter_map(|case| {
            let (s1, u1) = submission_map.get(&case.submission_id_1)?.clone();
            let (s2, u2) = submission_map.get(&case.submission_id_2)?.clone();

            Some(PlagiarismCaseResponse {
                id: case.id,
                status: case.status.to_string(),
                description: case.description,
                similarity: case.similarity,
                created_at: case.created_at,
                updated_at: case.updated_at,
                submission_1: SubmissionResponse {
                    id: s1.id,
                    filename: s1.filename,
                    created_at: s1.created_at,
                    user: UserResponse {
                        id: u1.id,
                        username: u1.username,
                        email: u1.email,
                        profile_picture_path: u1.profile_picture_path,
                    },
                },
                submission_2: SubmissionResponse {
                    id: s2.id,
                    filename: s2.filename,
                    created_at: s2.created_at,
                    user: UserResponse {
                        id: u2.id,
                        username: u2.username,
                        email: u2.email,
                        profile_picture_path: u2.profile_picture_path,
                    },
                },
            })
        })
        .collect();

    let response = PlagiarismCaseListResponse {
        cases: response_cases,
        page,
        per_page,
        total: total_items,
    };

    (
        StatusCode::OK,
        Json(ApiResponse::success(
            response,
            "Plagiarism cases retrieved successfully",
        )),
    )
}

#[derive(Deserialize)]
pub struct PlagiarismQuery {
    pub status: Option<String>,
    pub min_similarity: Option<f32>,
    pub max_similarity: Option<f32>,
    pub user: Option<String>,
}

#[derive(Serialize)]
struct GraphLink {
    source: String,
    target: String,
    case_id: i64,
    similarity: f32,
    status: String, // serialize as string for simplicity
}



#[derive(Serialize)]
struct LinksResponse {
    links: Vec<GraphLink>,
}

/// GET /api/modules/{module_id}/assignments/{assignment_id}/plagiarism/graph
///
/// Builds a **user-to-user plagiarism graph** for the given assignment. Each edge corresponds
/// to a plagiarism case linking submissions from two users.
///
/// Accessible only to lecturers and assistant lecturers assigned to the module.
///
/// # Path Parameters
///
/// - `module_id`: The ID of the parent module
/// - `assignment_id`: The ID of the assignment whose plagiarism graph should be built
///
/// # Query Parameters
///
/// - `status` (optional): Filter edges by case status. One of:
///   - `"review"`
///   - `"flagged"`
///   - `"reviewed"`
/// - `min_similarity` (optional): Minimum similarity threshold (inclusive).
/// - `max_similarity` (optional): Maximum similarity threshold (inclusive).
/// - `user` (optional): Case-insensitive substring match against usernames.  
///   Keeps only edges where at least one endpoint matches the query.
///
/// # Semantics
///
/// - Nodes are **usernames** derived from the submissions involved in cases.
/// - Each returned `Link { source, target, case_id, similarity, status }` represents a plagiarism
///   case between the two users:
///   - `source` / `target`: Usernames of the authors of the submissions involved
///   - `case_id`: The plagiarism case’s unique ID
///   - `similarity`: Similarity score (0–100) reported for the case
///   - `status`: Current review status of the case (`"review"`, `"flagged"`, `"reviewed"`)
/// - Multiple cases between the same user pair will result in multiple edges.  
///   (If you prefer one edge per pair, aggregate or deduplicate in your client.)
/// - Only cases where **both** submissions belong to the specified assignment are considered.
///
/// # Returns
///
/// - `200 OK` with a `links` array (possibly empty) on success
/// - `400 BAD REQUEST` if `status` is provided but invalid
/// - `500 INTERNAL SERVER ERROR` if submissions, users, or cases could not be fetched
///
/// # Example Request
///
/// ```http
/// GET /api/modules/12/assignments/34/plagiarism/graph?status=flagged&min_similarity=60&user=u123
/// ```
///
/// # Example Response (200 OK)
///
/// ```json
/// {
///   "success": true,
///   "message": "Plagiarism graph retrieved successfully",
///   "data": {
///     "links": [
///       {
///         "source": "u12345678",
///         "target": "u87654321",
///         "case_id": 42,
///         "similarity": 83.5,
///         "status": "flagged"
///       }
///     ]
///   }
/// }
/// ```
///
/// # Example Response (Empty Graph)
///
/// ```json
/// {
///   "success": true,
///   "message": "Plagiarism graph retrieved successfully",
///   "data": { "links": [] }
/// }
/// ```
///
/// # Example Response (400 Bad Request)
///
/// ```json
/// {
///   "success": false,
///   "message": "Invalid status parameter"
/// }
/// ```
///
/// # Notes
/// - This endpoint is optimized for **visualization**. If you need full case details,
///   use the list endpoint (`GET /plagiarism`) instead.
/// - Edges are derived from the **current** cases in the database after any filtering.
/// - Usernames are taken from the submissions’ authors at query time.
// TODO: Testing @Aidan
pub async fn get_graph(
    State(app_state): State<AppState>,
    Path((_module_id, assignment_id)): Path<(i64, i64)>,
    Query(query): Query<PlagiarismQuery>,
) -> impl IntoResponse {
    use sea_orm::{ColumnTrait, QueryFilter};

    // 1) Base: assignment filter
    let mut q = PlagiarismEntity::find()
        .filter(plagiarism_case::Column::AssignmentId.eq(assignment_id));

    // 2) Optional status (validated)
    if let Some(status_str) = query.status.as_deref() {
        match Status::try_from(status_str) {
            Ok(status) => {
                q = q.filter(plagiarism_case::Column::Status.eq(status));
            }
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::<LinksResponse>::error("Invalid status parameter")),
                );
            }
        }
    }

    // 3) Optional similarity (pushed down to SQL)
    if let Some(min) = query.min_similarity {
        q = q.filter(plagiarism_case::Column::Similarity.gte(min));
    }
    if let Some(max) = query.max_similarity {
        q = q.filter(plagiarism_case::Column::Similarity.lte(max));
    }

    // 4) Fetch cases
    let cases = match q.all(app_state.db()).await {
        Ok(cs) => cs,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<LinksResponse>::error("Failed to fetch plagiarism cases")),
            );
        }
    };

    if cases.is_empty() {
        return (
            StatusCode::OK,
            Json(ApiResponse::success(
                LinksResponse { links: vec![] },
                "Plagiarism graph retrieved successfully",
            )),
        );
    }

    // 5) Load submissions
    use std::collections::HashMap;
    let all_sub_ids: Vec<i64> = cases
        .iter()
        .flat_map(|c| [c.submission_id_1, c.submission_id_2])
        .collect();

    let submissions = match SubmissionEntity::find()
        .filter(assignment_submission::Column::Id.is_in(all_sub_ids.clone()))
        .all(app_state.db())
        .await
    {
        Ok(ss) => ss,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<LinksResponse>::error("Failed to fetch submissions for cases")),
            );
        }
    };
    let sub_by_id: HashMap<i64, _> = submissions.into_iter().map(|s| (s.id, s)).collect();

    // 6) Load users
    let user_ids: Vec<i64> = sub_by_id.values().map(|s| s.user_id).collect();
    let users = match UserEntity::find()
        .filter(user::Column::Id.is_in(user_ids))
        .all(app_state.db())
        .await
    {
        Ok(us) => us,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<LinksResponse>::error("Failed to fetch users")),
            );
        }
    };
    let user_by_id: HashMap<i64, _> = users.into_iter().map(|u| (u.id, u)).collect();

    // 7) Build links, applying optional USERNAME filter (case-insensitive partial)
    let user_q = query.user.as_ref().map(|s| s.to_lowercase());
    let mut links = Vec::with_capacity(cases.len());

    for case in cases {
        let (s1, s2) = (case.submission_id_1, case.submission_id_2);
        let (Some(sub1), Some(sub2)) = (sub_by_id.get(&s1), sub_by_id.get(&s2)) else { continue };
        let (Some(u1), Some(u2)) = (user_by_id.get(&sub1.user_id), user_by_id.get(&sub2.user_id)) else { continue };

        let u1_name = u1.username.as_str();
        let u2_name = u2.username.as_str();

        if let Some(ref ql) = user_q {
            let u1_match = u1_name.to_lowercase().contains(ql);
            let u2_match = u2_name.to_lowercase().contains(ql);
            if !(u1_match || u2_match) {
                continue; // username filter not satisfied
            }
        }

        links.push(GraphLink {
            source: u1_name.to_string(),
            target: u2_name.to_string(),
            case_id: case.id,
            similarity: case.similarity,
            status: case.status.to_string().to_lowercase(),
        });
    }

    (
        StatusCode::OK,
        Json(ApiResponse::success(
            LinksResponse { links },
            "Plagiarism graph retrieved successfully",
        )),
    )
}

/// GET /api/modules/{module_id}/assignments/{assignment_id}/plagiarism/moss/archive/download
///
/// Streams the **latest archived MOSS report** as a ZIP attachment.
///
/// # Path
/// - `module_id`
/// - `assignment_id`
///
/// # Storage layout
/// - ZIP expected at:
///   `<storage_root>/module_{module_id}/assignment_{assignment_id}/moss_archive.zip`
///
/// # Responses
/// - 200 OK — streams the zip (`application/zip`, `Content-Disposition: attachment`)
/// - 404 Not Found — archive zip not found
/// - 500 Internal Server Error — failed to open or stream file
pub async fn download_moss_archive(
    Path((module_id, assignment_id)): Path<(i64, i64)>,
) -> impl IntoResponse {
    // Where the zip is written by the archive endpoint
    let zip_path: PathBuf = assignment_dir(module_id, assignment_id).join("moss_archive.zip");

    if !zip_path.exists() {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Archive not found. Please archive a MOSS report first.".to_string()))
        )
            .into_response();
    }

    let file = match File::open(&zip_path).await {
        Ok(f) => f,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(format!("Failed to open archive: {e}"))),
            )
                .into_response();
        }
    };

    let filename = zip_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("moss_archive.zip");

    let mut headers = HeaderMap::new();
    headers.insert(
        axum::http::header::CONTENT_TYPE,
        HeaderValue::from_static("application/zip"),
    );
    headers.insert(
        axum::http::header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!("attachment; filename=\"{}\"", filename))
            .unwrap_or(HeaderValue::from_static("attachment; filename=\"archive.zip\"")),
    );

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    (headers, body).into_response()
}