use axum::{Router, middleware::from_fn};
use api::routes::routes;
use api::auth::guards::validate_known_ids;
use tokio::runtime::Runtime;
use std::sync::OnceLock;
use util::app_state::app_state::AppState;
use db::get_connection;
use ctor::{ctor, dtor};

static TEST_RUNTIME: OnceLock<Runtime> = OnceLock::new();

#[ctor]
fn setup_tests() {
    println!("🚀 Initializing test environment...");
    
    let rt = Runtime::new().expect("Failed to create tokio runtime");
    
    rt.block_on(async {
        let _state = AppState::init(true);
        let _db = get_connection().await;
        
        println!("✅ Test environment initialized successfully");
    });
    
    // Store the runtime so it stays alive
    TEST_RUNTIME.set(rt).expect("Failed to set test runtime");
}

#[dtor]
fn cleanup_tests() {
    println!("🧹 Cleaning up test environment...");
    
    // if let Some(rt) = TEST_RUNTIME.get() {
    //     rt.block_on(async {
    //         // Final cleanup if needed
    //         let _ = std::fs::remove_dir_all("./tmp");
    //         let _ = std::fs::remove_file("test.db");
    //     });
    // }
    
    println!("✅ Test environment cleaned up");
}

pub fn make_app() -> Router {
    Router::new()
        .nest("/api", routes())
        .layer(from_fn(validate_known_ids))
}