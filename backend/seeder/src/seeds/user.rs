use crate::seed::Seeder;
use db::models::user::Model;
use fake::{Fake, faker::internet::en::SafeEmail};
use sea_orm::DatabaseConnection;
use std::pin::Pin;

pub struct UserSeeder;

impl Seeder for UserSeeder {
    fn seed<'a>(&'a self, _db: &'a DatabaseConnection) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            // Fixed Admin User
            let _ = UserService::create(CreateUser {
                username: "admin".to_string(),
                email: "admin@example.com".to_string(),
                password: "1".to_string(),
                admin: true,
            }).await;

            // Fixed Lecturer User
            let _ = UserService::create(CreateUser {
                username: "lecturer".to_string(),
                email: "lecturer@example.com".to_string(),
                password: "1".to_string(),
                admin: false,
            }).await;

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
                let _ = UserService::create(CreateUser {
                    username: "tutor".to_string(),
                    email: "tutor@example.com".to_string(),
                    password: "1".to_string(),
                    admin: false,
                }).await;

            // Fixed Student User
            let _ = Model::create(db, "student", "student@example.com", "1", false).await;

            // Composite-role users
            let _ = Model::create(db, "student_tutor", "student_tutor@example.com", "1", false).await;
            let _ = Model::create(db, "all_staff", "all_staff@example.com", "1", false).await;
            let _ = Model::create(db, "lecturer_assistant", "lecturer_assistant@example.com", "1", false).await;

            // User with every role (distributed across modules)
            let _ = Model::create(db, "all", "all@example.com", "1", false).await;

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
        })
    }
}