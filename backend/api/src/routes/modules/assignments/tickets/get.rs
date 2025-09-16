//! Ticket retrieval handlers.
//!
//! Provides endpoints to fetch tickets for an assignment.
//!
//! Users can retrieve a single ticket or a list of tickets, with support for
//! filtering, sorting, and pagination. The endpoints validate that the user
//! has permission to view the ticket(s) before returning data.

use crate::{
    auth::AuthUser, response::ApiResponse,
    routes::modules::assignments::tickets::common::{is_valid, TicketResponse, TicketWithUserResponse},
};
use axum::{
    Extension,
    extract::{Path, Query},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use util::filters::{FilterParam, QueryParam};
use services::service::Service;
use services::ticket::{TicketService, TicketStatus};
use services::user_module_role::UserModuleRoleService;

/// GET /api/modules/{module_id}/assignments/{assignment_id}/tickets/{ticket_id}
///
/// Retrieve a specific ticket along with information about the user who created it.
/// Accessible to users assigned to the module (e.g., student, tutor, lecturer).
///
/// ### Path Parameters
/// - `module_id` (i64): The ID of the module containing the assignment
/// - `assignment_id` (i64): The ID of the assignment containing the ticket
/// - `ticket_id` (i64): The ID of the ticket to retrieve
///
/// ### Responses
///
/// - `200 OK`
/// ```json
/// {
///   "success": true,
///   "message": "Ticket with user retrieved",
///   "data": {
///     "ticket": {
///       "id": 101,
///       "assignment_id": 456,
///       "user_id": 789,
///       "title": "Issue with question 2",
///       "description": "I'm not sure what the question is asking.",
///       "status": "open",
///       "created_at": "2025-08-01T12:00:00Z",
///       "updated_at": "2025-08-01T12:30:00Z"
///     },
///     "user": {
///       "id": 789,
///       "username": "u23571561",
///       "email": "student@example.com",
///       "profile_picture_path": "uploads/users/789/profile.png"
///     }
///   }
/// }
/// ```
///
/// - `403 Forbidden`
/// ```json
/// {
///   "success": false,
///   "message": "Forbidden"
/// }
/// ```
///
/// - `404 Not Found`
/// ```json
/// {
///   "success": false,
///   "message": "Ticket not found"
/// }
/// ```
///
/// - `500 Internal Server Error`
/// ```json
/// {
///   "success": false,
///   "message": "Failed to retrieve ticket"
/// }
/// ```
pub async fn get_ticket(
    Path((module_id, _, ticket_id)): Path<(i64, i64, i64)>,
    Extension(AuthUser(claims)): Extension<AuthUser>,
) -> impl IntoResponse {
    let user_id = claims.sub;

    if !is_valid(user_id, ticket_id, module_id, claims.admin).await {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<()>::error("Forbidden")),
        ).into_response();
    }

    // Fetch ticket and preload the user relation
    match TicketEntity::find_by_id(ticket_id)
        .find_also_related(UserEntity)
        .one(db)
        .await
    {
        Ok(Some((ticket, Some(user)))) => {
            let response = TicketWithUserResponse {
                ticket: ticket.into(),
                user: user.into(),
            };
            (
                StatusCode::OK,
                Json(ApiResponse::success(response, "Ticket with user retrieved")),
            ).into_response()
        }
        Ok(Some((_ticket, None))) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error("User not found")),
        ).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Ticket not found")),
        ).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error("Failed to retrieve ticket")),
        ).into_response(),
    }
}

/// Query parameters for filtering, sorting, and pagination
#[derive(Debug, Deserialize)]
pub struct FilterReq {
    /// Page number (default: 1)
    pub page: Option<u64>,
    /// Items per page (default: 20, max: 100)
    pub per_page: Option<u64>,
    /// Search query (matches title or description)
    pub query: Option<String>,
    /// Filter by ticket status
    pub status: Option<String>,
    /// Sort by fields (e.g., "created_at,-status")
    pub sort: Option<String>,
}

/// Response for a paginated list of tickets
#[derive(Serialize)]
pub struct FilterResponse {
    pub tickets: Vec<TicketResponse>,
    pub page: u64,
    pub per_page: u64,
    pub total: u64,
}

