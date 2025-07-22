use axum::{Router, middleware::from_fn_with_state};
use api::routes::routes;
use api::auth::guards::validate_known_ids;
use sea_orm::DatabaseConnection;

pub fn make_app(db: DatabaseConnection) -> Router {
    Router::new()
        .nest("/api", routes(db.clone()))
        .with_state(db.clone())
        .layer(from_fn_with_state(db, validate_known_ids))
} 