use axum::{
    Extension, Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use db::models::{
    assignment, module, tickets, user,
    user_module_role::{self},
};
use migration::Expr;
use sea_orm::{
    ColumnTrait, Condition, EntityTrait, JoinType, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, RelationTrait
};
use serde::{Deserialize, Serialize};
use util::state::AppState;

use crate::{auth::AuthUser, response::ApiResponse};

#[derive(Debug, Deserialize)]
pub struct FilterReq {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub query: Option<String>,
    pub role: Option<String>,
    pub year: Option<i32>,
    pub status: Option<String>,
    pub sort: Option<String>,
}

#[derive(Serialize)]
pub struct FilterResponse {
    pub tickets: Vec<TicketsResponse>,
    pub page: i32,
    pub per_page: i32,
    pub total: i32,
}

impl FilterResponse {
    fn new(tickets: Vec<TicketsResponse>, page: i32, per_page: i32, total: i32) -> Self {
        Self {
            tickets,
            page,
            per_page,
            total,
        }
    }
}

#[derive(Serialize)]
pub struct TicketsResponse {
    pub id: i64,
    pub title: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub module: ModuleResponse,
    pub assignment: AssignmentResponse,
    pub user: UserResponse,
}

#[derive(Serialize)]
pub struct UserResponse {
    pub id: i64,
    pub username: String,
}

#[derive(Serialize)]
pub struct AssignmentResponse {
    pub id: i64,
    pub name: String,
}

#[derive(Serialize)]
pub struct ModuleResponse {
    pub id: i64,
    pub code: String,
}

pub async fn get_my_tickets(
    State(state): State<AppState>,
    Extension(AuthUser(claims)): Extension<AuthUser>,
    Query(params): Query<FilterReq>,
) -> impl IntoResponse {
    let db = state.db();
    let user_id = claims.sub;

    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).min(100);

    let requested_role = params.role.clone().unwrap_or_else(|| "student".to_string());

    let memberships = user_module_role::Entity::find()
        .filter(user_module_role::Column::UserId.eq(user_id))
        .filter(user_module_role::Column::Role.eq(requested_role.clone()))
        .all(db)
        .await
        .unwrap_or_default();

    if memberships.is_empty() {
        let response = FilterResponse::new(vec![], page, per_page, 0);
        return (
            StatusCode::OK,
            Json(ApiResponse::success(response, "Tickets retrieved successfully")),
        )
            .into_response();
    }

    let module_ids: Vec<i64> = memberships.iter().map(|m| m.module_id).collect();

    let assignments = assignment::Entity::find()
        .filter(assignment::Column::ModuleId.is_in(module_ids.clone()))
        .all(db)
        .await
        .unwrap_or_default();

    if assignments.is_empty() {
        let response = FilterResponse::new(vec![], page, per_page, 0);
        return (
            StatusCode::OK,
            Json(ApiResponse::success(response, "Tickets retrieved successfully")),
        )
            .into_response();
    }

    let assignment_ids: Vec<i64> = assignments.iter().map(|a| a.id).collect();

    let mut condition = Condition::all()
        .add(tickets::Column::AssignmentId.is_in(assignment_ids.clone()));

    if requested_role == "student" {
        condition = condition.add(tickets::Column::UserId.eq(user_id));
    } else {
        condition = condition.add(tickets::Column::UserId.ne(user_id));
    }

    if let Some(year) = params.year {
        condition = condition.add(Expr::col((module::Entity, module::Column::Year)).eq(year));
    }

    if let Some(ref q) = params.query {
        let pattern = format!("%{}%", q.to_lowercase());
        condition = condition.add(
            Condition::any()
                .add(Expr::cust("LOWER(tickets.title)").like(&pattern))
                .add(Expr::cust("LOWER(module.code)").like(&pattern))
                .add(Expr::cust("LOWER(assignment.name)").like(&pattern)),
        );
        if requested_role != "student" {
            condition = condition.add(Expr::cust("LOWER(user.username)").like(&pattern));
        }
    }

    if let Some(ref s) = params.status {
        match s.parse::<tickets::TicketStatus>() {
            Ok(st) => condition = condition.add(tickets::Column::Status.eq(st)),
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::<FilterResponse>::error("Invalid status value")),
                )
                    .into_response();
            }
        }
    }

    let mut query = tickets::Entity::find()
        .join(JoinType::InnerJoin, tickets::Relation::Assignment.def())
        .join(JoinType::InnerJoin, assignment::Relation::Module.def())
        .filter(condition);

    if requested_role != "student" {
        query = query.join(JoinType::InnerJoin, tickets::Relation::User.def());
    }

    if let Some(sort_param) = &params.sort {
        for sort in sort_param.split(',') {
            let (field, asc) = if sort.starts_with('-') {
                (&sort[1..], false)
            } else {
                (sort, true)
            };
            query = match field {
                "created_at" => {
                    if asc { query.order_by_asc(tickets::Column::CreatedAt) }
                    else { query.order_by_desc(tickets::Column::CreatedAt) }
                }
                _ => query,
            };
        }
    } else {
        query = query
            .order_by_desc(tickets::Column::CreatedAt)
            .order_by_asc(tickets::Column::Id);
    }

    let paginator = query.clone().paginate(db, per_page as u64);
    let total = match paginator.num_items().await {
        Ok(n) => n as i32,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<FilterResponse>::error("Error counting tickets")),
            )
                .into_response();
        }
    };

    match paginator.fetch_page((page - 1) as u64).await {
        Ok(results) => {
            let mut tickets_vec = Vec::new();
            for t in results {
                let a = assignment::Entity::find_by_id(t.assignment_id)
                    .one(db)
                    .await
                    .unwrap_or(None);
                if a.is_none() { continue; }
                let a = a.unwrap();

                let m = module::Entity::find_by_id(a.module_id)
                    .one(db)
                    .await
                    .unwrap_or(None);
                if m.is_none() { continue; }
                let m = m.unwrap();

                let u = user::Entity::find_by_id(t.user_id)
                    .one(db)
                    .await
                    .unwrap_or(None);

                tickets_vec.push(TicketsResponse {
                    id: t.id,
                    title: t.title,
                    status: t.status.to_string(),
                    created_at: t.created_at.to_string(),
                    updated_at: t.updated_at.to_string(),
                    module: ModuleResponse { id: m.id, code: m.code },
                    assignment: AssignmentResponse { id: a.id, name: a.name },
                    user: UserResponse {
                        id: t.user_id,
                        username: u.map(|uu| uu.username).unwrap_or_default(),
                    },
                });
            }

            let response = FilterResponse::new(tickets_vec, page, per_page, total);
            (
                StatusCode::OK,
                Json(ApiResponse::success(response, "Tickets retrieved successfully")),
            )
                .into_response()
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<FilterResponse>::error("Failed to retrieve tickets")),
        )
            .into_response(),
    }
}
