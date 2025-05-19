use crate::db::models::user::User;
use fake::{Fake, faker::internet::en::FreeEmail, faker::lorem::en::Word};
use rand::seq::SliceRandom;
use sqlx::SqlitePool;

/// Creates and inserts a random user into the database.
/// Returns the inserted User.
pub async fn make(pool: &SqlitePool) -> User {
    let email: String = FreeEmail().fake();
    let password: String = Word().fake(); // simple fake password
    let roles = ["admin", "tutor", "student"];
    let role = roles.choose(&mut rand::thread_rng()).unwrap();

    User::create(pool, &email, &password, role)
        .await
        .expect("Failed to create user")
}
