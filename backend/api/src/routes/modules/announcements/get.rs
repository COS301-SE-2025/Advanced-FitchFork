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
