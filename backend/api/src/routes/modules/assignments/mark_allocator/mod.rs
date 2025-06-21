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

pub fn mark_allocator_routes() -> Router {
    Router::new()
        .route(
            "/",
            post(generate).layer(from_fn(|Path(params): Path<(i64,)>, req, next| {
                require_lecturer(Path(params), req, next)
            })),
        )
        .route(
            "/",
            get(load).layer(from_fn(|Path(params): Path<(i64,)>, req, next| {
                require_lecturer(Path(params), req, next)
            })),
        )
        .route(
            "/",
            put(save).layer(from_fn(|Path(params): Path<(i64,)>, req, next| {
                require_lecturer(Path(params), req, next)
            })),
        )
}
