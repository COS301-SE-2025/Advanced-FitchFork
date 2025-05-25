use api::routes::routes;
use axum::Router;
use common::{config::Config, logger::init_logger};
use log::info;

#[tokio::main]
async fn main() {
    // Load config from .env
    let config = Config::init(".env");

    // Initialize logger with values from config
    init_logger(&config.log_level, &config.log_file);

    // Initialize the database
    db::init(&config.database_url, true).await;
    db::seed_db().await;
    docker_example::run_assignment_code("docker_example/src/files/good_java_example.zip", "java")
        .await;

    info!(
        "Starting {} on http://{}:{}",
        config.project_name, config.host, config.port
    );

    // Compose routes and apply middleware
    let app = Router::new().nest("/api", routes());

    // Bind TCP listener
    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind");

    // Serve the app
    axum::serve(listener, app).await.expect("Server crashed");
}
