use std::{collections::HashMap, str::FromStr};

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use chrono::{DateTime, Utc};
use db::models::{
    assignment,
    assignment_submission::{self, Column as SubmissionColumn, Entity as SubmissionEntity},
    module,
    plagiarism_case::{self, Entity as PlagiarismEntity, Status},
    user,
    user_module_role::{self, Role},
};
use sea_orm::{ColumnTrait, EntityTrait, Order, PaginatorTrait, QueryFilter, QueryOrder};
use serde::{Deserialize, Serialize};
use util::state::AppState;
use validator::Validate;

use crate::{auth::claims::AuthUser, response::ApiResponse};

#[derive(Debug, Deserialize, Validate)]
pub struct GetMyPlagiarismCasesQuery {
    #[validate(range(min = 1))]
    pub page: Option<u64>,
    #[validate(range(min = 1, max = 100))]
    pub per_page: Option<u64>,
    pub module_id: Option<i64>,
    pub assignment_id: Option<i64>,
    pub status: Option<String>,
    pub sort: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SubmissionSummary {
    pub submission_id: i64,
    pub user_id: i64,
    pub username: String,
}

#[derive(Debug, Serialize)]
pub struct AssignmentSummary {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct ModuleSummary {
    pub id: i64,
    pub code: String,
}

#[derive(Debug, Serialize)]
pub struct PlagiarismCaseSummary {
    pub id: i64,
    pub assignment: AssignmentSummary,
    pub module: ModuleSummary,
    pub status: String,
    pub similarity: f32,
    pub lines_matched: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub submission_1: SubmissionSummary,
    pub submission_2: SubmissionSummary,
}

#[derive(Debug, Serialize)]
pub struct GetMyPlagiarismCasesResponse {
    pub cases: Vec<PlagiarismCaseSummary>,
    pub page: u64,
    pub per_page: u64,
    pub total: u64,
}

pub async fn get_my_plagiarism_cases(
    State(state): State<AppState>,
    axum::Extension(user): axum::Extension<AuthUser>,
    Query(query): Query<GetMyPlagiarismCasesQuery>,
) -> impl IntoResponse {
    if let Err(e) = query.validate() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(e.to_string())),
        )
            .into_response();
    }

    let db = state.db();
    let user_id = user.0.sub;

    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);

    // Ensure caller has lecturer/assistant lecturer access
    let memberships = user_module_role::Entity::find()
        .filter(user_module_role::Column::UserId.eq(user_id))
        .filter(
            user_module_role::Column::Role.is_in(vec![Role::Lecturer, Role::AssistantLecturer]),
        )
        .all(db)
        .await
        .unwrap_or_default();

    if memberships.is_empty() {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<()>::error(
                "Only lecturers or assistant lecturers can view plagiarism cases",
            )),
        )
            .into_response();
    }

    let module_ids: Vec<i64> = memberships
        .iter()
        .filter(|m| {
            if let Some(module_filter) = query.module_id {
                m.module_id == module_filter
            } else {
                true
            }
        })
        .map(|m| m.module_id)
        .collect();

    if module_ids.is_empty() {
        let response = GetMyPlagiarismCasesResponse {
            cases: vec![],
            page,
            per_page,
            total: 0,
        };
        return (
            StatusCode::OK,
            Json(ApiResponse::success(response, "Plagiarism cases retrieved")),
        )
            .into_response();
    }

    let mut assignments_query = assignment::Entity::find()
        .filter(assignment::Column::ModuleId.is_in(module_ids.clone()));

    if let Some(assignment_id) = query.assignment_id {
        assignments_query = assignments_query.filter(assignment::Column::Id.eq(assignment_id));
    }

    let assignments = match assignments_query.all(db).await {
        Ok(list) => list,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(format!("Database error: {err}"))),
            )
                .into_response();
        }
    };

    if assignments.is_empty() {
        let response = GetMyPlagiarismCasesResponse {
            cases: vec![],
            page,
            per_page,
            total: 0,
        };
        return (
            StatusCode::OK,
            Json(ApiResponse::success(response, "Plagiarism cases retrieved")),
        )
            .into_response();
    }

    let assignment_ids: Vec<i64> = assignments.iter().map(|a| a.id).collect();
    let module_ids_for_assignments: Vec<i64> = assignments.iter().map(|a| a.module_id).collect();

    let modules = module::Entity::find()
        .filter(module::Column::Id.is_in(module_ids_for_assignments.clone()))
        .all(db)
        .await
        .unwrap_or_default();
    let module_map: HashMap<i64, module::Model> = modules.into_iter().map(|m| (m.id, m)).collect();
    let assignment_map: HashMap<i64, assignment::Model> = assignments
        .into_iter()
        .map(|a| (a.id, a))
        .collect();

    let mut cases_query = PlagiarismEntity::find()
        .filter(plagiarism_case::Column::AssignmentId.is_in(assignment_ids.clone()));

    if let Some(status) = &query.status {
        match Status::from_str(status) {
            Ok(status_enum) => {
                cases_query = cases_query.filter(plagiarism_case::Column::Status.eq(status_enum));
            }
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::<()>::error("Invalid status parameter")),
                )
                    .into_response();
            }
        }
    }

    if let Some(sort) = &query.sort {
        for item in sort.split(',') {
            let (order, key) = if let Some(stripped) = item.strip_prefix('-') {
                (Order::Desc, stripped)
            } else {
                (Order::Asc, item)
            };
            cases_query = match key {
                "created_at" => cases_query.order_by(plagiarism_case::Column::CreatedAt, order),
                "updated_at" => cases_query.order_by(plagiarism_case::Column::UpdatedAt, order),
                "similarity" => cases_query.order_by(plagiarism_case::Column::Similarity, order),
                "lines_matched" => cases_query.order_by(plagiarism_case::Column::LinesMatched, order),
                "status" => cases_query.order_by(plagiarism_case::Column::Status, order),
                _ => cases_query,
            };
        }
    } else {
        cases_query = cases_query.order_by_desc(plagiarism_case::Column::CreatedAt);
    }

    let paginator = cases_query.paginate(db, per_page);
    let total = paginator.num_items().await.unwrap_or(0);
    let cases = paginator.fetch_page(page - 1).await.unwrap_or_default();

    let submission_ids: Vec<i64> = cases
        .iter()
        .flat_map(|c| [c.submission_id_1, c.submission_id_2])
        .collect();

    let submissions = SubmissionEntity::find()
        .filter(SubmissionColumn::Id.is_in(submission_ids.clone()))
        .all(db)
        .await
        .unwrap_or_default();

    let user_ids: Vec<i64> = submissions.iter().map(|s| s.user_id).collect();
    let users = user::Entity::find()
        .filter(user::Column::Id.is_in(user_ids))
        .all(db)
        .await
        .unwrap_or_default();

    let user_map: HashMap<i64, user::Model> = users.into_iter().map(|u| (u.id, u)).collect();
    let submission_map: HashMap<i64, (assignment_submission::Model, user::Model)> = submissions
        .into_iter()
        .filter_map(|s| user_map.get(&s.user_id).cloned().map(|u| (s.id, (s, u))))
        .collect();

    let mut summaries: Vec<PlagiarismCaseSummary> = Vec::with_capacity(cases.len());

    for case in cases {
        let (submission1, user1) = match submission_map.get(&case.submission_id_1) {
            Some(data) => data.clone(),
            None => continue,
        };
        let (submission2, user2) = match submission_map.get(&case.submission_id_2) {
            Some(data) => data.clone(),
            None => continue,
        };

        let assignment = match assignment_map.get(&case.assignment_id) {
            Some(a) => a,
            None => continue,
        };

        let module = match module_map.get(&assignment.module_id) {
            Some(m) => m,
            None => continue,
        };

        summaries.push(PlagiarismCaseSummary {
            id: case.id,
            assignment: AssignmentSummary {
                id: assignment.id,
                name: assignment.name.clone(),
            },
            module: ModuleSummary {
                id: module.id,
                code: module.code.clone(),
            },
            status: case.status.to_string(),
            similarity: case.similarity,
            lines_matched: case.lines_matched,
            created_at: case.created_at,
            updated_at: case.updated_at,
            submission_1: SubmissionSummary {
                submission_id: submission1.id,
                user_id: submission1.user_id,
                username: user1.username,
            },
            submission_2: SubmissionSummary {
                submission_id: submission2.id,
                user_id: submission2.user_id,
                username: user2.username,
            },
        });
    }

    let response = GetMyPlagiarismCasesResponse {
        cases: summaries,
        page,
        per_page,
        total,
    };

    (
        StatusCode::OK,
        Json(ApiResponse::success(
            response,
            "Plagiarism cases retrieved successfully",
        )),
    )
        .into_response()
}
