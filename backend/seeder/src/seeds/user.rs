use crate::seed::Seeder;
use db::models::user::Model;
use fake::{faker::internet::en::SafeEmail, Fake};
use sea_orm::DatabaseConnection;

pub struct UserSeeder;

#[async_trait::async_trait]
impl Seeder for UserSeeder {
    async fn seed(&self, db: &DatabaseConnection) {
        // Fixed Admin User
        let _ = Model::create(db, "admin", "admin@example.com", "1", true).await;

        // Fixed Lecturer User
        let _ = Model::create(db, "lecturer", "lecturer@example.com", "1", false).await;

        // Fixed Assistant Lecturer User
        let _ = Model::create(db, "assistant_lecturer", "assistant_lecturer@example.com", "1", false).await;

        // Fixed Tutor User
        let _ = Model::create(db, "tutor", "tutor@example.com", "1", false).await;

        // Fixed Student User
        let _ = Model::create(db, "student", "student@example.com", "1", false).await;

        // Random Users
        for _ in 0..10 {
            let username = format!("u{:08}", fastrand::u32(..100_000_000));
            let email: String = SafeEmail().fake();
            let _ = Model::create(db, &username, &email, "password_hash", false).await;
        }
    }
}
