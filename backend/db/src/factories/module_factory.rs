use crate::models::module::Module;
use sqlx::SqlitePool;

pub async fn make(code: &str, year: i64, description: Option<&str>, pool: &SqlitePool) -> Module {
    let record: Module = sqlx::query_as::<_, Module>(
        "INSERT INTO modules (code, year, description)
         VALUES (?, ?, ?)
         RETURNING id, code, year, description",
    )
    .bind(code)
    .bind(year)
    .bind(&description)
    .fetch_one(pool)
    .await
    .expect("Failed to create module");

    record
}
