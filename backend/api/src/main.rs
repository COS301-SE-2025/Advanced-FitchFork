use api::auth::guards::{SUPERUSER_IDS, validate_known_ids};
use api::routes::routes;
use api::ws::system::payload::{
    CodeManagerAdmin, CodeManagerGeneral, CpuInfo, DiskSummary, LoadAverages, MemoryInfo,
    SystemHealthAdminPayload, SystemHealthGeneralPayload,
};
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

use util::system_health::sample_system_metrics;
use util::{config, state::AppState, ws::WebSocketManager};

use api::ws::system::emit::{health_admin, health_general};

#[tokio::main]
async fn main() {
    // Load configuration and initialize logging
    let _log_guard = init_logging(&config::log_file(), &config::log_level());

    // Initialize superuser IDs
    let _ = once_cell::sync::Lazy::force(&SUPERUSER_IDS);

    // Set up dependencies
    let db = connect().await;
    let ws = WebSocketManager::new();
    let app_state = AppState::new(db.clone(), ws);

    // Initialize achievement service
    let achievement_config = db::achievement_service::AchievementService::initialize(
        db.clone(),
        Some(db::achievement_engine::AchievementEngineConfig {
            debug_logging: config::env() != "production",
            emit_level_events: true,
            max_achievements_per_event: 100,
        })
    ).await;
    
    match achievement_config {
        Ok(_) => tracing::info!("Achievement service initialized successfully"),
        Err(e) => {
            tracing::warn!("Failed to initialize achievement service: {}. Achievement system will be disabled.", e);
            tracing::warn!("This may be due to missing achievements.json file or database issues.");
        }
    }

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
        .layer(cors)
        .with_state(app_state);

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
        let persist_interval = Duration::from_secs(config::system_health_persist_seconds());
        let mut last_persist = Instant::now();
        let mut first_persist = true;

        loop {
            tokio::time::sleep(Duration::from_millis(interval_ms)).await;
            let metrics = sample_system_metrics();

            // Code manager stats
            let mut cm_running: usize = 0;
            let mut cm_waiting: usize = 0;
            let mut cm_max: Option<usize> = None;

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
                            .map(|n| n as usize);
                    }
                }
            }

            // ----- build typed payloads -----
            let ts = chrono::Utc::now().to_rfc3339();

            let load = LoadAverages {
                one: metrics.load_one,
                five: metrics.load_five,
                fifteen: metrics.load_fifteen,
            };

            let general_payload = SystemHealthGeneralPayload {
                ts: ts.clone(),
                load: load.clone(),
                code_manager: CodeManagerGeneral {
                    running: cm_running,
                    waiting: cm_waiting,
                },
            };

            let admin_payload = SystemHealthAdminPayload {
                ts,
                env: config::env().to_string(),
                host: config::host().to_string(),
                uptime_seconds: metrics.uptime_seconds,
                load,
                cpu: CpuInfo {
                    cores: metrics.cpu_cores,
                    avg_usage: metrics.cpu_avg_usage,
                    per_core: metrics.cpu_per_core.clone(),
                },
                memory: MemoryInfo {
                    total: metrics.mem_total,
                    used: metrics.mem_used,
                    swap_total: metrics.swap_total,
                    swap_used: metrics.swap_used,
                },
                disks: metrics
                    .disks
                    .iter()
                    .map(|d| DiskSummary {
                        name: d.name.clone(),
                        total: d.total,
                        available: d.available,
                        file_system: d.file_system.clone(),
                        mount_point: d.mount_point.clone(),
                    })
                    .collect(),
                code_manager: CodeManagerAdmin {
                    running: cm_running,
                    waiting: cm_waiting,
                    max_concurrent: cm_max,
                },
            };

            // ----- emit via WS (typed events → enveloped & serialized once) -----
            health_general(&ws, general_payload).await;
            health_admin(&ws, admin_payload).await;

            // ----- periodic persistence -----
            if first_persist || last_persist.elapsed() >= persist_interval {
                // If you need a mem %:
                let mem_used_bytes = metrics.mem_used as f64;
                let mem_total_bytes = metrics.mem_total as f64;
                let mem_pct = if mem_total_bytes > 0.0 {
                    (mem_used_bytes / mem_total_bytes) * 100.0
                } else {
                    0.0
                };

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
