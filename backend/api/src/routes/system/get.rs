pub mod code_manager {
    use crate::response::ApiResponse;
    use axum::{Json, http::StatusCode};
    use serde::Deserialize;
    use util::config;

    #[derive(Deserialize)]
    struct CodeManagerMaxConcurrentResp {
        max_concurrent: usize,
    }

    pub async fn get_max_concurrent_handler() -> (StatusCode, Json<ApiResponse<usize>>) {
        let url = format!(
            "http://{}:{}/max_concurrent",
            config::code_manager_host(),
            config::code_manager_port()
        );
        let client = reqwest::Client::new();
        match client.get(url).send().await {
            Ok(resp) if resp.status().is_success() => {
                match resp.json::<CodeManagerMaxConcurrentResp>().await {
                    Ok(body) => (
                        StatusCode::OK,
                        Json(ApiResponse::success(body.max_concurrent, "OK")),
                    ),
                    Err(e) => (
                        StatusCode::BAD_GATEWAY,
                        Json(ApiResponse::error(&format!(
                            "Failed to parse response: {e}"
                        ))),
                    ),
                }
            }
            Ok(resp) => (
                StatusCode::BAD_GATEWAY,
                Json(ApiResponse::error(&format!(
                    "code_manager error: {}",
                    resp.status()
                ))),
            ),
            Err(e) => (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ApiResponse::error(&format!(
                    "Failed to contact code_manager: {e}"
                ))),
            ),
        }
    }
}

pub use code_manager::get_max_concurrent_handler;

pub mod metrics {
    use axum::http::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
    use axum::{
        Json,
        extract::{Query, State},
        http::{HeaderMap, HeaderValue},
        response::IntoResponse,
    };
    use chrono::{Datelike, Duration, TimeZone, Timelike, Utc};
    use db::models::system_metric::{
        Column as MetCol, Entity as SystemMetric, Model as SystemMetricModel,
    };
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
    use serde::Deserialize;
    use serde_json::json;
    use util::state::AppState;

    #[derive(Debug, Deserialize)]
    pub struct MetricsQuery {
        pub start: Option<String>,
        pub end: Option<String>,
        pub bucket: Option<String>, // day|week|month|year
    }

    fn parse_time(s: &str) -> Option<chrono::DateTime<chrono::Utc>> {
        chrono::DateTime::parse_from_rfc3339(s)
            .ok()
            .map(|dt| dt.with_timezone(&chrono::Utc))
    }

