use axum::{
    Extension, Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use db::models::{announcements, module, user, user_module_role};
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
    pub pinned: Option<bool>,
    pub sort: Option<String>,
}

#[derive(Serialize)]
pub struct AnnouncementResponse {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub pinned: bool,
    pub created_at: String,
    pub updated_at: String,
    pub module: ModuleResponse,
    pub user: UserResponse,
}

#[derive(Serialize)]
pub struct UserResponse {
    pub id: i64,
    pub username: String,
}

#[derive(Serialize)]
pub struct ModuleResponse {
    pub id: i64,
    pub code: String,
}

#[derive(Serialize)]
pub struct FilterResponse {
    pub announcements: Vec<AnnouncementResponse>,
    pub page: i32,
    pub per_page: i32,
    pub total: i32,
}

impl FilterResponse {
    fn new(announcements: Vec<AnnouncementResponse>, page: i32, per_page: i32, total: i32) -> Self {
        Self { announcements, page, per_page, total }
    }
}

pub async fn get_my_announcements(
    State(state): State<AppState>,
    Extension(AuthUser(claims)): Extension<AuthUser>,
    Query(params): Query<FilterReq>,
) -> impl IntoResponse {
    let db = state.db();
    let user_id = claims.sub;
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).min(100);

    let allowed_roles = vec!["lecturer", "assistant_lecturer", "tutor", "student"];
    let requested_role = params.role.clone().filter(|r| allowed_roles.contains(&r.as_str()));

    let memberships = user_module_role::Entity::find()
        .filter(user_module_role::Column::UserId.eq(user_id))
        .filter(user_module_role::Column::Role.is_in(allowed_roles.clone()))
        .all(db)
        .await
        .unwrap_or_default();

    if memberships.is_empty() {
        let response = FilterResponse::new(vec![], page, per_page, 0);
        return (StatusCode::OK, Json(ApiResponse::success(response, "Announcements retrieved"))).into_response();
    }

    let module_ids: Vec<i64> = memberships.iter()
        .filter(|m| requested_role.as_ref().map_or(true, |r| &m.role.to_string() == r))
        .map(|m| m.module_id)
        .collect();

    if module_ids.is_empty() {
        let response = FilterResponse::new(vec![], page, per_page, 0);
        return (StatusCode::OK, Json(ApiResponse::success(response, "Announcements retrieved"))).into_response();
    }

    let mut condition = Condition::all().add(announcements::Column::ModuleId.is_in(module_ids));

    if let Some(year) = params.year {
        condition = condition.add(Expr::col((module::Entity, module::Column::Year)).eq(year));
    }

    if let Some(pinned) = params.pinned {
        condition = condition.add(announcements::Column::Pinned.eq(pinned));
    }

    if let Some(ref q) = params.query {
        let pattern = format!("%{}%", q.to_lowercase());
        condition = condition.add(
            Condition::any()
                .add(Expr::cust("LOWER(announcements.title)").like(&pattern))
                .add(Expr::cust("LOWER(module.code)").like(&pattern))
                .add(Expr::cust("LOWER(user.username)").like(&pattern))
        );
    }

    let mut query = announcements::Entity::find()
        .join(JoinType::InnerJoin, announcements::Relation::Module.def())
        .join(JoinType::InnerJoin, announcements::Relation::User.def())
        .filter(condition);

    if let Some(sort_param) = &params.sort {
        for sort in sort_param.split(',') {
            let (field, asc) = if sort.starts_with('-') { (&sort[1..], false) } else { (sort, true) };
            query = match field {
                "created_at" => if asc { query.order_by_asc(announcements::Column::CreatedAt) } else { query.order_by_desc(announcements::Column::CreatedAt) },
                "updated_at" => if asc { query.order_by_asc(announcements::Column::UpdatedAt) } else { query.order_by_desc(announcements::Column::UpdatedAt) },
                _ => query,
            };
        }
    } else {
        query = query.order_by_desc(announcements::Column::CreatedAt).order_by_asc(announcements::Column::Id);
    }

    let paginator = query.clone().paginate(db, per_page as u64);
    let total = match paginator.num_items().await {
        Ok(n) => n as i32,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<FilterResponse>::error("Error counting announcements"))).into_response(),
    };

    match paginator.fetch_page((page - 1) as u64).await {
        Ok(results) => {
            let mut announcements_vec = Vec::new();
            for a in results {
                let m = module::Entity::find_by_id(a.module_id).one(db).await.unwrap_or(None);
                if m.is_none() { continue; }
                let m = m.unwrap();

                let u = user::Entity::find_by_id(a.user_id).one(db).await.unwrap_or(None);

                announcements_vec.push(AnnouncementResponse {
                    id: a.id,
                    title: a.title,
                    content: a.body,
                    pinned: a.pinned,
                    created_at: a.created_at.to_string(),
                    updated_at: a.updated_at.to_string(),
                    module: ModuleResponse { id: m.id, code: m.code },
                    user: UserResponse { id: a.user_id, username: u.map(|uu| uu.username).unwrap_or_default() },
                });
            }

            let response = FilterResponse::new(announcements_vec, page, per_page, total);
            (StatusCode::OK, Json(ApiResponse::success(response, "Announcements retrieved"))).into_response()
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<FilterResponse>::error("Failed to retrieve announcements"))).into_response(),
    }
}
