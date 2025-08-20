//! Get announcements handler.
//!
//! Provides an endpoint to retrieve a paginated list of announcements for a specific module.
//!
//! Supports filtering by search query, pinned status, and sorting by various fields.

use crate::response::ApiResponse;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use sea_orm::{ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder};
use serde::{Deserialize, Serialize};
use util::state::AppState;
use db::models::announcements::{
    Column as AnnouncementColumn, Entity as AnnouncementEntity, Model as AnnouncementModel,
};
use db::models::user::{Entity as UserEntity};

#[derive(Serialize)]
pub struct MinimalUser {
    pub id: i64,
    pub username: String,
}

#[derive(Serialize)]
pub struct ShowAnnouncementResponse {
    pub announcement: AnnouncementModel,
    pub user: MinimalUser,
}


#[derive(Debug, Deserialize)]
pub struct FilterReq {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub query: Option<String>,
    pub pinned: Option<String>,
    pub sort: Option<String>,
}

#[derive(Serialize)]
pub struct FilterResponse {
    pub announcements: Vec<AnnouncementModel>,
    pub page: i32,
    pub per_page: i32,
    pub total: i32,
}

impl FilterResponse {
    fn new(announcements: Vec<AnnouncementModel>, page: i32, per_page: i32, total: i32) -> Self {
        Self {
            announcements,
            page,
            per_page,
            total,
        }
    }
}

/// GET /api/modules/{module_id}/announcements
///
/// Retrieves a paginated and optionally filtered list of announcements for a specific module.
/// By default, results are sorted with pinned announcements first (`pinned DESC`)  
/// and then by most recent creation date (`created_at DESC`).  
/// This ensures pinned announcements always appear at the top, with the newest first.
/// If the user explicitly includes `pinned` in the `sort` parameter, the default is overridden.
///
/// # Path Parameters
///
/// - `module_id`: The ID of the module to retrieve announcements for.
///
/// # Query Parameters
///
/// Extracted via the `FilterReq` struct:
/// - `page`: (Optional) Page number for pagination. Defaults to 1. Minimum is 1.
/// - `per_page`: (Optional) Number of items per page. Defaults to 20. Maximum is 100. Minimum is 1.
/// - `query`: (Optional) General search string. Matches announcements by `title` or `body`.
/// - `pinned`: (Optional) Filter by pinned status. Accepts `true` or `false`.
/// - `sort`: (Optional) Comma-separated list of fields to sort by.  
///   Prefix with `-` for descending order (e.g., `-created_at`).  
///   Allowed fields: `"created_at"`, `"updated_at"`, `"title"`, `"pinned"`.
///
/// # Sorting Behavior
///
/// - **Default**: If `sort` is not provided or does not include `pinned`,  
///   results are automatically sorted by:
///   1. `pinned DESC` (pinned items first)
///   2. `created_at DESC` (newest items first)
/// - If `pinned` is explicitly included in `sort`, that order is respected and overrides the default.
///
/// # Returns
///
/// Returns an HTTP response in the standardized API format:
///
/// - `200 OK`: Successfully retrieved the paginated list of announcements.
/// - `400 BAD REQUEST`: Invalid sort field or invalid `pinned` value.
/// - `500 INTERNAL SERVER ERROR`: Database query failed.
///
/// Response contains:
/// - `announcements`: Array of announcement objects.
/// - Pagination metadata: `page`, `per_page`, `total`.
///
/// # Example Response
///
/// **200 OK**
/// ```json
/// {
///   "success": true,
///   "data": {
///     "announcements": [
///       {
///         "id": 1,
///         "module_id": 101,
///         "user_id": 5,
///         "title": "Important update",
///         "body": "Please note the following changes...",
///         "pinned": true,
///         "created_at": "2025-08-16T12:00:00Z",
///         "updated_at": "2025-08-16T12:00:00Z"
///       }
///     ],
///     "page": 1,
///     "per_page": 20,
///     "total": 45
///   },
///   "message": "Announcements retrieved successfully"
/// }
/// ```
///
/// **400 BAD REQUEST**
/// ```json
/// {
///   "success": false,
///   "message": "Invalid field used for sorting"
/// }
/// ```
///
/// **500 INTERNAL SERVER ERROR**
/// ```json
/// {
///   "success": false,
///   "message": "Failed to retrieve announcements"
/// }
/// ```
pub async fn get_announcements(
    Path(module_id): Path<i64>,
    State(app_state): State<AppState>,
    Query(params): Query<FilterReq>,
) -> impl IntoResponse {
    let db = app_state.db();

    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).min(100);

    if let Some(sort_field) = &params.sort {
        let valid_fields = ["created_at", "updated_at", "title", "pinned"];
        for field in sort_field.split(',') {
            let field = field.trim().trim_start_matches('-');
            if !valid_fields.contains(&field) {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::<FilterResponse>::error("Invalid field used")),
                )
                .into_response();
            }
        }
    }

    let mut condition = Condition::all().add(AnnouncementColumn::ModuleId.eq(module_id));

    if let Some(ref query) = params.query {
        let pattern = format!("%{}%", query.to_lowercase());
        condition = condition.add(
            Condition::any()
                .add(AnnouncementColumn::Title.contains(&pattern))
                .add(AnnouncementColumn::Body.contains(&pattern)),
        );
    }

    if let Some(ref pinned) = params.pinned {
        match pinned.parse::<bool>() {
            Ok(pinned_bool) => {
                condition = condition.add(AnnouncementColumn::Pinned.eq(pinned_bool));
            }
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::<FilterResponse>::error("Invalid pinned value")),
                )
                .into_response();
            }
        }
    }

    let mut query = AnnouncementEntity::find().filter(condition);

    let mut applied_pinned_sort = false;

    if let Some(sort_param) = &params.sort {
        for sort in sort_param.split(',') {
            let (field, asc) = if sort.starts_with('-') {
                (&sort[1..], false)
            } else {
                (sort, true)
            };

            query = match field {
                "created_at" => {
                    if asc {
                        query.order_by_asc(AnnouncementColumn::CreatedAt)
                    } else {
                        query.order_by_desc(AnnouncementColumn::CreatedAt)
                    }
                }
                "updated_at" => {
                    if asc {
                        query.order_by_asc(AnnouncementColumn::UpdatedAt)
                    } else {
                        query.order_by_desc(AnnouncementColumn::UpdatedAt)
                    }
                }
                "title" => {
                    if asc {
                        query.order_by_asc(AnnouncementColumn::Title)
                    } else {
                        query.order_by_desc(AnnouncementColumn::Title)
                    }
                }
                "pinned" => {
                    applied_pinned_sort = true;
                    if asc {
                        query.order_by_asc(AnnouncementColumn::Pinned)
                    } else {
                        query.order_by_desc(AnnouncementColumn::Pinned)
                    }
                }
                _ => query,
            };
        }
    }

    // Default pinned DESC if not explicitly sorted by pinned
    if !applied_pinned_sort {
        query = query.order_by_desc(AnnouncementColumn::Pinned).order_by_desc(AnnouncementColumn::CreatedAt);
    }

    let paginator = query.clone().paginate(db, per_page as u64);
    let total = match paginator.num_items().await {
        Ok(n) => n as i32,
        Err(e) => {
            eprintln!("Error counting announcements: {:?}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<FilterResponse>::error("Error counting announcements")),
            )
            .into_response();
        }
    };

    match paginator.fetch_page((page - 1) as u64).await {
        Ok(results) => {
            let response = FilterResponse::new(results, page, per_page, total);
            (
                StatusCode::OK,
                Json(ApiResponse::success(
                    response,
                    "Announcements retrieved successfully",
                )),
            )
            .into_response()
        }
        Err(err) => {
            eprintln!("Error fetching announcements: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<FilterResponse>::error(
                    "Failed to retrieve announcements",
                )),
            )
            .into_response()
        }
    }
}

