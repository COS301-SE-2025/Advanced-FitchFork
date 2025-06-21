use axum::{extract::Path, response::IntoResponse, Json};
use serde_json::Value;
use util::mark_allocator::mark_allocator::save_allocator;

pub async fn save(
    Path((module_id, assignment_id)): Path<(i64, i64)>,
    Json(req): Json<Value>,
) -> impl IntoResponse {
}
