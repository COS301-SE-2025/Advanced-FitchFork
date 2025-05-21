use crate::models::user::User;
use fake::{faker::internet::en::FreeEmail, faker::lorem::en::Word, Fake};
use rand::seq::SliceRandom;
use rand::{distributions::Uniform, Rng};
use sqlx::SqlitePool;

//Inserts randomly made user into database and returns them
pub async fn make_random(pool: &SqlitePool) -> User {
    let student_number = generate_student_number();
    let email: String = FreeEmail().fake();
    let password_hash: String = Word().fake();
    let admin = rand::random::<bool>();

    User::create(pool, &student_number, &email, &password_hash, admin)
        .await
        .expect("Failed to create user")
}

//Creates a randomly made user and assigns it as a lecturer to a module
pub async fn make_random_lecturer(pool: &SqlitePool) -> (User, i64) {
    let user = make_random(pool).await;
    let module_id = random_module_id(pool).await;

    sqlx::query("INSERT INTO module_lecturers (module_id, user_id) VALUES (?, ?)")
        .bind(module_id)
        .bind(user.id)
        .execute(pool)
        .await
        .expect("Failed to assign lecturer");

    (user, module_id)
}

//Creates a randomly made user and assigns it as a tutor to a module
pub async fn make_random_tutor(pool: &SqlitePool) -> (User, i64) {
    let user = make_random(pool).await;
    let module_id = random_module_id(pool).await;

    sqlx::query("INSERT INTO module_tutors (module_id, user_id) VALUES (?, ?)")
        .bind(module_id)
        .bind(user.id)
        .execute(pool)
        .await
        .expect("Failed to assign tutor");

    (user, module_id)
}

//Creates a randomly made user and assigns it as a student to a module
pub async fn make_random_student(pool: &SqlitePool) -> (User, i64) {
    let user = make_random(pool).await;
    let module_id = random_module_id(pool).await;

    sqlx::query("INSERT INTO module_students (module_id, user_id) VALUES (?, ?)")
        .bind(module_id)
        .bind(user.id)
        .execute(pool)
        .await
        .expect("Failed to assign student");

    (user, module_id)
}

//=====PRIVATE HELPER FUNCTIONS=====

//Generates random student number that conforms to Regex
fn generate_student_number() -> String {
    let mut rng = rand::thread_rng();
    //Low is 10000000 and since it has to be 8 digits (regex in database) -> 00000000 will cause student numbers like 172
    let number: u32 = rng.sample(Uniform::new_inclusive(10000000, 99999999));
    format!("u{}", number)
}

//Pick a random module id from the database
async fn random_module_id(pool: &SqlitePool) -> i64 {
    let ids: Vec<(i64,)> = sqlx::query_as("SELECT id FROM modules")
        .fetch_all(pool)
        .await
        .expect("Failed to fetch modules");

    let mut rng = rand::thread_rng();
    let (random_id,) = ids.choose(&mut rng).expect("No modules found");
    *random_id
}
