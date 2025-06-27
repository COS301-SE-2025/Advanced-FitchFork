use sea_orm::{Database, DatabaseConnection};
use migration::Migrator;
use sea_orm_migration::MigratorTrait;

pub async fn setup_test_db() -> DatabaseConnection {
    let db = Database::connect("sqlite::memory:")
        .await
        .expect("Failed to connect to in-memory db");

    Migrator::up(&db, None)
        .await
        .expect("Failed to run migrations");

    db
}
