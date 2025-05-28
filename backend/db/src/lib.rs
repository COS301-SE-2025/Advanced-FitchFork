pub mod factories;
pub mod models;
pub mod pool;
pub mod seeders;

use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{migrate::MigrateError, migrate::Migrator, sqlite::SqlitePoolOptions, SqlitePool};
use std::fs;
use std::path::Path;
use std::str::FromStr;

static MIGRATOR: Migrator = sqlx::migrate!("./src/migrations");

pub async fn init(database_url: &str, is_dev: bool) {
    let pool = connect_and_initialize(database_url, is_dev).await;
    pool::set(pool);
}

pub async fn seed_db() {
    seeders::seed(pool::get()).await;
}

async fn connect_and_initialize(database_path: &str, is_dev: bool) -> SqlitePool {
    prepare_sqlite_path(database_path);

    let connection_str = format!("sqlite://{}", database_path);
    let pool = SqlitePoolOptions::new()
        .connect(&connection_str)
        .await
        .expect("Failed to connect to the database");

    match MIGRATOR.run(&pool).await {
        Ok(_) => pool,
        Err(MigrateError::VersionMismatch(_)) if is_dev => {
            eprintln!("⚠️ Migration mismatch. Resetting dev DB...");
            handle_version_mismatch_and_reconnect(database_path).await
        }
        Err(err) => panic!("❌ Failed to run migrations: {}", err),
    }
}

async fn handle_version_mismatch_and_reconnect(database_path: &str) -> SqlitePool {
    delete_database(database_path);
    prepare_sqlite_path(database_path);

    let connection_str = format!("sqlite://{}", database_path);
    let pool = SqlitePoolOptions::new()
        .connect(&connection_str)
        .await
        .expect("Failed to reconnect to the database after reset");

    MIGRATOR
        .run(&pool)
        .await
        .expect("Failed to re-run migrations after resetting DB");

    pool
}

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

pub fn delete_database(database_path: &str) {
    let db_path = Path::new(database_path);
    if db_path.exists() {
        fs::remove_file(db_path).expect("Failed to delete database file");
        println!("Deleted database file: {}", db_path.display());
    }
}

pub async fn create_test_db(path: Option<&str>) -> SqlitePool {
    let (_, options) = match path {
        Some(p) => {
            let url = format!("sqlite://{}", p);
            let opts = SqliteConnectOptions::from_str(&url)
                .unwrap()
                .create_if_missing(true);
            (url, opts)
        }
        None => {
            let url = "sqlite::memory:".to_string();
            let opts = SqliteConnectOptions::from_str(&url).unwrap();
            (url, opts)
        }
    };

    let pool = SqlitePoolOptions::new()
        .max_connections(1) // REQUIRED for sqlite::memory:
        .connect_with(options)
        .await
        .expect("Failed to connect to test database");

    MIGRATOR
        .run(&pool)
        .await
        .expect("Failed to run test migrations");

    pool
}
