use crate::response::ApiResponse;
use axum::{Json, http::StatusCode};
use serde::{Deserialize, Serialize};
use util::config;

#[derive(Deserialize)]
pub struct SetMaxConcurrentRequest {
    pub max_concurrent: usize,
}

#[derive(Serialize)]
struct CodeManagerSetReq {
    max_concurrent: usize,
}

pub async fn set_max_concurrent_handler(
    Json(req): Json<SetMaxConcurrentRequest>,
) -> (StatusCode, Json<ApiResponse<usize>>) {
    if req.max_concurrent == 0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error("max_concurrent must be >= 1")),
        );
    }

    let url = format!(
        "http://{}:{}/max_concurrent",
        config::code_manager_host(),
        config::code_manager_port()
    );
    let body = CodeManagerSetReq {
        max_concurrent: req.max_concurrent,
    };
    let client = reqwest::Client::new();
    match client.post(url).json(&body).send().await {
        Ok(resp) if resp.status().is_success() => (
            StatusCode::OK,
            Json(ApiResponse::success(req.max_concurrent, "Updated")),
        ),
        Ok(resp) => (
            StatusCode::BAD_GATEWAY,
            Json(ApiResponse::error(&format!(
                "code_manager error: {}",
                resp.status()
            ))),
        ),
        Err(e) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ApiResponse::error(&format!(
                "Failed to contact code_manager: {e}"
            ))),
        ),
    }
}