    fn validate_bucket(b: &str) -> &'static str {
        match b {
            "day" => "day",
            "week" => "week",
            "month" => "month",
            "year" => "year",
            _ => "day",
        }
    }

    fn group_unit(bucket: &str) -> &'static str {
        match bucket {
            "day" => "hour",
            "week" => "day",
            "month" => "day",
            "year" => "month",
            _ => "hour",
        }
    }

    fn floor_to_unit(ts: chrono::DateTime<Utc>, unit: &str) -> chrono::DateTime<Utc> {
        match unit {
            "hour" => ts
                .date_naive()
                .and_hms_opt(ts.hour(), 0, 0)
                .unwrap()
                .and_utc(),
            "day" => ts.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc(),
            "month" => ts
                .date_naive()
                .with_day(1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_utc(),
            _ => ts,
        }
    }

    fn default_bounds(
        now: chrono::DateTime<Utc>,
        bucket: &str,
    ) -> (chrono::DateTime<Utc>, chrono::DateTime<Utc>) {
        match bucket {
            "day" => {
                let s = now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
                (s, now)
            }
            "week" => {
                let wd = now.weekday().num_days_from_monday() as i64;
                let s = (now - Duration::days(wd))
                    .date_naive()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_utc();
                (s, now)
            }
            "month" => {
                let s = now
                    .date_naive()
                    .with_day(1)
                    .unwrap()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_utc();
                (s, now)
            }
            "year" => {
                let s = now
                    .date_naive()
                    .with_month(1)
                    .unwrap()
                    .with_day(1)
                    .unwrap()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_utc();
                (s, now)
            }
            _ => (now - Duration::hours(24), now),
        }
    }

    fn step_next(dt: chrono::DateTime<Utc>, unit: &str) -> chrono::DateTime<Utc> {
        match unit {
            "hour" => dt + Duration::hours(1),
            "day" => dt + Duration::days(1),
            "month" => {
                let (y, m) = (dt.year(), dt.month());
                let (ny, nm) = if m == 12 { (y + 1, 1) } else { (y, m + 1) };
                Utc.with_ymd_and_hms(ny, nm, 1, 0, 0, 0).unwrap()
            }
            _ => dt,
        }
    }

    #[derive(Debug)]
    struct MetricPoint {
        ts: chrono::DateTime<Utc>,
        cpu_avg: f64,
        mem_pct: f64,
    }

    fn aggregate_points(
        rows: Vec<SystemMetricModel>,
        start: chrono::DateTime<Utc>,
        end: chrono::DateTime<Utc>,
        unit: &str,
    ) -> Vec<MetricPoint> {
        use std::collections::BTreeMap;

        #[derive(Default)]
        struct Agg {
            n: u64,
            cpu: f64,
            mem_pct: f64,
        }

        let mut map: BTreeMap<i64, Agg> = BTreeMap::new();
        for r in rows {
            let key = floor_to_unit(r.created_at, unit).timestamp();
            let entry = map.entry(key).or_default();
            entry.n += 1;
            entry.cpu += r.cpu_avg as f64;
            entry.mem_pct += r.mem_pct as f64;
        }

        let mut points = Vec::new();
        let mut t = start;
        while t <= end {
            let key = t.timestamp();
            if let Some(a) = map.get(&key) {
                let n = a.n.max(1) as f64;
                points.push(MetricPoint {
                    ts: t,
                    cpu_avg: a.cpu / n,
                    mem_pct: a.mem_pct / n,
                });
            } else {
                points.push(MetricPoint {
                    ts: t,
                    cpu_avg: 0.0,
                    mem_pct: 0.0,
                });
            }
            t = step_next(t, unit);
        }

        points
    }

    pub async fn get_metrics(
        State(state): State<AppState>,
        Query(q): Query<MetricsQuery>,
    ) -> Json<serde_json::Value> {
        let bucket = validate_bucket(q.bucket.as_deref().unwrap_or("day"));
        let unit = group_unit(bucket);

        let (mut start, mut end) = match (
            q.start.as_deref().and_then(parse_time),
            q.end.as_deref().and_then(parse_time),
        ) {
            (Some(s), Some(e)) => (s, e),
            _ => default_bounds(Utc::now(), bucket),
        };
        start = floor_to_unit(start, unit);
        end = floor_to_unit(end, unit);

        let rows = SystemMetric::find()
            .filter(MetCol::CreatedAt.gte(start))
            .filter(MetCol::CreatedAt.lte(end))
            .all(state.db())
            .await
            .unwrap_or_default();

        let points = aggregate_points(rows, start, end, unit);
        let payload: Vec<_> = points
            .into_iter()
            .map(|p| {
                json!({
                    "ts": p.ts.to_rfc3339(),
                    "cpu_avg": p.cpu_avg,
                    "mem_pct": p.mem_pct,
                })
            })
            .collect();

        Json(json!({ "points": payload }))
    }

    pub async fn get_metrics_csv(
        State(state): State<AppState>,
        Query(q): Query<MetricsQuery>,
    ) -> impl IntoResponse {
        let bucket = validate_bucket(q.bucket.as_deref().unwrap_or("day"));
        let unit = group_unit(bucket);

        let (mut start, mut end) = match (
            q.start.as_deref().and_then(parse_time),
            q.end.as_deref().and_then(parse_time),
        ) {
            (Some(s), Some(e)) => (s, e),
            _ => default_bounds(Utc::now(), bucket),
        };
        start = floor_to_unit(start, unit);
        end = floor_to_unit(end, unit);

        let rows = SystemMetric::find()
            .filter(MetCol::CreatedAt.gte(start))
            .filter(MetCol::CreatedAt.lte(end))
            .all(state.db())
            .await
            .unwrap_or_default();

        let points = aggregate_points(rows, start, end, unit);
        let mut csv = String::from("timestamp,cpu_avg,mem_pct\n");
        for p in points {
            csv.push_str(&format!(
                "{},{:.4},{:.4}\n",
                p.ts.to_rfc3339(),
                p.cpu_avg,
                p.mem_pct
            ));
        }

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("text/csv"));
        headers.insert(
            CONTENT_DISPOSITION,
            HeaderValue::from_static("attachment; filename=system_metrics.csv"),
        );

        (headers, csv)
    }
}

