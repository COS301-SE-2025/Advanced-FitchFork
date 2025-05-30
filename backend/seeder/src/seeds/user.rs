use fake::{Fake, faker::internet::en::SafeEmail};
use sea_orm::{DatabaseConnection};
use db::models::user::Model;
use crate::seed::Seeder;

pub struct UserSeeder;

#[async_trait::async_trait]
impl Seeder for UserSeeder {
    async fn seed(&self, db: &DatabaseConnection) {
        // Fixed Admin User
        let _ = Model::create(
            db,
            "u00000001",
            "admin@example.com",
            "password123",
            true,
        )
        .await;

        // Fixed Normal User
        let _ = Model::create(
            db,
            "u00000002",
            "user@example.com",
            "password123",
            false,
        )
        .await;

        // Random Users
        for _ in 0..20 {
            let student_number = format!("u{:08}", fastrand::u32(..100_000_000));
            let email: String = SafeEmail().fake();
            let _ = Model::create(
                db,
                &student_number,
                &email,
                "password_hash",
                false,
            )
            .await;
        }
    }
}
