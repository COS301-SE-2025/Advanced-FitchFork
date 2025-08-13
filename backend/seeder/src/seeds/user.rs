use crate::seed::Seeder;
use db::repositories::user_repository::UserRepository;
use services::{
    service::Service,
    user_service::{UserService, CreateUser},
};
use fake::{faker::internet::en::SafeEmail, Fake};
use sea_orm::DatabaseConnection;
use std::pin::Pin;

pub struct UserSeeder;

impl Seeder for UserSeeder {
    fn seed<'a>(&'a self, db: &'a DatabaseConnection) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let service = UserService::new(UserRepository::new(db.clone()));

            // Fixed Admin User
            let _ = service.create(CreateUser {
                username: "admin".to_string(),
                email: "admin@example.com".to_string(),
                password: "1".to_string(),
                admin: true,
            }).await;

            // Fixed Lecturer User
            let _ = service.create(CreateUser {
                username: "lecturer".to_string(),
                email: "lecturer@example.com".to_string(),
                password: "1".to_string(),
                admin: false,
            }).await;

            // Fixed Assistant Lecturer User
            let _ = service.create(CreateUser {
                username: "assistant_lecturer".to_string(),
                email: "assistant_lecturer@example.com".to_string(),
                password: "1".to_string(),
                admin: false,
            }).await;

            // Fixed Tutor User
            let _ = service.create(CreateUser {
                username: "tutor".to_string(),
                email: "tutor@example.com".to_string(),
                password: "1".to_string(),
                admin: false,
            }).await;

            // Fixed Student User
            let _ = service.create(CreateUser {
                username: "student".to_string(),
                email: "student@example.com".to_string(),
                password: "1".to_string(),
                admin: false,
            }).await;

            // Random Users
            for _ in 0..10 {
                let username = format!("u{:08}", fastrand::u32(..100_000_000));
                let email: String = SafeEmail().fake();
                let _ = service.create(CreateUser {
                    username,
                    email,
                    password: "password_hash".to_string(),
                    admin: false,
                }).await;
            }
        })
    }
}
