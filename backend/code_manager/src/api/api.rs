//api/api.rs
use crate::manager::manager::ContainerManager;
use axum::{extract::Json, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use util::execution_config::ExecutionConfig;

#[derive(Debug, Deserialize)]
pub struct RunRequest {
    pub config: HashMap<String, Value>,
    pub commands: Vec<String>,
    pub files: Vec<(String, Vec<u8>)>,
}

#[derive(Debug, Serialize)]
pub struct RunResponse {
    pub output: Vec<String>,
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

    let config_json = Value::Object(payload.config.into_iter().collect());

    let execution_config: ExecutionConfig = match serde_json::from_value(config_json) {
        Ok(cfg) => cfg,
        Err(e) => {
            let msg = format!("Invalid config: {}", e);
            tracing::error!("{}", msg);
            return (StatusCode::BAD_REQUEST, msg).into_response();
        }
    };

    match manager
        .run(&execution_config, payload.commands, payload.files)
        .await
    {
        Ok(output) => (StatusCode::OK, axum::Json(RunResponse { output })).into_response(),
        Err(e) => {
            let msg = format!("Error running container: {}", e);
            tracing::error!("{}", msg);
            (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
        }
    }
}

/// Initialize global container manager - called once at startup
pub fn init_manager(max_concurrent: usize) {
    if MANAGER.set(ContainerManager::new(max_concurrent)).is_err() {
        tracing::warn!("ContainerManager was already initialized");
    }
}
