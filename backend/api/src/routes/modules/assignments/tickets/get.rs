use crate::{
    auth::AuthUser, response::ApiResponse,
    routes::modules::assignments::tickets::common::{is_valid, TicketResponse, TicketWithUserResponse},
};
use axum::{
    Extension,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use db::models::{tickets::{
    Column as TicketColumn, Entity as TicketEntity, TicketStatus,
}, user, user_module_role::{self, Role}};
use db::models::user::{Entity as UserEntity};
use migration::Expr;
use sea_orm::{ColumnTrait, Condition, DatabaseConnection, EntityTrait, JoinType, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, RelationTrait};
use serde::{Deserialize, Serialize};
use util::state::AppState;

/// GET /api/modules/{module_id}/assignments/{assignment_id}/tickets/{ticket_id}/with-user
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
    State(app_state): State<AppState>,
    Path((module_id, _, ticket_id)): Path<(i64, i64, i64)>,
    Extension(AuthUser(claims)): Extension<AuthUser>,
) -> impl IntoResponse {
    let db = app_state.db();
    let user_id = claims.sub;

    if !is_valid(user_id, ticket_id, module_id, claims.admin, db).await {
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

#[derive(Debug, Deserialize)]
pub struct FilterReq {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub query: Option<String>,
    pub status: Option<String>,
    pub sort: Option<String>,
}

#[derive(Serialize)]
pub struct FilterResponse {
    pub tickets: Vec<TicketResponse>,
    pub page: i32,
    pub per_page: i32,
    pub total: i32,
}

impl FilterResponse {
    fn new(tickets: Vec<TicketResponse>, page: i32, per_page: i32, total: i32) -> Self {
        Self {
            tickets,
            page,
            per_page,
            total,
        }
    }
}


async fn is_student(module_id: i64, user_id: i64, db: &DatabaseConnection) -> bool {
    user_module_role::Entity::find()
        .filter(user_module_role::Column::UserId.eq(user_id))
        .filter(user_module_role::Column::ModuleId.eq(module_id))
        .filter(user_module_role::Column::Role.eq(Role::Student))
        .join(JoinType::InnerJoin, user_module_role::Relation::User.def())
        .filter(user::Column::Admin.eq(false))
        .one(db)
        .await
        .map(|opt| opt.is_some())
        .unwrap_or(false)
}

pub async fn get_tickets(
    Path((module_id, assignment_id)): Path<(i64, i64)>,
    Extension(AuthUser(claims)): Extension<AuthUser>,
    State(app_state): State<AppState>,
    Query(params): Query<FilterReq>,
) -> impl IntoResponse {
    let db = app_state.db();
    let user_id = claims.sub;

    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).min(100);

    if let Some(sort_field) = &params.sort {
        let valid_fields = ["created_at", "updated_at", "status"];
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

    let mut condition = Condition::all().add(TicketColumn::AssignmentId.eq(assignment_id));

    if is_student(module_id, user_id, db).await {
        condition = condition.add(TicketColumn::UserId.eq(user_id));
    }

    if let Some(ref query) = params.query {
        let pattern = format!("%{}%", query.to_lowercase());
        condition = condition.add(
            Condition::any()
                .add(Expr::cust("LOWER(title)").like(&pattern))
                .add(Expr::cust("LOWER(description)").like(&pattern)),
        );
    }

    if let Some(ref status) = params.status {
        match status.parse::<TicketStatus>() {
            Ok(status_enum) => {
                condition = condition.add(TicketColumn::Status.eq(status_enum));
            }
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::<FilterResponse>::error("Invalid status value")),
                )
                    .into_response();
            }
        }
    }

    let mut query = TicketEntity::find().filter(condition);

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
                        query.order_by_asc(TicketColumn::CreatedAt)
                    } else {
                        query.order_by_desc(TicketColumn::CreatedAt)
                    }
                }
                "updated_at" => {
                    if asc {
                        query.order_by_asc(TicketColumn::UpdatedAt)
                    } else {
                        query.order_by_desc(TicketColumn::UpdatedAt)
                    }
                }
                "status" => {
                    if asc {
                        query.order_by_asc(TicketColumn::Status)
                    } else {
                        query.order_by_desc(TicketColumn::Status)
                    }
                }
                _ => query,
            };
        }
    }

    let paginator = query.clone().paginate(db, per_page as u64);
    let total = match paginator.num_items().await {
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

    match paginator.fetch_page((page - 1) as u64).await {
        Ok(results) => {
            let tickets: Vec<TicketResponse> =
                results.into_iter().map(TicketResponse::from).collect();

            let response = FilterResponse::new(tickets, page, per_page, total);
            (
                StatusCode::OK,
                Json(ApiResponse::success(
                    response,
                    "Tickets retrieved successfully",
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