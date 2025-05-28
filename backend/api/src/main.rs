use api::routes::routes;
use axum::Router;
use axum::http::header;
use common::{config::Config, logger::init_logger};
use log::info;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() {
    let config = Config::init(".env");
    init_logger(&config.log_level, &config.log_file);
    db::init(&config.database_url, true).await;
    db::seed_db().await;
    // docker_example::run_assignment_code("docker_example/src/files/good_java_example.zip", "java").await;

    info!(
        "Starting {} on http://{}:{}",
        config.project_name, config.host, config.port
    );

    // CORS setup (allow frontend origin)
    let cors = CorsLayer::new()
        .allow_origin(axum::http::HeaderValue::from_static(
            "http://localhost:5173",
        ))
        .allow_methods(Any)
        .allow_headers(Any)
        .expose_headers([header::CONTENT_DISPOSITION]);

    // Compose routes and apply middleware
    let app = Router::new().nest("/api", routes()).layer(cors);

    // Bind and serve
    let addr: SocketAddr = format!("{}:{}", config.host, config.port)
        .parse()
        .expect("Invalid address");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind");

    axum::serve(listener, app).await.expect("Server crashed");
}
