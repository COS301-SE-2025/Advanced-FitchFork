mod api;
mod config;

use common::logger;
use config::ApiConfig;

use std::net::SocketAddr;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    // Initialize config (singleton)
    ApiConfig::init(".env");
    let config = ApiConfig::get();

    // Setup logging (logs to terminal + file)
    logger::init_logger(&config.log_level, &config.log_file);

    log::info!("Starting {} backend...", config.project_name);

    db::init(&config.database_url, true).await;
    db::seed_db().await;

    // Build our application
    let app = api::routes();

    let addr = SocketAddr::new(config.host.parse().expect("Invalid HOST"), config.port);
    let listener = TcpListener::bind(addr).await.unwrap();

    log::info!("{}-api running at http://{}", config.project_name, addr);

    // Serve the application
    axum::serve(listener, app).await.unwrap();
}
