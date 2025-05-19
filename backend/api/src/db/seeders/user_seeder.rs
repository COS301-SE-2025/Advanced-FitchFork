use crate::db::factories::user_factory;
use sqlx::SqlitePool;

pub async fn seed(pool: &SqlitePool) {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await
        .unwrap();

    if count.0 > 0 {
        log::info!("Skipping user seeder.");
        return;
    }

    log::info!("Seeding users...");

    for _ in 0..5 {
        user_factory::make(pool).await;
    }

    log::info!("Users seeded.");
}
