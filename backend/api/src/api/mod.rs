pub mod health;
pub mod users;

use axum::Router;

pub fn routes() -> Router {
    Router::new().nest("/api", health::routes().merge(users::routes()))
}
