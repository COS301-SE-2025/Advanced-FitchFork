use sea_orm::DatabaseConnection;
use colored::*;
use futures::FutureExt;
use std::io::{self, Write};
use std::time::Instant;
use std::pin::Pin;

const STATUS_COLUMN: usize = 80;

pub trait Seeder {
    fn seed<'a>(&'a self, db: &'a DatabaseConnection) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>>;
}

pub async fn run_seeder<S: Seeder + ?Sized>(seeder: &S, name: &str, db: &DatabaseConnection) {
    let base_msg = format!("Seeding {}", name.bold());
    let dots = ".".repeat(STATUS_COLUMN.saturating_sub(base_msg.len()));
    print!("{}{} ", base_msg, dots);
    io::stdout().flush().unwrap();

    let start = Instant::now();
    let duration = match std::panic::AssertUnwindSafe(seeder.seed(db)).catch_unwind().await {
        Ok(_) => Some(start.elapsed()),
        Err(_) => {
            println!("{}", "failed".red());
            std::process::exit(1);
        }
    };

    let time_str = format!("({:.2?})", duration.unwrap()).dimmed();
    println!("{} {}", "done".green(), time_str);
}