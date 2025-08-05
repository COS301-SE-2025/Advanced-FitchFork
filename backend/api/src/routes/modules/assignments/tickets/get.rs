use crate::{
    auth::AuthUser, response::ApiResponse,
    routes::modules::assignments::tickets::common::TicketResponse,
    routes::modules::assignments::tickets::common::is_valid,
};
use axum::{
    Extension,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use db::models::tickets::{
    Column as TicketColumn, Entity as TicketEntity, Model as TicketModel, TicketStatus,
};
use migration::Expr;
use sea_orm::{ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder};
use serde::{Deserialize, Serialize};
use util::state::AppState;

pub async fn get_ticket(
    State(app_state): State<AppState>,
    Path((_, _, ticket_id)): Path<(i64, i64, i64)>,
    Extension(AuthUser(claims)): Extension<AuthUser>,
) -> impl IntoResponse {
    let db = app_state.db();
    let user_id = claims.sub;

    if !is_valid(user_id, ticket_id, db).await {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<()>::error("Forbidden")),
        )
            .into_response();
    }

    match TicketModel::get_by_id(db, ticket_id).await {
        Ok(Some(ticket)) => (
            StatusCode::OK,
            Json(ApiResponse::<TicketResponse>::success(
                ticket.into(),
                "Ticket retrieved successfully",
            )),
        )
            .into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Ticket not found")),
        )
            .into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error("Failed to retrieve ticket")),
        )
            .into_response(),
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
pub async fn get_tickets(
    State(app_state): State<AppState>,
    Query(params): Query<FilterReq>,
) -> impl IntoResponse {
    let db = app_state.db();

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

    let mut condition = Condition::all();

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
