use axum::{extract::Path, response::IntoResponse};
use util::mark_allocator::mark_allocator::generate_allocator;

pub async fn generate(
    Path((module_id, assignment_id)): Path<(i64, i64)>,
) -> impl IntoResponse {
}
