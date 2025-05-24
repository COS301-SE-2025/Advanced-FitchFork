pub mod assignment_seeder;
pub mod module_seeder;
pub mod user_seeder;

use sqlx::SqlitePool;

pub async fn seed(pool: &SqlitePool) {
    log::info!("Clearing all tables...");
    sqlx::query("DELETE FROM module_students").execute(pool).await.unwrap();
    sqlx::query("DELETE FROM module_tutors").execute(pool).await.unwrap();
    sqlx::query("DELETE FROM module_lecturers").execute(pool).await.unwrap();
    sqlx::query("DELETE FROM assignments").execute(pool).await.unwrap();
    sqlx::query("DELETE FROM modules").execute(pool).await.unwrap();
    sqlx::query("DELETE FROM users").execute(pool).await.unwrap();
    sqlx::query("DELETE FROM sqlite_sequence").execute(pool).await.unwrap();
    
    module_seeder::seed(pool).await;
    user_seeder::seed(pool).await;
    assignment_seeder::seed(pool).await;
}
