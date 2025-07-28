// modules/personnel/mod.rs
use axum::{Router, routing::{get, post, delete}};
use sea_orm::DatabaseConnection;

mod get;
mod post;
mod delete;

pub fn personnel_routes() -> Router<DatabaseConnection> {
    Router::new()
        .route("/", get(get::get_personnel))
        .route("/", post(post::assign_personnel))
        .route("/", delete(delete::remove_personnel))
        .route("/eligible", get(get::get_eligible_users_for_module))
}
