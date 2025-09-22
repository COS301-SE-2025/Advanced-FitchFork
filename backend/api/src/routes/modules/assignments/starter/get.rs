use axum::{extract::State, response::IntoResponse, Json};
use hyper::StatusCode;
use util::state::AppState;
use crate::{response::ApiResponse, routes::modules::assignments::starter::common::{StarterPack, STARTER_PACKS}};

pub async fn list(_: State<AppState>) -> impl IntoResponse {
    // Return as ApiResponse so the frontend’s api utils can consume it.
    let items: Vec<StarterPack> = STARTER_PACKS.iter().cloned().collect();
    (StatusCode::OK, Json(ApiResponse::success(items, "Starter packs")))
}
