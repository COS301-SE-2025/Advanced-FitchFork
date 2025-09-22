use api::auth::guards::{SUPERUSER_IDS, validate_known_ids};
use api::routes::routes;
use api::ws::system::topics; // for topic helpers
use api::{auth::middleware::log_request, ws::ws_routes};
use axum::{
    Router,
    http::header::{CONTENT_DISPOSITION, CONTENT_TYPE},
    middleware::{from_fn, from_fn_with_state},
};
use db::connect;
use db::models::system_metric::ActiveModel as SystemMetricActive;
use sea_orm::{
    ActiveModelTrait,
    ActiveValue::{NotSet, Set},
};
use std::{
    net::SocketAddr,
    time::{Duration, Instant},
};
use tower_http::cors::CorsLayer;
use tracing_appender::rolling;
use util::system_health::{build_health_payloads, sample_system_metrics};
use util::{config, state::AppState, ws::WebSocketManager};

#[tokio::main]
async fn main() {
    // Load configuration and initialize logging
    let _log_guard = init_logging(&config::log_file(), &config::log_level());

    // Initialize superuser IDs
    let _ = once_cell::sync::Lazy::force(&SUPERUSER_IDS);

    // Set up dependencies
    let _ = AppState::init(false);

    // Spawn periodic system health broadcaster over WebSockets
    spawn_system_health_broadcaster(app_state.clone());

    // Configure middleware
    let cors = CorsLayer::very_permissive().expose_headers([CONTENT_DISPOSITION, CONTENT_TYPE]);

    // Build app router
    let app = Router::new()
        .nest(
            "/api",
            routes(app_state.clone())
                .layer(from_fn_with_state(app_state.clone(), validate_known_ids)),
        )
        .nest("/ws", ws_routes(app_state.clone()))
        .layer(from_fn(log_request))
        .layer(cors);
    // Start server
    let addr: SocketAddr = format!("{}:{}", config::host(), config::port())
        .parse()
        .expect("Invalid address");

    println!(
        "Starting {} on http://{}:{}",
        config::project_name(),
        config::host(),
        config::port()
    );

    axum::serve(
        tokio::net::TcpListener::bind(&addr)
            .await
            .expect("Failed to bind"),
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .expect("Server crashed");
}

fn init_logging(log_file: &str, _log_level: &str) -> tracing_appender::non_blocking::WorkerGuard {
    use std::fs;
    use tracing_subscriber::{EnvFilter, fmt, prelude::*};

    fs::create_dir_all("logs").ok();

    let file_appender = rolling::daily("logs", log_file);
    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);

    let file_layer = fmt::layer()
        .with_writer(file_writer)
        .with_ansi(false)
        .with_target(true)
        .with_thread_ids(true);

    let log_to_stdout = config::log_to_stdout();

    let stdout_layer = fmt::layer()
        .with_writer(std::io::stdout)
        .with_ansi(true)
        .with_target(true)
        .with_thread_ids(true);

    let env_filter =
        EnvFilter::try_from_env("LOG_LEVEL").unwrap_or_else(|_| EnvFilter::new("api=info"));

    let registry = tracing_subscriber::registry()
        .with(env_filter)
        .with(file_layer);

    if log_to_stdout {
        registry.with(stdout_layer).init();
    } else {
        registry.init();
    }

    guard
}

fn spawn_system_health_broadcaster(app_state: AppState) {
    let interval_ms = config::system_health_broadcast_ms();
    let cm_host = config::code_manager_host();
    let cm_port = config::code_manager_port();
    let ws = app_state.ws_clone();
    let db = app_state.db_clone();

    tokio::spawn(async move {
        let client = reqwest::Client::new();
        let general_topic = topics::system_health_topic();
        let admin_topic = topics::system_health_admin_topic();
        let persist_interval = Duration::from_secs(config::system_health_persist_seconds());
        let mut last_persist = Instant::now();
        let mut first_persist = true;

        loop {
            tokio::time::sleep(Duration::from_millis(interval_ms)).await;
            let metrics = sample_system_metrics();

            // Code manager stats
            let mut cm_running: usize = 0;
            let mut cm_waiting: usize = 0;
            let mut cm_max: usize = 0;
            let cm_url = format!("http://{}:{}/stats", cm_host, cm_port);
            if let Ok(resp) = client.get(&cm_url).send().await {
                if resp.status().is_success() {
                    if let Ok(v) = resp.json::<serde_json::Value>().await {
                        cm_running =
                            v.get("running").and_then(|x| x.as_u64()).unwrap_or(0) as usize;
                        cm_waiting =
                            v.get("waiting").and_then(|x| x.as_u64()).unwrap_or(0) as usize;
                        cm_max = v
                            .get("max_concurrent")
                            .and_then(|x| x.as_u64())
                            .unwrap_or(0) as usize;
                    }
                }
            }

            let (general, admin) = build_health_payloads(
                &metrics,
                cm_running,
                cm_waiting,
                Some(cm_max),
                &config::env(),
                &config::host(),
            );

            ws.broadcast(&general_topic, general.to_string()).await;
            ws.broadcast(&admin_topic, admin.to_string()).await;

            if first_persist || last_persist.elapsed() >= persist_interval {
                // Option A: compute here if you have used/total (bytes)
                let mem_used_bytes = metrics.mem_used as f64;
                let mem_total_bytes = metrics.mem_total as f64; // ensure sampler provides this
                let mem_pct = if mem_total_bytes > 0.0 {
                    (mem_used_bytes / mem_total_bytes) * 100.0
                } else {
                    0.0
                };

                // Option B: if sampler already has metrics.mem_pct (0..100), just use that
                // let mem_pct = metrics.mem_pct as f64;

                let rec = SystemMetricActive {
                    id: NotSet,
                    created_at: Set(chrono::Utc::now()),
                    cpu_avg: Set(metrics.cpu_avg_usage as f32),
                    mem_pct: Set(mem_pct as f32),
                };
                let _ = rec.insert(&db).await;
                last_persist = Instant::now();
                first_persist = false;
            }
        }
    });
}
