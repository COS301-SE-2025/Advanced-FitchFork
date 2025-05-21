pub mod assignment_seeder;
pub mod module_seeder;
pub mod user_seeder;

use sqlx::SqlitePool;

pub async fn seed(pool: &SqlitePool) {
    user_seeder::seed(pool).await;
    module_seeder::seed(pool).await;
    assignment_seeder::seed(pool).await;
}
