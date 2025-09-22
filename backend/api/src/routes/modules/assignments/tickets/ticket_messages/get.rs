//! Ticket messages retrieval handler.
//!
//! Provides an endpoint to retrieve messages for a specific ticket within an assignment.
//!
//! Only the user who has access to the ticket can view its messages. Supports pagination
//! and optional search query for filtering messages by content.

use crate::{
    auth::AuthUser,
    response::ApiResponse,
    routes::modules::assignments::tickets::{
        common::is_valid,
        ticket_messages::common::{MessageResponse, UserResponse},
    },
};
use axum::{
    Extension, Json,
    extract::{Path, Query},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct FilterReq {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub query: Option<String>,
}

#[derive(Serialize)]
pub struct FilterResponse {
    pub tickets: Vec<MessageResponse>,
    pub page: i32,
    pub per_page: i32,
    pub total: i32,
}

/// GET /api/modules/{module_id}/assignments/{assignment_id}/tickets/{ticket_id}/messages
///
/// Retrieve a paginated list of **ticket messages** for a specific ticket.  
/// Requires authentication and that the caller is allowed to view the ticket
/// (validated by `is_valid`, e.g., ticket participant/assigned staff for the module).
///
/// ### Path Parameters
/// - `module_id` (i64): ID of the module that owns the assignment/ticket
/// - `assignment_id` (i64): ID of the assignment (present in the route; not used in filtering here)
/// - `ticket_id` (i64): ID of the ticket whose messages are being fetched
///
/// ### Query Parameters
/// - `page` (optional, i32): Page number. Defaults to **1**. Minimum **1**
/// - `per_page` (optional, i32): Items per page. Defaults to **50**. Maximum **100**
/// - `query` (optional, string): Case-insensitive substring filter applied to message `content`
///
/// > **Note:** Sorting is not supported on this endpoint. Results are returned in the
/// database's default order for the query.
///
/// ### Responses
///
/// - `200 OK`
/// ```json
/// {
///   "success": true,
///   "message": "Messages retrieved successfully",
///   "data": {
///     "tickets": [
///       {
///         "id": 101,
///         "ticket_id": 99,
///         "content": "Hey, I'm blocked on step 3.",
///         "created_at": "2025-02-18T09:12:33Z",
///         "updated_at": "2025-02-18T09:12:33Z",
///         "user": {
///           "id": 12,
///           "username": "alice"
///         }
///       },
///       {
///         "id": 102,
///         "ticket_id": 99,
///         "content": "Try re-running with the latest config.",
///         "created_at": "2025-02-18T09:14:10Z",
///         "updated_at": "2025-02-18T09:14:10Z",
///         "user": {
///           "id": 8,
///           "username": "tutor_bob"
///         }
///       }
///     ],
///     "page": 1,
///     "per_page": 50,
///     "total": 2
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
/// - `500 Internal Server Error`
/// ```json
/// {
///   "success": false,
///   "message": "Error counting tickets"
/// }
/// ```
/// or
/// ```json
/// {
///   "success": false,
///   "message": "Failed to retrieve tickets"
/// }
/// ```
///
/// ### Example Request
/// ```http
/// GET /api/modules/42/assignments/7/tickets/99/messages?page=1&per_page=50&query=blocked
/// Authorization: Bearer <token>
/// ```
///
/// ### Example Success (200)
/// See the `200 OK` example above.
pub async fn get_ticket_messages(
    Path((module_id, _, ticket_id)): Path<(i64, i64, i64)>,
    Extension(AuthUser(claims)): Extension<AuthUser>,
    Query(params): Query<FilterReq>,
) -> impl IntoResponse {
    let user_id: i64 = claims.sub;

    if !is_valid(user_id, ticket_id, module_id, claims.admin).await {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<()>::error("Forbidden")),
        )
            .into_response();
    }

    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(50).min(100);
    let mut condition = Condition::all().add(TicketMessageColumn::TicketId.eq(ticket_id));

    if let Some(ref q) = params.query {
        let pattern = format!("%{}%", q.to_lowercase());
        condition =
            condition.add(Condition::any().add(Expr::cust("LOWER(content)").like(&pattern)));
    }

    let total = match TicketMessageEntity::find()
        .filter(condition.clone())
        .count(db)
        .await
    {
        Ok(n) => n as i32,
        Err(e) => {
            eprintln!("Error counting tickets: {:?}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<FilterResponse>::error(
                    "Error counting tickets",
                )),
            )
                .into_response();
        }
    };

    let paginator = TicketMessageEntity::find()
        .filter(condition)
        .find_also_related(UserEntity)
        .paginate(db, per_page as u64);
    match paginator.fetch_page((page - 1) as u64).await {
        Ok(results) => {
            let messages: Vec<MessageResponse> = results
                .into_iter()
                .map(|(message, user)| MessageResponse {
                    id: message.id,
                    ticket_id: message.ticket_id,
                    content: message.content,
                    created_at: message.created_at.to_rfc3339(),
                    updated_at: message.updated_at.to_rfc3339(),
                    user: user.map(|u| UserResponse {
                        id: u.id,
                        username: u.username,
                    }),
                })
                .collect();

            let response = FilterResponse {
                tickets: messages,
                page,
                per_page,
                total,
            };

            (
                StatusCode::OK,
                Json(ApiResponse::success(
                    response,
                    "Messages retrieved successfully",
                )),
            )
                .into_response()
        }
        Err(err) => {
            eprintln!("Error fetching tickets: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<FilterResponse>::error(
                    "Failed to retrieve tickets",
                )),
            )
                .into_response()
        }
    }
}
