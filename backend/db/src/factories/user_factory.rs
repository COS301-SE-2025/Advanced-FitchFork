use crate::models::user::User;
use fake::{faker::internet::en::FreeEmail, faker::lorem::en::Word, Fake};
use rand::seq::SliceRandom;
use rand::{distributions::Uniform, Rng};
use sqlx::{Sqlite, SqlitePool, Transaction};

//Inserts randomly made user into database and returns them
pub async fn make_random(pool: &SqlitePool) -> User {
    let student_number = generate_student_number();
    let email: String = FreeEmail().fake();
    let password_hash: String = Word().fake();
    let admin = rand::random::<bool>();

    User::create(Some(&pool), &student_number, &email, &password_hash, admin)
        .await
        .expect("Failed to create user")
}

//Assigns user to a random number of modules in the specified role table using a transaction
async fn assign_to_random_modules(
    tx: &mut Transaction<'_, Sqlite>,
    user_id: i64,
    table: &str,
    module_ids: &[i64],
) {
    let mut rng = rand::thread_rng();
    let num_modules = rng.gen_range(1..=module_ids.len().min(3));
    let chosen_ids = module_ids
        .choose_multiple(&mut rng, num_modules)
        .cloned()
        .collect::<Vec<_>>();

    for module_id in chosen_ids {
        let query = format!("INSERT INTO {} (module_id, user_id) VALUES (?, ?)", table);
        sqlx::query(&query)
            .bind(module_id)
            .bind(user_id)
            .execute(&mut **tx)
            .await
            .unwrap_or_else(|_| panic!("Failed to assign user to {}", table));
    }
}

//Creates a randomly made user and assigns it as a lecturer to multiple modules
pub async fn make_random_lecturer(pool: &SqlitePool, module_ids: &[i64]) -> User {
    let mut tx = pool.begin().await.unwrap();
    let user = make_random(pool).await;
    assign_to_random_modules(&mut tx, user.id, "module_lecturers", module_ids).await;
    tx.commit().await.unwrap();
    user
}

//Creates a randomly made user and assigns it as a tutor to multiple modules
pub async fn make_random_tutor(pool: &SqlitePool, module_ids: &[i64]) -> User {
    let mut tx = pool.begin().await.unwrap();
    let user = make_random(pool).await;
    assign_to_random_modules(&mut tx, user.id, "module_tutors", module_ids).await;
    tx.commit().await.unwrap();
    user
}

//Creates a randomly made user and assigns it as a student to multiple modules
pub async fn make_random_student(pool: &SqlitePool, module_ids: &[i64]) -> User {
    let mut tx = pool.begin().await.unwrap();
    let user = make_random(pool).await;
    assign_to_random_modules(&mut tx, user.id, "module_students", module_ids).await;
    tx.commit().await.unwrap();
    user
}

//=====PRIVATE HELPER FUNCTIONS=====

fn generate_student_number() -> String {
    let mut rng = rand::thread_rng();
    let number: u32 = rng.sample(Uniform::new_inclusive(10000000, 99999999));
    format!("u{}", number)
}

//Fetch all module IDs from the database once
pub async fn all_module_ids(pool: &SqlitePool) -> Vec<i64> {
    let ids: Vec<(i64,)> = sqlx::query_as("SELECT id FROM modules")
        .fetch_all(pool)
        .await
        .expect("Failed to fetch modules");

    ids.into_iter().map(|(id,)| id).collect()
}
