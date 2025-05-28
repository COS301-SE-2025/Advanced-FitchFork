//! # Example Routes Module
//!
//! This module defines and wires up routes under the `/example` endpoint group.
//! It is primarily used for demonstrating and testing routing, authentication, and middleware usage.
//!
//! ## Structure
//! - `get.rs` — GET handlers for test and auth-related routes
//! - `post.rs` — POST handlers for creating test data
//! - `delete.rs` — DELETE handlers (placeholder for protected deletion)
//!
//! ## JWT-Based Middleware
//! Some routes require authentication using JWTs, which are parsed and validated
//! by Axum middleware. The middleware extracts an [`AuthUser`] from the JWT and injects it
//! into request extensions. These claims are accessible in route handlers via `Extension<AuthUser>`.
//!
//! JWT Claims (inside the token):
//! - `sub`: the user's unique ID (`i64`)
//! - `admin`: whether the user has admin privileges (`bool`)
//! - `exp`: token expiration time (`Unix timestamp` in seconds)
//!
//! ### Middleware Summary
//! - `require_authenticated`: ensures the JWT is present and valid
//! - `require_admin`: ensures the user is authenticated **and** has `admin = true`
//!
//! ## Routes Overview
//! - `GET /example` – public route (no authentication required)
//! - `GET /example/auth` – requires a valid JWT
//! - `GET /example/admin` – requires JWT with `admin = true`
//! - `GET /example/admin-auth` – requires both `require_authenticated` and `require_admin` middleware (redundant by design, for demonstration)
//! - `POST /example` – creates a new item (public route)
//! - `DELETE /example/:id` – placeholder route (middleware not yet applied)

pub mod get;
pub mod post;
pub mod delete;

use axum::{
    Router,
    routing::{get, post, delete},
    middleware,
};
use crate::auth::guards::{require_authenticated, require_admin};

use get::{index, test_get_route_auth, test_get_route_admin, test_get_route_admin_double_protected};
use post::create;
use delete::delete_example;

/// Composes the `/example` route group with all available endpoints for testing routing and middleware.
/// 
/// ### Route Definitions:
/// 
/// - `GET /`  
///   → `index` — public route, no authentication required
///
/// - `GET /auth`  
///   → `test_get_route_auth` — requires a valid JWT (`require_authenticated`)
///
/// - `GET /admin`  
///   → `test_get_route_admin` — requires JWT with `admin = true` (`require_admin`)
///
/// - `GET /admin-auth`  
///   → `test_get_route_admin_double_protected` — demonstrates both `require_authenticated` and `require_admin` middleware (redundant for example purposes)
///
/// - `POST /`  
///   → `create` — public route, no authentication required
///
/// - `DELETE /:id`  
///   → `delete_example` — placeholder for protected delete route (middleware not yet applied)
pub fn example_routes() -> Router {
    Router::new()
        .route("/", get(index))
        .route("/auth", get(test_get_route_auth).layer(middleware::from_fn(require_authenticated)))
        .route("/admin", get(test_get_route_admin).layer(middleware::from_fn(require_admin)))
        .route(
            "/admin-auth",
            get(test_get_route_admin_double_protected)
                .layer(middleware::from_fn(require_authenticated))
                .layer(middleware::from_fn(require_admin)),
        )
        .route("/", post(create))
        .route("/:id", delete(delete_example))
}