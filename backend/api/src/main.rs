use api::{auth::middleware::log_request, ws::ws_routes};
use api::auth::guards::{validate_known_ids, SUPERUSER_IDS};
use api::routes::routes;
use axum::{
    http::header::{CONTENT_DISPOSITION, CONTENT_TYPE},
    middleware::{from_fn, from_fn_with_state},
    Router,
};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tracing_appender::rolling;
use util::{config::AppConfig, state::AppState, ws::WebSocketManager};
use db::connect;

#[tokio::main]
async fn main() {
    // Load configuration and initialize logging
    let config = AppConfig::from_env();
    let _log_guard = init_logging(&config.log_file, &config.log_level);

    // Initialize superuser IDs
    SUPERUSER_IDS.set(config.superuser_ids).unwrap();

    // Set up dependencies
    let db = connect().await;
    let ws = WebSocketManager::new();
    let app_state = AppState::new(db, ws);

    // Configure middleware
    let cors = CorsLayer::very_permissive().expose_headers([CONTENT_DISPOSITION, CONTENT_TYPE]);

    // Build app router
    let app = Router::new()
        .nest("/api", routes(app_state.clone()).layer(from_fn_with_state(app_state.clone(), validate_known_ids)))
        .nest("/ws", ws_routes(app_state.clone()))
        .layer(from_fn(log_request))
        .layer(cors)
        .with_state(app_state);

    // Start server
    let addr: SocketAddr = format!("{}:{}", config.host, config.port)
        .parse()
        .expect("Invalid address");

    println!(
        "Starting {} on http://{}:{}",
        config.project_name, config.host, config.port
    );

    axum::serve(
        tokio::net::TcpListener::bind(&addr).await.expect("Failed to bind"),
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .expect("Server crashed");
}

fn init_logging(log_file: &str, _log_level: &str) -> tracing_appender::non_blocking::WorkerGuard {
    use std::{env, fs};
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};

    fs::create_dir_all("logs").ok();

    let file_appender = rolling::daily("logs", log_file);
    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);

    let file_layer = fmt::layer()
        .with_writer(file_writer)
        .with_ansi(false)
        .with_target(true)
        .with_thread_ids(true);

    let log_to_stdout = env::var("LOG_TO_STDOUT")
        .unwrap_or_else(|_| "false".to_string())
        .to_lowercase()
        == "true";

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
