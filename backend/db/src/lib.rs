pub mod factories;
pub mod models;
pub mod pool;
pub mod seeders;

use sqlx::{SqlitePool, migrate::Migrator, sqlite::SqlitePoolOptions};
use std::fs;
use std::path::Path;

static MIGRATOR: Migrator = sqlx::migrate!("./src/migrations");

/// Public init function that connects, stores pool globally, and seeds DB
pub async fn init(database_url: &str) {
    let pool = connect_and_initialize(database_url).await;
    pool::set(pool);
    seeders::seed(pool::get()).await;
}

/// Connects to the SQLite DB using a given file path
async fn connect_and_initialize(database_path: &str) -> SqlitePool {
    prepare_sqlite_path(database_path);

    let pool = SqlitePoolOptions::new()
        .connect(&format!("sqlite://{}", database_path))
        .await
        .expect("Failed to connect to the database");

    MIGRATOR.run(&pool).await.expect("Failed to run migrations");

    pool
}

/// Ensures parent directory and empty .db file exist before SQLx connects
fn prepare_sqlite_path(database_path: &str) {
    let db_path = Path::new(database_path);

    if let Some(parent) = db_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).expect("Could not create database folder");
            println!("Created database directory: {}", parent.display());
        }
    }

    if !db_path.exists() {
        fs::File::create(&db_path).expect("Could not create SQLite database file");
        println!("Created database file: {}", db_path.display());
    }
}
