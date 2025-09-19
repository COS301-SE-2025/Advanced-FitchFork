pub mod models;
pub mod test_utils;

use sea_orm::{Database, DatabaseConnection};
use std::path::Path;
use util::config;

pub async fn connect() -> DatabaseConnection {
    let path_or_url = config::database_path(); // your env var
    // If it's already a DSN, use it as-is; otherwise treat it as a SQLite file path.
    let url = if path_or_url.starts_with("sqlite:")
        || path_or_url.starts_with("postgres://")
        || path_or_url.starts_with("mysql://")
    {
        path_or_url
    } else {
        // Ensure parent directory exists (SQLite won't create intermediate dirs).
        if let Some(parent) = Path::new(&path_or_url).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        format!("sqlite://{path_or_url}") // yields sqlite:///abs/path for absolute paths
    };

    Database::connect(&url)
        .await
        .expect("Failed to connect to database")
}
