use crate::factories::user_factory;
use sqlx::SqlitePool;
use crate::models::user::User;

pub async fn seed(pool: &SqlitePool) {
    log::info!("Seeding users...");

    // Test Admin User
    let _  = User::create(Some(pool), "u99999999", "u9999999@tuks.co.za", "test1234", true).await;

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
