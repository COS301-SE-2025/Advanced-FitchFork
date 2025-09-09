use crate::seed::Seeder;
use services::service::Service;
use services::user_service::{UserService, CreateUser};
use fake::{Fake, faker::internet::en::SafeEmail};
use std::pin::Pin;

pub struct UserSeeder;

impl Seeder for UserSeeder {
    fn seed<'a>(&'a self) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
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
            let _ = UserService::create(CreateUser {
                username: "assistant_lecturer".to_string(),
                email: "assistant_lecturer@example.com".to_string(),
                password: "1".to_string(),
                admin: false,
            }).await;

            // Fixed Tutor User
            let _ = UserService::create(CreateUser {
                username: "tutor".to_string(),
                email: "tutor@example.com".to_string(),
                password: "1".to_string(),
                admin: false,
            }).await;

            // Fixed Student User
            let _ = UserService::create(CreateUser {
                username: "student".to_string(),
                email: "student@example.com".to_string(),
                password: "1".to_string(),
                admin: false,
            }).await;

            // Composite-role users
            let _ = UserService::create(CreateUser {
                username: "student_tutor".to_string(),
                email: "student_tutor@example.com".to_string(),
                password: "1".to_string(),
                admin: false,
            }).await;
            let _ = UserService::create(CreateUser {
                username: "all_staff".to_string(),
                email: "all_staff@example.com".to_string(),
                password: "1".to_string(),
                admin: false,
            }).await;
            let _ = UserService::create(CreateUser {
                username: "lecturer_assistant".to_string(),
                email: "lecturer_assistant@example.com".to_string(),
                password: "1".to_string(),
                admin: false,
            }).await;

            // User with every role (distributed across modules)
            let _ = UserService::create(CreateUser {
                username: "all".to_string(),
                email: "all@example.com".to_string(),
                password: "1".to_string(),
                admin: false,
            }).await;

            // Random Users
            for _ in 0..100 {
                let username = format!("u{:08}", fastrand::u32(..100_000_000));
                let email: String = SafeEmail().fake();
                let _ = UserService::create_fake_user_with_no_hashed_password_do_not_use(
                    &username,
                    &email,
                    "password_hash",
                    false,
                ).await;
            }
        })
    }
}