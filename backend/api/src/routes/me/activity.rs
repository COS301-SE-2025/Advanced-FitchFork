use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use axum::{
    Extension, Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::{DateTime, Duration, SecondsFormat, Utc};
use db::models::{
    announcements, assignment, assignment_submission, module, user, user_module_role,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use serde::{Deserialize, Serialize};
use util::state::AppState;

use crate::{auth::AuthUser, response::ApiResponse};

#[derive(Debug, Deserialize)]
pub struct ActivityQuery {
    /// Page number (default: 1)
    pub page: Option<i32>,
    /// Items per page (default: 25, max: 100)
    pub per_page: Option<i32>,
    /// Only include activities for a specific module
    pub module_id: Option<i64>,
    /// Restrict activities to a specific role membership
    pub role: Option<String>,
    /// Comma-separated list of activity types to include (e.g. `announcement,submission`)
    pub types: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ActivityModule {
    pub id: i64,
    pub code: String,
    pub year: i32,
}

#[derive(Debug, Serialize, Clone)]
pub struct ActivityItem {
    pub id: String,
    pub activity_type: String,
    pub title: String,
    pub summary: String,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module: Option<ActivityModule>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ActivityResponse {
    pub activities: Vec<ActivityItem>,
    pub page: i32,
    pub per_page: i32,
    pub total: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ActivityKind {
    Announcement,
    AssignmentAvailable,
    AssignmentDue,
    Submission,
}

impl ActivityKind {
    fn as_str(&self) -> &'static str {
        match self {
            ActivityKind::Announcement => "announcement",
            ActivityKind::AssignmentAvailable => "assignment_available",
            ActivityKind::AssignmentDue => "assignment_due",
            ActivityKind::Submission => "submission",
        }
    }
}

impl FromStr for ActivityKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "announcement" => Ok(ActivityKind::Announcement),
            "assignment_available" | "assignment-open" | "assignment_open" => {
                Ok(ActivityKind::AssignmentAvailable)
            }
            "assignment_due" | "assignment-due" => Ok(ActivityKind::AssignmentDue),
            "submission" | "submissions" => Ok(ActivityKind::Submission),
            other => Err(format!("Unknown activity type '{other}'")),
        }
    }
}

struct ActivityEnvelope {
    sort_key: DateTime<Utc>,
    item: ActivityItem,
}

fn format_timestamp(ts: DateTime<Utc>) -> String {
    ts.to_rfc3339_opts(SecondsFormat::Secs, true)
}

fn format_display(ts: DateTime<Utc>) -> String {
    ts.format("%b %e, %Y %H:%M").to_string()
}

pub async fn get_my_activity(
    State(state): State<AppState>,
    Extension(AuthUser(claims)): Extension<AuthUser>,
    Query(params): Query<ActivityQuery>,
) -> impl IntoResponse {
    let db = state.db();
    let user_id = claims.sub;

    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(25).clamp(1, 100);

    let type_filter = if let Some(ref types) = params.types {
        let mut requested = HashSet::new();
        for raw in types.split(',') {
            if raw.trim().is_empty() {
                continue;
            }
            match ActivityKind::from_str(raw) {
                Ok(kind) => {
                    requested.insert(kind);
                }
                Err(msg) => {
                    return (StatusCode::BAD_REQUEST, Json(ApiResponse::<()>::error(msg)))
                        .into_response();
                }
            }
        }
        if requested.is_empty() {
            None
        } else {
            Some(requested)
        }
    } else {
        None
    };

    let include_kind = |kind: ActivityKind| {
        type_filter
            .as_ref()
            .map_or(true, |selected| selected.contains(&kind))
    };

    let requested_role = if let Some(role) = params.role.as_ref() {
        match user_module_role::Role::from_str(role) {
            Ok(role) => Some(role),
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::<()>::error("Invalid role filter")),
                )
                    .into_response();
            }
        }
    } else {
        None
    };

    let memberships = match user_module_role::Entity::find()
        .filter(user_module_role::Column::UserId.eq(user_id))
        .all(db)
        .await
    {
        Ok(records) => records,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(format!(
                    "Failed to load module memberships: {err}"
                ))),
            )
                .into_response();
        }
    };

    let mut module_ids: Vec<i64> = memberships
        .iter()
        .filter(|membership| {
            requested_role
                .as_ref()
                .map_or(true, |role| membership.role == *role)
        })
        .filter(|membership| {
            params
                .module_id
                .map_or(true, |module_id| membership.module_id == module_id)
        })
        .map(|membership| membership.module_id)
        .collect();

    if module_ids.is_empty() {
        let response = ActivityResponse {
            activities: Vec::new(),
            page,
            per_page,
            total: 0,
        };
        return (
            StatusCode::OK,
            Json(ApiResponse::success(response, "Activity retrieved")),
        )
            .into_response();
    }

    module_ids.sort_unstable();
    module_ids.dedup();

    let module_id_set: HashSet<i64> = module_ids.iter().copied().collect();

    let modules = match module::Entity::find()
        .filter(module::Column::Id.is_in(module_ids.clone()))
        .all(db)
        .await
    {
        Ok(records) => records,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(format!(
                    "Failed to load modules: {err}"
                ))),
            )
                .into_response();
        }
    };

    let module_map: HashMap<i64, module::Model> = modules.into_iter().map(|m| (m.id, m)).collect();

    let target_items = (page * per_page).max(per_page) as u64;
    let fetch_limit = (target_items + per_page as u64 * 3).min(500);

    let mut envelopes: Vec<ActivityEnvelope> = Vec::new();

    if include_kind(ActivityKind::Announcement) {
        match announcements::Entity::find()
            .filter(announcements::Column::ModuleId.is_in(module_ids.clone()))
            .order_by_desc(announcements::Column::CreatedAt)
            .limit(fetch_limit)
            .all(db)
            .await
        {
            Ok(records) => {
                let user_ids: Vec<i64> = records.iter().map(|a| a.user_id).collect();
                let user_map: HashMap<i64, user::Model> = if user_ids.is_empty() {
                    HashMap::new()
                } else {
                    match user::Entity::find()
                        .filter(user::Column::Id.is_in(user_ids.clone()))
                        .all(db)
                        .await
                    {
                        Ok(users) => users.into_iter().map(|u| (u.id, u)).collect(),
                        Err(err) => {
                            return (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                Json(ApiResponse::<()>::error(format!(
                                    "Failed to load announcement authors: {err}"
                                ))),
                            )
                                .into_response();
                        }
                    }
                };

                for announcement in records {
                    let Some(module) = module_map.get(&announcement.module_id) else {
                        continue;
                    };
                    let timestamp = announcement.created_at;
                    let author = user_map.get(&announcement.user_id);
                    let summary = match author {
                        Some(author) => format!("Announcement by {}", author.username),
                        None => format!("New announcement in {}", module.code),
                    };

                    envelopes.push(ActivityEnvelope {
                        sort_key: timestamp,
                        item: ActivityItem {
                            id: format!("announcement-{}", announcement.id),
                            activity_type: ActivityKind::Announcement.as_str().to_string(),
                            title: announcement.title.clone(),
                            summary,
                            timestamp: format_timestamp(timestamp),
                            module: Some(ActivityModule {
                                id: module.id,
                                code: module.code.clone(),
                                year: module.year,
                            }),
                            link: Some(format!("/modules/{}/announcements", module.id)),
                        },
                    });
                }
            }
            Err(err) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()>::error(format!(
                        "Failed to load announcements: {err}"
                    ))),
                )
                    .into_response();
            }
        }
    }

    if include_kind(ActivityKind::AssignmentAvailable) || include_kind(ActivityKind::AssignmentDue)
    {
        match assignment::Entity::find()
            .filter(assignment::Column::ModuleId.is_in(module_ids.clone()))
            .order_by_desc(assignment::Column::UpdatedAt)
            .limit(fetch_limit)
            .all(db)
            .await
        {
            Ok(assignments) => {
                for assignment in assignments {
                    let Some(module) = module_map.get(&assignment.module_id) else {
                        continue;
                    };

                    let now = Utc::now();
                    let future_cutoff = now + Duration::days(14);
                    let event = if include_kind(ActivityKind::AssignmentDue)
                        && now >= assignment.available_from
                    {
                        let ts = assignment.due_date;
                        (
                            ActivityKind::AssignmentDue,
                            ts,
                            format!("Due {}", format_display(ts)),
                        )
                    } else if include_kind(ActivityKind::AssignmentAvailable) {
                        let ts = assignment.available_from;
                        (
                            ActivityKind::AssignmentAvailable,
                            ts,
                            format!("Opens {}", format_display(ts)),
                        )
                    } else {
                        continue;
                    };

                    let (kind, timestamp, summary) = event;

                    if timestamp > future_cutoff {
                        continue;
                    }

                    envelopes.push(ActivityEnvelope {
                        sort_key: timestamp,
                        item: ActivityItem {
                            id: format!("assignment-{}-{}", assignment.id, kind.as_str()),
                            activity_type: kind.as_str().to_string(),
                            title: assignment.name.clone(),
                            summary,
                            timestamp: format_timestamp(timestamp),
                            module: Some(ActivityModule {
                                id: module.id,
                                code: module.code.clone(),
                                year: module.year,
                            }),
                            link: Some(format!(
                                "/modules/{}/assignments/{}",
                                module.id, assignment.id
                            )),
                        },
                    });
                }
            }
            Err(err) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()>::error(format!(
                        "Failed to load assignments: {err}"
                    ))),
                )
                    .into_response();
            }
        }
    }

    if include_kind(ActivityKind::Submission) {
        match assignment_submission::Entity::find()
            .filter(assignment_submission::Column::UserId.eq(user_id))
            .order_by_desc(assignment_submission::Column::CreatedAt)
            .limit(fetch_limit)
            .find_also_related(assignment::Entity)
            .all(db)
            .await
        {
            Ok(results) => {
                for (submission, assignment) in results {
                    let Some(assignment) = assignment else {
                        continue;
                    };

                    if !module_id_set.contains(&assignment.module_id) {
                        continue;
                    }

                    let Some(module) = module_map.get(&assignment.module_id) else {
                        continue;
                    };

                    let timestamp = submission.created_at;
                    let attempt_label = format!("Attempt {}", submission.attempt);
                    let score_label = if submission.total > 0.0 {
                        format!("Score {:.1}/{:.1}", submission.earned, submission.total)
                    } else {
                        submission.status.to_string()
                    };
                    let summary = format!("{} - {}", attempt_label, score_label);

                    envelopes.push(ActivityEnvelope {
                        sort_key: timestamp,
                        item: ActivityItem {
                            id: format!("submission-{}", submission.id),
                            activity_type: ActivityKind::Submission.as_str().to_string(),
                            title: assignment.name.clone(),
                            summary,
                            timestamp: format_timestamp(timestamp),
                            module: Some(ActivityModule {
                                id: module.id,
                                code: module.code.clone(),
                                year: module.year,
                            }),
                            link: Some(format!(
                                "/modules/{}/assignments/{}/submissions/{}",
                                module.id, assignment.id, submission.id
                            )),
                        },
                    });
                }
            }
            Err(err) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()>::error(format!(
                        "Failed to load submissions: {err}"
                    ))),
                )
                    .into_response();
            }
        }
    }

    envelopes.sort_by(|a, b| b.sort_key.cmp(&a.sort_key));
    let total = envelopes.len() as i32;

    let start = ((page - 1) * per_page).max(0) as usize;
    let end = usize::min(start + per_page as usize, envelopes.len());
    let activities = if start >= envelopes.len() {
        Vec::new()
    } else {
        envelopes[start..end]
            .iter()
            .map(|envelope| envelope.item.clone())
            .collect()
    };

    let response = ActivityResponse {
        activities,
        page,
        per_page,
        total,
    };

    (
        StatusCode::OK,
        Json(ApiResponse::success(response, "Activity retrieved")),
    )
        .into_response()
}
