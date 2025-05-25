use crate::factories::user_factory;
use crate::models::user::User;
use sqlx::SqlitePool;

pub async fn seed(pool: &SqlitePool) {
    log::info!("Seeding users...");

    // Explicit admin user
    log::info!("Creating explicit admin user...");
    User::create(
        Some(pool),
        "u00000001",
        "admin@example.com",
        "password123",
        true,
    )
    .await
    .expect("Failed to create admin user");

    // Explicit non-admin user
    log::info!("Creating explicit non-admin user...");
    User::create(
        Some(pool),
        "u00000002",
        "user@example.com",
        "password123",
        false,
    ).await
    .expect("Failed to create regular user");

    //Create 5 users without roles (admin and not admin)
    for _ in 0..5 {
        user_factory::make_random(pool).await;
    }

    //Create 5 lecturers
    log::info!("Seeding module lecturers...");
    for _ in 0..5 {
        user_factory::make_random_lecturer(pool).await;
    }

    //Create 5 tutors
    log::info!("Seeding module tutors...");
    for _ in 0..5 {
        user_factory::make_random_tutor(pool).await;
    }

    //Create 5 students
    log::info!("Seeding module students...");
    for _ in 0..5 {
        user_factory::make_random_student(pool).await;
    }

    log::info!("Users seeded.");
}
