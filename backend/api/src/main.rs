use api::auth::middleware::log_request;
use api::routes::routes;
use axum::middleware::from_fn;
use axum::Router;
use dotenvy::dotenv;
use std::env;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tracing::info;
use tracing_appender::rolling;
use axum::http::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
use sea_orm::DatabaseConnection;
use db::connect;

#[tokio::main]
async fn main() {
    dotenv().ok();

    // Read env vars
    let project_name = env::var("PROJECT_NAME").unwrap_or_else(|_| "unknown-project".to_string());
    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .expect("PORT must be a valid number");
    let log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());
    let log_file = env::var("LOG_FILE").unwrap_or_else(|_| "api.log".to_string());

    // Important: hold the guard to flush logs
    let _log_guard = init_logging(&log_file, &log_level);

    let db: DatabaseConnection = connect().await;

    info!("Starting {} on http://{}:{}", project_name, host, port);
    println!("Server running at http://{}:{} ({})", host, port, project_name);

    // Setup CORS
    let cors = CorsLayer::very_permissive()
      .expose_headers([CONTENT_DISPOSITION, CONTENT_TYPE]);

    // Setup Axum app
    let app = Router::new()
        .nest("/api", routes(db.clone()))
        .layer(cors)
        .layer(from_fn(log_request))
        .with_state(db);

    let addr: SocketAddr = format!("{}:{}", host, port)
        .parse()
        .expect("Invalid address");

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind");

    axum::serve(listener, app)
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