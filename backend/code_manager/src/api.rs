//api.rs
use crate::manager::manager::ContainerManager;
use axum::{extract::Json, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct RunRequest {
    pub language: String,
    pub files: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct RunResponse {
    pub output: String,
}

// Hold ContainerManager in a global static for shared access
use once_cell::sync::OnceCell;
static MANAGER: OnceCell<ContainerManager> = OnceCell::new();

pub async fn health() -> impl IntoResponse {
    (StatusCode::OK, "code_manager is running")
}

pub async fn run_code(Json(payload): Json<RunRequest>) -> impl IntoResponse {
    tracing::info!("Received run request: {:?}", payload);

    let manager = MANAGER.get().expect("Manager not initialized");

    let output = manager.run(&payload.language, &payload.files).await;

    (StatusCode::OK, axum::Json(RunResponse { output }))
}

/// Initialize global container manager - called once at startup
pub fn init_manager(max_concurrent: usize) {
    if MANAGER.set(ContainerManager::new(max_concurrent)).is_err() {
        tracing::warn!("ContainerManager was already initialized");
    }
}
