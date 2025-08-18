//! Ticket message creation handler.
//!
//! Provides an endpoint to create a new message for a ticket in a module.
//!
//! Only users authorized to view the ticket (author or staff) can create messages.

use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use db::models::{
    ticket_messages::Model as TicketMessageModel,
    user::{Column as UserColumn, Entity as UserEntity, Model as UserModel},
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use util::state::AppState;

use crate::{
    auth::AuthUser,
    response::ApiResponse,
    routes::modules::assignments::tickets::{
        common::is_valid,
        ticket_messages::common::{MessageResponse, UserResponse},
    },
};

/// Creates a new message for a ticket.
///
/// **Endpoint:** `POST /modules/{module_id}/assignments/{assignment_id}/tickets/{ticket_id}/messages`  
/// **Permissions:** User must be authorized to view the ticket (author or staff).
///
/// ### Path parameters
/// - `module_id`       → ID of the module containing the ticket
/// - `assignment_id`   → ID of the assignment (unused in handler, for route consistency)
/// - `ticket_id`       → ID of the ticket
///
/// ### Request body
/// ```json
/// {
///   "content": "Message content here"
/// }
/// ```
///
/// ### Responses
/// - `200 OK` → Message created successfully
/// ```json
/// {
///   "success": true,
///   "data": {
///       "id": 123,
///       "ticket_id": 456,
///       "content": "Message content here",
///       "created_at": "2025-08-18T10:00:00Z",
///       "updated_at": "2025-08-18T10:00:00Z",
///       "user": { "id": 789, "username": "john_doe" }
///   },
///   "message": "Message created successfully"
/// }
/// ```
/// - `400 Bad Request` → Content missing or empty
/// ```json
/// {
///   "success": false,
///   "data": null,
///   "message": "Content is required"
/// }
/// ```
/// - `403 Forbidden` → User not authorized to create a message for this ticket
/// ```json
/// {
///   "success": false,
///   "data": null,
///   "message": "Forbidden"
/// }
/// ```
/// - `404 Not Found` → User not found
/// ```json
/// {
///   "success": false,
///   "data": null,
///   "message": "User not found"
/// }
/// ```
/// - `500 Internal Server Error` → Failed to create the message
/// ```json
/// {
///   "success": false,
///   "data": null,
///   "message": "Failed to create message"
/// }
/// ```
pub async fn create_message(
    Path((module_id, _assignment_id, ticket_id)): Path<(i64, i64, i64)>,
    State(app_state): State<AppState>,
    Extension(AuthUser(claims)): Extension<AuthUser>,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    let db = app_state.db();
    let user_id = claims.sub;

    if !is_valid(user_id, ticket_id, module_id, db).await {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<()>::error("Forbidden")),
        )
            .into_response();
    }

    let content = match req.get("content").and_then(|v| v.as_str()) {
        Some(c) if !c.trim().is_empty() => c.trim().to_string(),
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<()>::error("Content is required")),
            )
                .into_response();
        }
    };

    let user: Option<UserModel> = UserEntity::find()
        .filter(UserColumn::Id.eq(user_id))
        .one(db)
        .await
        .unwrap_or(None);

    let user = match user {
        Some(u) => u,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<()>::error("User not found")),
            )
                .into_response();
        }
    };

    let message = match TicketMessageModel::create(db, ticket_id, user_id, &content).await {
        Ok(msg) => msg,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("Failed to create message")),
            )
                .into_response();
        }
    };

    let response = MessageResponse {
        id: message.id,
        ticket_id: message.ticket_id,
        content: message.content,
        created_at: message.created_at.to_rfc3339(),
        updated_at: message.updated_at.to_rfc3339(),
        user: Some(UserResponse {
            id: user.id,
            username: user.username,
        }),
    };

    (
        StatusCode::OK,
        Json(ApiResponse::success(
            response,
            "Message created successfully",
        )),
    )
        .into_response()
}
