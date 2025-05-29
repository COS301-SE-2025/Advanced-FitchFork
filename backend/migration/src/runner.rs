use colored::*;
use futures::FutureExt;
use sea_orm_migration::prelude::*;
use std::io::{self, Write};
use std::time::Instant;

const STATUS_COLUMN: usize = 80;

pub async fn run_all_migrations(url: &str) {
    let db = sea_orm::Database::connect(url)
        .await
        .expect("DB connection failed");

    println!("Running migrations...");
    let schema_manager = SchemaManager::new(&db);

    for migration in <crate::Migrator as MigratorTrait>::migrations() {
        run_migration(&schema_manager, migration).await;
    }
}

async fn run_migration(
    schema_manager: &SchemaManager<'_>,
    migration: Box<dyn MigrationTrait>,
) {
    let name_str = format!("Applying {}", migration.name().bold());
    let dots = ".".repeat(STATUS_COLUMN.saturating_sub(name_str.len()));
    print!("{}{} ", name_str, dots);
    io::stdout().flush().unwrap();

    let start = Instant::now();
    let result = std::panic::AssertUnwindSafe(migration.up(schema_manager))
        .catch_unwind()
        .await;

    match result {
        Ok(_) => {
            let time_str = format!("({:.2?})", start.elapsed()).dimmed();
            println!("{} {}", "done".green(), time_str);
        }
        Err(_) => {
            println!("{}", "failed".red());
            std::process::exit(1);
        }
    }
}
