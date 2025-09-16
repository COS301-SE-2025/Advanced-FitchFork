//! File Routes Module
//!
//! This module defines the routing for assignment file-related endpoints, including uploading, listing, downloading, and deleting files. It applies access control middleware to ensure appropriate permissions for each operation.

use crate::{auth::guards::{require_lecturer}, routes::modules::assignments::interpreter::get::get_interpreter_info};
use axum::{
    Router,
    middleware::from_fn,
    routing::{delete, get, post},
};
use delete::delete_interpreter;
use get::download_interpreter;
use post::upload_interpreter;

pub mod delete;
pub mod get;
pub mod post;

/// Registers the routes for assignment file endpoints.
///
/// This function sets up the following endpoints under the current router:
///
/// - `POST /`: Upload files to an assignment. Access is restricted to lecturers assigned to the module.
/// - `GET /`: List all files for an assignment. Access is restricted to lecturers assigned to the module.
/// - `GET /{file_id}`: Download a specific file from an assignment. Access is restricted to lecturers assigned to the module.
/// - `DELETE /`: Delete files from an assignment. Access is restricted to lecturers assigned to the module.
///
/// Routes apply appropriate middleware based on the operation:
/// - Upload and delete operations require lecturer permissions
/// - List and download operations require module assignment
///
/// # Returns
/// An [`axum::Router`] with the file endpoints and their associated middleware.
pub fn interpreter_routes() -> Router {
    Router::new()
        .route("/",post(upload_interpreter).route_layer(from_fn(require_lecturer)))
        .route("/",get(download_interpreter).route_layer(from_fn(require_lecturer)))
        .route("/info",get(get_interpreter_info).route_layer(from_fn(require_lecturer)))
        .route("/",delete(delete_interpreter).route_layer(from_fn(require_lecturer)))
}
