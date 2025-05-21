use crate::models::user::User;
use fake::{faker::internet::en::FreeEmail, faker::lorem::en::Word, Fake};
use rand::{distributions::Uniform, Rng};
use sqlx::SqlitePool;

///Inserts randomly made user into database and returns them
pub async fn make(pool: &SqlitePool) -> User {
    let student_number = generate_student_number();
    let email: String = FreeEmail().fake();
    let password_hash: String = Word().fake();
    let admin = rand::random::<bool>();

    User::create(pool, &student_number, &email, &password_hash, admin)
        .await
        .expect("Failed to create user")
}

//Generates random student number that conforms to Regex
fn generate_student_number() -> String {
    let mut rng = rand::thread_rng();
    let number: u32 = rng.sample(Uniform::new_inclusive(00000000, 99999999));
    format!("u{}", number)
}
