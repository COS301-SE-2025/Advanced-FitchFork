use crate::{
    response::ApiResponse,
    routes::modules::assignments::starter::common::{STARTER_PACKS, StarterPack},
};
use axum::{Json, extract::State, response::IntoResponse};
use hyper::StatusCode;
use util::state::AppState;

pub async fn list(_: State<AppState>) -> impl IntoResponse {
    // Return as ApiResponse so the frontend’s api utils can consume it.
    let items: Vec<StarterPack> = STARTER_PACKS.iter().cloned().collect();
    (
        StatusCode::OK,
        Json(ApiResponse::success(items, "Starter packs")),
    )
}
