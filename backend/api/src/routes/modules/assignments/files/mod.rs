//! File Routes Module
//!
//! This module defines the routing for assignment file-related endpoints, including uploading, listing, downloading, and deleting files. It applies access control middleware to ensure appropriate permissions for each operation.

use crate::auth::guards::{allow_student, allow_lecturer};
use axum::{
    Router,
    middleware::from_fn_with_state,
    routing::{delete, get, post},
};
use delete::delete_files;
use get::{download_file, list_files};
use post::upload_files;
use util::state::AppState;

pub mod delete;
pub mod get;
pub mod post;

/// Registers the routes for assignment file endpoints.
///
/// This function sets up the following endpoints under the current router:
///
/// - `POST /`: Upload files to an assignment. Access is restricted to lecturers assigned to the module.
/// - `GET /`: List all files for an assignment. Access is restricted to users assigned to the module.
/// - `GET /{file_id}`: Download a specific file from an assignment. Access is restricted to users assigned to the module.
/// - `DELETE /`: Delete files from an assignment. Access is restricted to lecturers assigned to the module.
///
/// Routes apply appropriate middleware based on the operation:
/// - Upload and delete operations require lecturer permissions
/// - List and download operations require module assignment
///
/// # Returns
/// An [`axum::Router`] with the file endpoints and their associated middleware.
pub fn files_routes(app_state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            "/",
            post(upload_files).route_layer(from_fn_with_state(app_state.clone(), allow_lecturer)),
        )
        .route(
            "/",
            get(list_files).route_layer(from_fn_with_state(
                app_state.clone(),
                allow_student,
            )),
        )
        .route(
            "/",
            delete(delete_files)
                .route_layer(from_fn_with_state(app_state.clone(), allow_lecturer)),
        )
        .route(
            "/{file_id}",
            get(download_file).route_layer(from_fn_with_state(
                app_state.clone(),
                allow_student,
            )),
        )
}
