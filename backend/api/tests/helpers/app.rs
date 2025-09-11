use axum::{
    body::Body,
    http::Request,
    middleware::from_fn,
    response::Response,
    Router,
};
use util::state::AppState;
use api::{
    auth::guards::validate_known_ids,
    routes::routes,
    ws::ws_routes,
};
use services::util::UtilService;
use tower::util::BoxCloneService;
use tower::ServiceExt;
use std::convert::Infallible;
use ctor::{ctor, dtor};

#[ctor]
fn setup_tests() {
    println!("🚀 Setting up test environment...");

    let _ = AppState::init(true);

    println!("✅ Test environment set up");
}

#[dtor]
fn cleanup_tests() {
    println!("🧹 Cleaning up test environment...");

    let _ = std::fs::remove_dir_all("./tmp");
    let _ = std::fs::remove_file("test.db");
    
    println!("✅ Test environment cleaned up");
}

pub async fn make_test_app() -> BoxCloneService<Request<Body>, Response, Infallible> {
    UtilService::clean_db().await.expect("Failed to clean db");

    let router = Router::new()
        .nest("/api", routes().layer(from_fn(validate_known_ids)))
        .nest("/ws", ws_routes());

    router.into_service().boxed_clone()
}