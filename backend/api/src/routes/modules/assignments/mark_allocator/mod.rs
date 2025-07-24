/// Mark Allocator route module.
///
/// This module defines HTTP routes for generating, loading, and saving mark allocator data.
/// Each route is protected with middleware that ensures only lecturers can access them.

pub mod get;
pub mod post;
pub mod put;

use axum::{
    extract::Path,
    middleware::from_fn,
    routing::{get, post, put},
    Router,
};

use get::load;
use post::generate;
use put::save;

use crate::auth::guards::require_lecturer;

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
        .route(
            "/generate",
            post(generate).layer(from_fn(|Path(params): Path<(i64, i64)>, req, next| {
                require_lecturer(Path(params), req, next)
            })),
        )
        .route(
            "/",
            get(load).layer(from_fn(|Path(params): Path<(i64, i64)>, req, next| {
                require_lecturer(Path(params), req, next)
            })),
        )
        .route(
            "/",
            put(save).layer(from_fn(|Path(params): Path<(i64, i64)>, req, next| {
                require_lecturer(Path(params), req, next)
            })),
        )
}