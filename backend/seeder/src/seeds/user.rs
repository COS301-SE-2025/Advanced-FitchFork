use crate::seed::Seeder;
use db::models::user::Model;
use fake::{faker::internet::en::SafeEmail, Fake};
use sea_orm::DatabaseConnection;

pub struct UserSeeder;

#[async_trait::async_trait]
impl Seeder for UserSeeder {
    async fn seed(&self, db: &DatabaseConnection) {
        // Fixed Admin User
        let _ = Model::create(db, "u00000001", "admin@example.com", "password123", true).await;

        // Fixed Normal User
        let _ = Model::create(db, "u00000002", "user@example.com", "password123", false).await;

        // Random Users
        for _ in 0..10 {
            let username = format!("u{:08}", fastrand::u32(..100_000_000));
            let email: String = SafeEmail().fake();
            let _ = Model::create(db, &username, &email, "password_hash", false).await;
        }
    }
}
