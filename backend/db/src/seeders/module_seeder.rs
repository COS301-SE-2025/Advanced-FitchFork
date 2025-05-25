use crate::factories::module_factory;
use sqlx::SqlitePool;

pub async fn seed(pool: &SqlitePool) {
    log::info!("Seeding modules...");

    module_factory::make("COS314", 2025, Some("Artificial Intelligence"), 16, pool).await;
    module_factory::make("COS333", 2025, Some("Software Engineering"), 16, pool).await;

    log::info!("Modules seeded.");
}