pub use metrics::{MetricsQuery, get_metrics, get_metrics_csv};

pub mod submissions {
    use axum::extract::{Query, State};
    use axum::{Json, http::HeaderValue, http::header};
    use chrono::{Datelike, Duration, TimeZone, Timelike, Utc};
    use db::models::assignment_submission::{Column as SubCol, Entity as Submission};
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use util::state::AppState;

    #[derive(Debug, Deserialize)]
    pub struct SubmissionsQuery {
        pub start: Option<String>,  // RFC3339
        pub end: Option<String>,    // RFC3339
        pub bucket: Option<String>, // day|week|month|year
        pub module_id: Option<i64>,
        pub assignment_id: Option<i64>,
    }

    fn parse_time(s: &str) -> Option<chrono::DateTime<chrono::Utc>> {
        chrono::DateTime::parse_from_rfc3339(s)
            .ok()
            .map(|dt| dt.with_timezone(&chrono::Utc))
    }

    #[derive(Debug, Serialize)]
    struct Point {
        period: String,
        count: i64,
    }

    fn validate_bucket(b: &str) -> &'static str {
        match b {
            "day" => "day",
            "week" => "week",
            "month" => "month",
            "year" => "year",
            _ => "day",
        }
    }

    fn group_unit(bucket: &str) -> &'static str {
        match bucket {
            "day" => "hour",
            "week" => "day",
            "month" => "day",
            "year" => "month",
            _ => "hour",
        }
    }

    fn floor_to_unit(ts: chrono::DateTime<Utc>, unit: &str) -> chrono::DateTime<Utc> {
        match unit {
            "hour" => ts
                .date_naive()
                .and_hms_opt(ts.hour(), 0, 0)
                .unwrap()
                .and_utc(),
            "day" => ts.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc(),
            "month" => ts
                .date_naive()
                .with_day(1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_utc(),
            _ => ts,
        }
    }

    fn default_bounds(
        now: chrono::DateTime<Utc>,
        bucket: &str,
    ) -> (chrono::DateTime<Utc>, chrono::DateTime<Utc>) {
        match bucket {
            "day" => {
                let start = now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
                let end = start + Duration::hours(24) - Duration::seconds(1);
                (start, end)
            }
            "week" => {
                let wd = now.weekday().num_days_from_monday() as i64;
                let start = (now - Duration::days(wd))
                    .date_naive()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_utc();
                let end = start + Duration::days(7) - Duration::seconds(1);
                (start, end)
            }
            "month" => {
                let start = now
                    .date_naive()
                    .with_day(1)
                    .unwrap()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_utc();
                let (y, m) = (start.year(), start.month());
                let (ny, nm) = if m == 12 { (y + 1, 1) } else { (y, m + 1) };
                let end = Utc.with_ymd_and_hms(ny, nm, 1, 0, 0, 0).unwrap() - Duration::seconds(1);
                (start, end)
            }
            "year" => {
                let start = now
                    .date_naive()
                    .with_month(1)
                    .unwrap()
                    .with_day(1)
                    .unwrap()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_utc();
                let end = Utc
                    .with_ymd_and_hms(start.year() + 1, 1, 1, 0, 0, 0)
                    .unwrap()
                    - Duration::seconds(1);
                (start, end)
            }
            _ => {
                let start = now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
                (start, now)
            }
        }
    }

    fn step_next(dt: chrono::DateTime<Utc>, unit: &str) -> chrono::DateTime<Utc> {
        match unit {
            "hour" => dt + Duration::hours(1),
            "day" => dt + Duration::days(1),
            "month" => {
                let (y, m) = (dt.year(), dt.month());
                let (ny, nm) = if m == 12 { (y + 1, 1) } else { (y, m + 1) };
                Utc.with_ymd_and_hms(ny, nm, 1, 0, 0, 0).unwrap()
            }
            _ => dt,
        }
    }

    fn key_string(dt: chrono::DateTime<Utc>, bucket: &str) -> String {
        match bucket {
            "day" => format!(
                "{:04}-{:02}-{:02} {:02}:00:00",
                dt.year(),
                dt.month(),
                dt.day(),
                dt.hour()
            ),
            "week" | "month" => format!("{:04}-{:02}-{:02}", dt.year(), dt.month(), dt.day()),
            "year" => format!("{:04}-{:02}", dt.year(), dt.month()),
            _ => dt.to_rfc3339(),
        }
    }

    pub async fn submissions_over_time(
        State(state): State<AppState>,
        Query(q): Query<SubmissionsQuery>,
    ) -> Json<serde_json::Value> {
        match query_rows(state.db(), q).await {
            Ok(rows) => Json(json!({ "points": rows })),
            Err(_) => Json(json!({ "points": [] })),
        }
    }

    pub async fn submissions_over_time_export(
        State(state): State<AppState>,
        Query(q): Query<SubmissionsQuery>,
    ) -> impl axum::response::IntoResponse {
        let rows = query_rows(state.db(), q).await.unwrap_or_default();
        let mut csv = String::from("period,count\n");
        for r in rows {
            csv.push_str(&format!("{},{}\n", r.period, r.count));
        }
        let mut headers = axum::http::HeaderMap::new();
        headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("text/csv"));
        headers.insert(
            header::CONTENT_DISPOSITION,
            HeaderValue::from_static("attachment; filename=submissions_over_time.csv"),
        );
        (headers, csv)
    }

    async fn query_rows(
        db: &sea_orm::DatabaseConnection,
        q: SubmissionsQuery,
    ) -> Result<Vec<Point>, sea_orm::DbErr> {
        let bucket = validate_bucket(q.bucket.as_deref().unwrap_or("day"));
        let unit = group_unit(bucket);

        let (mut start, mut end) = match (
            q.start.as_deref().and_then(parse_time),
            q.end.as_deref().and_then(parse_time),
        ) {
            (Some(s), Some(e)) => (s, e),
            _ => default_bounds(Utc::now(), bucket),
        };
        start = floor_to_unit(start, unit);
        end = floor_to_unit(end, unit);

        let mut finder = Submission::find()
            .filter(SubCol::Ignored.eq(false))
            .filter(SubCol::IsPractice.eq(false));

        finder = finder.filter(SubCol::CreatedAt.gte(start));
        finder = finder.filter(SubCol::CreatedAt.lte(end));

        if let Some(aid) = q.assignment_id {
            finder = finder.filter(SubCol::AssignmentId.eq(aid));
        }

        let subs = finder.all(db).await?;

        use std::collections::BTreeMap;
        let mut map: BTreeMap<i64, i64> = BTreeMap::new();

        for s in subs {
            let ts = s.created_at.with_timezone(&Utc);
            let key_dt = floor_to_unit(ts, unit);
            *map.entry(key_dt.timestamp()).or_insert(0) += 1;
        }

        let mut out = Vec::new();
        let mut t = start;
        while t <= end {
            let key = t.timestamp();
            let count = map.get(&key).copied().unwrap_or(0);
            out.push(Point {
                period: key_string(t, bucket).to_string(),
                count,
            });
            t = step_next(t, unit);
        }

        Ok(out)
    }
}

pub use submissions::{SubmissionsQuery, submissions_over_time, submissions_over_time_export};
