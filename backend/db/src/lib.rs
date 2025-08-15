pub mod models;
pub mod test_utils;
pub mod repositories;
pub mod filters;

use sea_orm::{Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;
use migration::Migrator;
use std::env;
use tokio::sync::OnceCell;
use util::state::AppState;

static DB_CONNECTION: OnceCell<DatabaseConnection> = OnceCell::const_new();

pub async fn get_connection() -> &'static DatabaseConnection {
    DB_CONNECTION
        .get_or_init(|| async {
            let state = AppState::get();
            let db_url = if state.is_test_mode() {
                "sqlite://test.db?mode=rwc".to_string() // TODO: Try to figure out how to make this in memory
            } else {
                env::var("DATABASE_PATH")
                    .map(|path| format!("sqlite://{}?mode=rwc", path))
                    .expect("DATABASE_PATH must be set in .env when TEST_MODE=false")
            };
            
            let db= Database::connect(&db_url)
                .await
                .expect("Failed to connect to database");

            Migrator::up(&db, None)
                .await
                .expect("Failed to run migrations");

            db
        })
        .await
}
pub use sea_orm::DbErr;