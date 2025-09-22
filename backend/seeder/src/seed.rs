use async_trait::async_trait;
use colored::*;
use futures::FutureExt;
use sea_orm::DatabaseConnection;
use std::io::{self, Write};
use std::pin::Pin;
use std::time::Instant;

const STATUS_COLUMN: usize = 80;

pub trait Seeder {
    fn seed<'a>(&'a self) -> Pin<Box<dyn Future<Output = Result<(), AppError>> + Send + 'a>>;
}

pub async fn run_seeder<S: Seeder + ?Sized>(seeder: &S, name: &str) {
    let base_msg = format!("Seeding {}", name.bold());
    let dots = ".".repeat(STATUS_COLUMN.saturating_sub(base_msg.len()));
    print!("{}{} ", base_msg, dots);
    io::stdout().flush().unwrap();

    let start = Instant::now();
    let duration = match std::panic::AssertUnwindSafe(seeder.seed(db))
        .catch_unwind()
        .await
    {
        Ok(_) => Some(start.elapsed()),
        Err(_) => {
            println!("{}", "failed".red());
            std::process::exit(1);
        }
    };

    let time_str = format!("({:.2?})", duration.unwrap()).dimmed();
    println!("{} {}", "done".green(), time_str);
}
