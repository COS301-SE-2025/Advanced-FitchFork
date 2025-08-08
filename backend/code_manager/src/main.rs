//main.rs
use axum::{routing::get, Router};
use code_manager::api::api::{health, init_manager, run_code};
use dotenv::dotenv;
use std::env;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // Load .env
    dotenv().ok();

    // Initialize tracing subscriber (logging)
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Initialize the global ContainerManager
    let max_containers: usize = env::var("MAX_NUM_CONTAINERS")
        .expect("MAX_NUM_CONTAINERS not set")
        .parse()
        .expect("MAX_NUM_CONTAINERS must be a valid usize");
    init_manager(max_containers);

    // Build API routes
    let app = Router::new()
        .route("/health", get(health))
        .route("/run", axum::routing::post(run_code));

    // Define address to listen on
    let host = env::var("CODE_MANAGER_HOST").expect("CODE_MANAGER_HOST not set");
    let port = env::var("CODE_MANAGER_PORT").expect("CODE_MANAGER_PORT not set");
    let addr: SocketAddr = format!("{}:{}", host, port)
        .parse()
        .expect("Invalid address");
    tracing::info!("Listening on {}", addr);

    // Create TCP listener and run server
    let listener = TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
