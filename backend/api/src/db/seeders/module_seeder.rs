use crate::db::factories::module_factory;
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

    let modules = vec![
        module_factory::make_module("COS314", Some("Artificial Intelligence")),
        module_factory::make_module("COS333", Some("Software Engineering")),
    ];

    for module in modules {
        sqlx::query("INSERT INTO modules (name, description, created_at) VALUES (?, ?, ?)")
            .bind(module.name)
            .bind(module.description)
            .bind(module.created_at)
            .execute(pool)
            .await
            .unwrap();
    }

    log::info!("Modules seeded.");
}
