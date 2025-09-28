use crate::seed::Seeder;
use db::models::user::Model;
use fake::{Fake, faker::internet::en::SafeEmail};
use sea_orm::DatabaseConnection;

pub struct UserSeeder;

#[async_trait::async_trait]
impl Seeder for UserSeeder {
    async fn seed(&self, db: &DatabaseConnection) {
        // Fixed Admin User
        let _ = Model::create(db, "admin", "admin@example.com", "owca", true).await;

        // Fixed Lecturer User
        let _ = Model::create(db, "lecturer", "lecturer@example.com", "owca", false).await;

        // Fixed Assistant Lecturer User
        let _ = Model::create(
            db,
            "assistant_lecturer",
            "assistant_lecturer@example.com",
            "1",
            false,
        )
        .await;

        // Fixed Tutor User
        let _ = Model::create(db, "tutor", "tutor@example.com", "owca", false).await;

        // Fixed Student User
        let _ = Model::create(db, "student", "student@example.com", "owca", false).await;

        // Composite-role users
        let _ = Model::create(
            db,
            "student_tutor",
            "student_tutor@example.com",
            "owca",
            false,
        )
        .await;
        let _ = Model::create(db, "all_staff", "all_staff@example.com", "owca", false).await;
        let _ = Model::create(
            db,
            "lecturer_assistant",
            "lecturer_assistant@example.com",
            "owca",
            false,
        )
        .await;

        // User with every role (distributed across modules)
        let _ = Model::create(db, "all", "all@example.com", "owca", false).await;

        // Fixed Demo User
        let _ = Model::create(db, "demo", "demo@example.com", "demo123", false).await;

        // Random Users
        for _ in 0..100 {
            let username = format!("u{:08}", fastrand::u32(..100_000_000));
            let email: String = SafeEmail().fake();
            let _ = Model::create_fake_user_with_no_hashed_password_do_not_use(
                db,
                &username,
                &email,
                "password_hash",
                false,
            )
            .await;
        }
    }
}