/// GET /api/modules/{module_id}/announcements/{announcement_id}
///
/// Retrieves a single announcement by ID for the specified module, including the **authoring user**.
///
/// # Path Parameters
///
/// - `module_id`: The module the announcement belongs to.
/// - `announcement_id`: The announcement ID to fetch.
///
/// # Behavior
///
/// - Verifies the announcement belongs to the given `module_id`.
/// - Eager-loads the related user (author) via the `belongs_to User` relation.
/// - Returns `404 NOT FOUND` if no matching announcement is found.
///
/// # Returns
///
/// - `200 OK` with `{ announcement, user }` on success (user is `{ id, username }` only).
/// - `404 NOT FOUND` if the announcement does not exist (or doesn’t belong to the module).
/// - `500 INTERNAL SERVER ERROR` on database errors.
///
/// # Example Responses
///
/// **200 OK**
/// ```json
/// {
///   "success": true,
///   "data": {
///     "announcement": {
///       "id": 42,
///       "module_id": 101,
///       "user_id": 5,
///       "title": "Important update",
///       "body": "Please note the following changes...",
//* pinned/created_at/updated_at omitted for brevity in this snippet
///       "pinned": true,
///       "created_at": "2025-08-16T12:00:00Z",
///       "updated_at": "2025-08-16T12:15:00Z"
///     },
///     "user": { "id": 5, "username": "lecturer" }
///   },
///   "message": "Announcement retrieved successfully"
/// }
/// ```
///
/// **404 NOT FOUND**
/// ```json
/// { "success": false, "message": "Announcement not found" }
/// ```
///
/// **500 INTERNAL SERVER ERROR**
/// ```json
/// { "success": false, "message": "Failed to retrieve announcement" }
/// ```
pub async fn get_announcement(
    State(app_state): State<AppState>,
    Path((module_id, announcement_id)): Path<(i64, i64)>,
) -> impl IntoResponse {
    let db = app_state.db();

    let result = AnnouncementEntity::find_by_id(announcement_id)
        .filter(AnnouncementColumn::ModuleId.eq(module_id))
        .find_also_related(UserEntity)
        .one(db)
        .await;

    match result {
        Ok(Some((announcement, Some(user)))) => {
            let thin = MinimalUser {
                id: user.id,
                username: user.username,
            };
            (
                StatusCode::OK,
                Json(ApiResponse::success(
                    ShowAnnouncementResponse {
                        announcement,
                        user: thin,
                    },
                    "Announcement retrieved successfully",
                )),
            )
                .into_response()
        }

        Ok(Some((_announcement, None))) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<ShowAnnouncementResponse>::error(
                "Related user not found for announcement",
            )),
        )
            .into_response(),

        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<ShowAnnouncementResponse>::error(
                "Announcement not found",
            )),
        )
            .into_response(),

        Err(err) => {
            eprintln!("Error fetching announcement: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<ShowAnnouncementResponse>::error(
                    "Failed to retrieve announcement",
                )),
            )
                .into_response()
        }
    }
}