use crate::factories::module_factory;
use sqlx::SqlitePool;

pub async fn seed(pool: &SqlitePool) {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM modules")
        .fetch_one(pool)
        .await
        .unwrap();

    if count.0 > 0 {
        log::info!("Skipping module seeder.");
        return;
    }

    log::info!("Seeding modules...");

    module_factory::make("COS314", 2025, Some("Artificial Intelligence"), pool).await;
    module_factory::make("COS333", 2025, Some("Software Engineering"), pool).await;

    log::info!("Modules seeded.");
}
