use sqlx::SqlitePool;

pub async fn seed(pool: &SqlitePool) {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await
        .unwrap();

    if count.0 > 0 {
        log::info!("Skipping seed: users already exist.");
        return;
    }

    log::info!("Seeding dev data...");

    sqlx::query("INSERT INTO users (email, password, role) VALUES (?, ?, ?)")
        .bind("admin@example.com")
        .bind("admin123")
        .bind("admin")
        .execute(pool)
        .await
        .unwrap();

    // Add modules, assignments, etc. here
}
