//api/api.rs
use crate::manager::manager::ContainerManager;
use axum::{extract::Json, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use util::{execution_config::ExecutionConfig, paths};

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
pub fn init_manager(default_max_concurrent: usize) {
    let resolved = match load_persisted_max_concurrent() {
        Some(value) => {
            tracing::info!(
                max_concurrent = value,
                "Using persisted max_concurrent value"
            );
            value
        }
        None => default_max_concurrent,
    };
    if MANAGER.set(ContainerManager::new(resolved)).is_err() {
        tracing::warn!("ContainerManager was already initialized");
    } else {
        tracing::info!(max_concurrent = resolved, "Initialized ContainerManager");
        if let Err(err) = persist_max_concurrent(resolved) {
            tracing::warn!(
                error = %err,
                value = resolved,
                "Failed to persist initial max_concurrent"
            );
        }
    }
}

#[derive(Debug, Serialize)]
pub struct StatsResponse {
    pub running: usize,
    pub waiting: usize,
    pub max_concurrent: usize,
}

pub async fn stats() -> impl IntoResponse {
    let manager = MANAGER.get().expect("Manager not initialized");
    let (running, waiting, max_concurrent) = manager.get_stats().await;
    (
        StatusCode::OK,
        axum::Json(StatsResponse {
            running,
            waiting,
            max_concurrent,
        }),
    )
        .into_response()
}

#[derive(Debug, Serialize)]
pub struct MaxConcurrentGetResponse {
    pub max_concurrent: usize,
}

#[derive(Debug, Deserialize)]
pub struct MaxConcurrentSetRequest {
    pub max_concurrent: usize,
}

pub async fn get_max_concurrent() -> impl IntoResponse {
    let manager = MANAGER.get().expect("Manager not initialized");
    let (_, _, max_concurrent) = manager.get_stats().await;
    (
        StatusCode::OK,
        axum::Json(MaxConcurrentGetResponse { max_concurrent }),
    )
        .into_response()
}

pub async fn set_max_concurrent(Json(req): Json<MaxConcurrentSetRequest>) -> impl IntoResponse {
    if req.max_concurrent == 0 {
        return (StatusCode::BAD_REQUEST, "max_concurrent must be >= 1").into_response();
    }
    let manager = MANAGER.get().expect("Manager not initialized");
    manager.set_max_concurrent(req.max_concurrent).await;
    if let Err(err) = persist_max_concurrent(req.max_concurrent) {
        tracing::warn!(
            error = %err,
            value = req.max_concurrent,
            "Failed to persist max_concurrent override"
        );
    }
    (
        StatusCode::OK,
        axum::Json(MaxConcurrentGetResponse {
            max_concurrent: req.max_concurrent,
        }),
    )
        .into_response()
}

fn max_concurrent_storage_path() -> std::path::PathBuf {
    paths::system_dir().join("max_concurrent.txt")
}

fn persist_max_concurrent(value: usize) -> std::io::Result<()> {
    let path = max_concurrent_storage_path();
    paths::ensure_parent_dir(&path)?;
    std::fs::write(path, value.to_string())
}

fn load_persisted_max_concurrent() -> Option<usize> {
    let path = max_concurrent_storage_path();
    match std::fs::read_to_string(&path) {
        Ok(contents) => match contents.trim().parse::<usize>() {
            Ok(value) => Some(value),
            Err(err) => {
                tracing::warn!(
                    error = %err,
                    "Invalid persisted max_concurrent value; falling back to default"
                );
                None
            }
        },
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => None,
        Err(err) => {
            tracing::warn!(error = %err, "Failed to read persisted max_concurrent");
            None
        }
    }
}
