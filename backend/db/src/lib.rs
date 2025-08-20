pub mod models;
pub mod test_utils;

use sea_orm::{Database, DatabaseConnection};
use std::env;

pub async fn connect() -> DatabaseConnection {
    let db_url = env::var("DATABASE_PATH")
        .map(|path| format!("sqlite://{}?mode=rwc", path))
        .expect("DATABASE_PATH must be set in .env");

    Database::connect(&db_url)
        .await
        .expect("Failed to connect to database")
}