use crate::db::models::user::User;
use crate::db::pool;
use axum::{Json, Router, response::IntoResponse, routing::get};

pub fn routes() -> Router {
    Router::new().route("/users", get(get_users))
}

async fn get_users() -> impl IntoResponse {
    let pool = pool::get();

    match sqlx::query_as::<_, User>("SELECT * FROM users")
        .fetch_all(pool)
        .await
    {
        Ok(users) => Json(users).into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to fetch users: {}", e),
        )
            .into_response(),
    }
}
