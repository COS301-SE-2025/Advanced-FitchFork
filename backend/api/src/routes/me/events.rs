use crate::{auth::claims::AuthUser, response::ApiResponse};
use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use common::format_validation_errors;
use db::models::{assignment, module, user_module_role};
use sea_orm::{
    ColumnTrait, Condition, EntityTrait, JoinType, QueryFilter, QuerySelect, RelationTrait,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use util::state::AppState;
use validator::Validate;

/// Query parameters for filtering events.
#[derive(Debug, Deserialize, Validate)]
pub struct GetEventsQuery {
    /// Start date filter in ISO 8601 (`YYYY-MM-DD` or `YYYY-MM-DDTHH:mm:ss`)
    #[serde(default)]
    pub from: Option<String>,

    /// End date filter in ISO 8601 (`YYYY-MM-DD` or `YYYY-MM-DDTHH:mm:ss`)
    #[serde(default)]
    pub to: Option<String>,

    /// Optional role filter to scope modules
    #[serde(default)]
    pub role: Option<String>,

    /// Optional module filter
    #[serde(default)]
    pub module_id: Option<i64>,
}

/// Represents a single event item
#[derive(Debug, Serialize)]
pub struct EventItem {
    pub r#type: String,
    pub content: String,
    pub module_id: i64,
    pub assignment_id: i64,
}

/// Response structure for listing events
#[derive(Debug, Serialize)]
pub struct EventsResponse {
    pub events: HashMap<String, Vec<EventItem>>,
}

/// Normalize timestamp to ISO 8601 without fractional seconds
fn format_iso_datetime(dt: DateTime<Utc>) -> String {
    dt.format("%Y-%m-%dT%H:%M:%S").to_string()
}

/// Parse optional datetime string
fn parse_optional_datetime(s: Option<&str>) -> Result<Option<DateTime<Utc>>, String> {
    let Some(s) = s else {
        return Ok(None);
    };

    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Ok(Some(dt.with_timezone(&Utc)));
    }

    if let Ok(date) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        let datetime = date.and_time(NaiveTime::MIN).and_utc();
        return Ok(Some(datetime));
    }

    if let Ok(naive_dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S") {
        return Ok(Some(Utc.from_utc_datetime(&naive_dt)));
    }

    Err("Invalid date format. Use YYYY-MM-DD or YYYY-MM-DDTHH:mm:ss".to_string())
}

/// GET handler for `/api/me/events`
///
/// Retrieves calendar events for the authenticated user within an optional date range.
/// Events include assignment availability dates and due dates, grouped by their timestamps.
///
/// ### Query Parameters
/// - `from` (optional): Start date filter in ISO 8601 format (`YYYY-MM-DD` or `YYYY-MM-DDTHH:mm:ss`)
/// - `to` (optional): End date filter in ISO 8601 format (`YYYY-MM-DD` or `YYYY-MM-DDTHH:mm:ss`)
///
/// ### Response
/// - `200 OK`: Returns a map where:
///   - Keys are ISO 8601 timestamps without fractional seconds
///   - Values are arrays of events occurring at that timestamp
///   - Each event has a type ("warning" for availability, "error" for due dates)
///   - Event content describes the assignment name and event type
pub async fn get_my_events(
    State(state): State<AppState>,
    axum::Extension(user): axum::Extension<AuthUser>,
    Query(query): Query<GetEventsQuery>,
) -> impl IntoResponse {
    if let Err(e) = query.validate() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(format_validation_errors(&e))),
        )
            .into_response();
    }

    let user_id = user.0.sub;

    let from_dt = match parse_optional_datetime(query.from.as_deref()) {
        Ok(dt) => dt,
        Err(msg) => {
            return (StatusCode::BAD_REQUEST, Json(ApiResponse::<()>::error(msg))).into_response();
        }
    };

    let to_dt = match parse_optional_datetime(query.to.as_deref()) {
        Ok(dt) => dt,
        Err(msg) => {
            return (StatusCode::BAD_REQUEST, Json(ApiResponse::<()>::error(msg))).into_response();
        }
    };

    if let (Some(from), Some(to)) = (from_dt, to_dt) {
        if from > to {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<()>::error(
                    "'from' date must be before 'to' date".to_string(),
                )),
            )
                .into_response();
        }
    }

    let mut events_map: HashMap<String, Vec<EventItem>> = HashMap::new();

    let mut assignment_query = assignment::Entity::find()
        .join(JoinType::InnerJoin, assignment::Relation::Module.def())
        .join(JoinType::InnerJoin, module::Relation::UserModuleRole.def())
        .filter(user_module_role::Column::UserId.eq(user_id));

    if let Some(role_filter) = &query.role {
        assignment_query =
            assignment_query.filter(user_module_role::Column::Role.eq(role_filter.clone()));
    }

    if let Some(module_filter) = query.module_id {
        assignment_query = assignment_query.filter(assignment::Column::ModuleId.eq(module_filter));
    }

    let mut date_conditions = Condition::all();

    if let Some(from) = from_dt {
        date_conditions = date_conditions.add(
            Condition::any()
                .add(assignment::Column::AvailableFrom.gte(from))
                .add(assignment::Column::DueDate.gte(from)),
        );
    }

    if let Some(to) = to_dt {
        date_conditions = date_conditions.add(
            Condition::any()
                .add(assignment::Column::AvailableFrom.lte(to))
                .add(assignment::Column::DueDate.lte(to)),
        );
    }

    assignment_query = assignment_query.filter(date_conditions);

    let assignments = match assignment_query.all(state.db()).await {
        Ok(assignments) => assignments,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(format!("Database error: {}", e))),
            )
                .into_response();
        }
    };

    for assignment in assignments {
        let available_in_range = match (from_dt, to_dt) {
            (Some(from), Some(to)) => {
                assignment.available_from >= from && assignment.available_from <= to
            }
            (Some(from), None) => assignment.available_from >= from,
            (None, Some(to)) => assignment.available_from <= to,
            (None, None) => true,
        };

        if available_in_range {
            let available_key = format_iso_datetime(assignment.available_from);
            events_map
                .entry(available_key)
                .or_default()
                .push(EventItem {
                    r#type: "warning".to_string(),
                    content: format!("{} available", assignment.name),
                    module_id: assignment.module_id,
                    assignment_id: assignment.id,
                });
        }

        let due_in_range = match (from_dt, to_dt) {
            (Some(from), Some(to)) => assignment.due_date >= from && assignment.due_date <= to,
            (Some(from), None) => assignment.due_date >= from,
            (None, Some(to)) => assignment.due_date <= to,
            (None, None) => true,
        };

        if due_in_range {
            let due_key = format_iso_datetime(assignment.due_date);
            events_map.entry(due_key).or_default().push(EventItem {
                r#type: "error".to_string(),
                content: format!("{} due", assignment.name),
                module_id: assignment.module_id,
                assignment_id: assignment.id,
            });
        }
    }

    (
        StatusCode::OK,
        Json(ApiResponse::success(
            EventsResponse { events: events_map },
            "Events retrieved successfully",
        )),
    )
        .into_response()
}
