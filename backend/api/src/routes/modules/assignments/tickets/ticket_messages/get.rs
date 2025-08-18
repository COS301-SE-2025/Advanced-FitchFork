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
        common::is_valid, ticket_messages::common::{MessageResponse, UserResponse},
    },
};
use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};

use db::models::{
    ticket_messages::{Column as TicketMessageColumn, Entity as TicketMessageEntity},
    user::Entity as UserEntity,
};
use migration::Expr;
use sea_orm::{ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use util::state::AppState;

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

/// Retrieves messages for a ticket.
///
/// **Endpoint:** `GET /modules/{module_id}/assignments/{assignment_id}/tickets/{ticket_id}/messages`  
/// **Permissions:** Only users with access to the ticket can retrieve messages.
///
/// ### Path parameters
/// - `module_id`       → ID of the module (used for permission check)
/// - `assignment_id`   → ID of the assignment (unused in handler, kept for route consistency)
/// - `ticket_id`       → ID of the ticket whose messages are being retrieved
///
/// ### Query parameters
/// - `page`            → Page number for pagination (default: 1)
/// - `per_page`        → Number of messages per page (default: 50, max: 100)
/// - `query`           → Optional search string to filter messages by content
///
/// ### Responses
/// - `200 OK` → Messages retrieved successfully
/// ```json
/// {
///   "success": true,
///   "data": {
///     "tickets": [/* array of messages */],
///     "page": 1,
///     "per_page": 50,
///     "total": 123
///   },
///   "message": "Messages retrieved successfully"
/// }
/// ```
/// - `403 Forbidden` → User does not have permission to view this ticket
/// ```json
/// {
///   "success": false,
///   "data": null,
///   "message": "Forbidden"
/// }
/// ```
/// - `500 Internal Server Error` → Failed to retrieve messages
/// ```json
/// {
///   "success": false,
///   "data": null,
///   "message": "Failed to retrieve tickets"
/// }
/// ```
pub async fn get_ticket_messages(
    Path((module_id, _, ticket_id)): Path<(i64, i64, i64)>,
    State(app_state): State<AppState>,
    Extension(AuthUser(claims)): Extension<AuthUser>,
    Query(params): Query<FilterReq>,
) -> impl IntoResponse {
    let user_id: i64 = claims.sub;
    let db = app_state.db();

    if !is_valid(user_id, ticket_id, module_id, db).await {
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
                    user: user.map(|u| {
                        UserResponse {
                            id: u.id,
                            username: u.username,
                        }
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
