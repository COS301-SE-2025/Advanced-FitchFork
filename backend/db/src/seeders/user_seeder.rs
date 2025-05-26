use crate::factories::user_factory;
use crate::models::module_lecturer::ModuleLecturer;
use crate::models::module_student::ModuleStudent;
use crate::models::module_tutor::ModuleTutor;
use crate::models::user::User;
use sqlx::SqlitePool;

//TODO -> make the first use (2) be a student, lecturer and tutor in different modules
pub async fn seed(pool: &SqlitePool) {
    log::info!("Seeding users...");

    // Fetch all module IDs once for reuse
    let module_ids = user_factory::all_module_ids(pool).await;
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
    )
    .await
    .expect("Failed to create regular user");

    ModuleLecturer::create(Some(pool), 1, 2)
        .await
        .expect("Failed");
    ModuleStudent::create(Some(pool), 2, 2)
        .await
        .expect("Failed");
    ModuleTutor::create(Some(pool), 3, 2).await.expect("Failed");

    // Create 5 users without roles (admin and non-admin)
    for _ in 0..5 {
        user_factory::make_random(pool).await;
    }

    // Create 5 lecturers
    log::info!("Seeding module lecturers...");
    for _ in 0..5 {
        user_factory::make_random_lecturer(pool, &module_ids).await;
    }

    // Create 5 tutors
    log::info!("Seeding module tutors...");
    for _ in 0..5 {
        user_factory::make_random_tutor(pool, &module_ids).await;
    }

    // Create 5 students
    log::info!("Seeding module students...");
    for _ in 0..5 {
        user_factory::make_random_student(pool, &module_ids).await;
    }

    log::info!("Users seeded.");
}
