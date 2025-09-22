/// Mark Allocator route module.
///
/// This module defines HTTP routes for generating, loading, and saving mark allocator data.
/// Each route is protected with middleware that ensures only lecturers can access them.
use axum::{
    Router,
    routing::{get, post, put},
};
use get::load;
use post::generate;
use put::save;

pub mod get;
pub mod post;
pub mod put;

/// Registers routes related to the mark allocator system.
///
/// The following endpoints are exposed at `/`:
///
/// - `POST /` → `generate` a new mark allocator based on memo output files.
/// - `GET  /` → `load` an existing allocator from disk.
/// - `PUT  /` → `save` updated allocator data to disk.
///
/// All routes require lecturer authentication using the `require_lecturer` middleware.
pub fn mark_allocator_routes() -> Router {
    Router::new()
        .route("/generate", post(generate))
        .route("/", get(load))
        .route("/", put(save))
}
