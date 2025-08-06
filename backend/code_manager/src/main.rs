//main.rs
use axum::{routing::get, Router};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use util::config::AppConfig;

mod api;
mod manager;

#[tokio::main]
async fn main() {
    let config = AppConfig::from_env();
    // Initialize tracing subscriber (logging)
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Build API routes
    let app = Router::new()
        .route("/health", get(api::health))
        .route("/run", axum::routing::post(api::run_code));

    // Define address to listen on
    let addr: SocketAddr = format!("{}:{}", config.code_manager_host, config.code_manager_port)
        .parse()
        .expect("Invalid address");
    tracing::info!("Listening on {}", addr);

    // Create TCP listener and run server
    let listener = TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