impl FilterResponse {
    fn new(tickets: Vec<TicketResponse>, page: u64, per_page: u64, total: u64) -> Self {
        Self {
            tickets,
            page,
            per_page,
            total,
        }
    }
}

/// Retrieves tickets for an assignment with optional filtering, sorting, and pagination.
///
/// **Endpoint:** `GET /modules/{module_id}/assignments/{assignment_id}/tickets`  
/// **Permissions:**  
/// - Students can only see their own tickets  
/// - Lecturers/assistants can see all tickets
///
/// ### Path parameters
/// - `module_id`       → ID of the module (used for permission check)
/// - `assignment_id`   → ID of the assignment
///
/// ### Query parameters
/// - `page` → Page number (default: 1)
/// - `per_page` → Number of items per page (default: 20, max: 100)
/// - `query` → Search in ticket title or description
/// - `status` → Filter by ticket status (`open`, `closed`)
/// - `sort` → Comma-separated fields to sort by (prefix with `-` for descending)
///
/// ### Responses
/// - `200 OK` → Tickets retrieved successfully
/// ```json
/// {
///   "success": true,
///   "data": {
///     "tickets": [ /* Ticket objects */ ],
///     "page": 1,
///     "per_page": 20,
///     "total": 42
///   },
///   "message": "Tickets retrieved successfully"
/// }
/// ```
/// - `400 Bad Request` → Invalid query parameters (sort or status)
/// ```json
/// {
///   "success": false,
///   "data": null,
///   "message": "Invalid field used"
/// }
/// ```
/// - `500 Internal Server Error` → Failed to fetch tickets
/// ```json
/// {
///   "success": false,
///   "data": null,
///   "message": "Failed to retrieve tickets"
/// }
/// ```
pub async fn get_tickets(
    Path((module_id, assignment_id)): Path<(i64, i64)>,
    Extension(AuthUser(claims)): Extension<AuthUser>,
    Query(params): Query<FilterReq>,
) -> impl IntoResponse {
    let user_id = claims.sub;
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).min(100);
    let sort = params.sort.clone();

    if let Some(sort_str) = &sort {
        let valid_fields = ["created_at", "updated_at", "status"];
        for field in sort_str.split(',') {
            let field = field.trim().trim_start_matches('-');
            if !valid_fields.contains(&field) {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::<FilterResponse>::error("Invalid field used")),
                );
            }
        }
    }

    let mut filters = vec![FilterParam::eq("assignment_id", assignment_id)];
    let mut queries = Vec::new();

    if !claims.admin {
        match UserModuleRoleService::find_one(
            &vec![
                FilterParam::eq("user_id", user_id),
                FilterParam::eq("module_id", module_id),
                FilterParam::eq("role", "student".to_string()),
            ],
            &vec![],
            None,
        ).await {
            Ok(Some(_)) => {
                filters.push(FilterParam::eq("user_id", user_id));
            },
            Ok(None) => {},
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<FilterResponse>::error(
                        format!("Database error: {}", e),
                    )),
                );
            }
        }
    };

    if let Some(query_text) = params.query {
        queries.push(QueryParam::new(
            vec!["title".to_string(), "description".to_string()],
            query_text,
        ));
    }

    if let Some(status_str) = params.status {
        match status_str.parse::<TicketStatus>() {
            Ok(status_enum) => {
                filters.push(FilterParam::eq("status", status_enum.to_string()));
            }
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::<FilterResponse>::error("Invalid status value")),
                );
            }
        }
    }

    let (tickets, total) = match TicketService::filter(
        &filters,
        &queries,
        page,
        per_page,
        sort,
    ).await {
        Ok(result) => result,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<FilterResponse>::error(
                    format!("Database error: {}", e),
                )),
            );
        }
    };

    let ticket_responses: Vec<TicketResponse> = tickets.into_iter().map(TicketResponse::from).collect();

    let response = FilterResponse::new(ticket_responses, page, per_page, total);
    (
        StatusCode::OK,
        Json(ApiResponse::success(
            response,
            "Tickets retrieved successfully",
        )),
    )
}