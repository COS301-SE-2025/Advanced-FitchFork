use migration::{Migrator};
use std::{env, fs, path::Path};

mod runner;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let db_path = env::var("DATABASE_PATH").expect("DATABASE_PATH must be set");
    let url = format!("sqlite://{}?mode=rwc", db_path);
    let args: Vec<String> = env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("clean") => {
            remove_db_file(&db_path);
        }
        Some("fresh") => {
            remove_db_file(&db_path);
            create_db_dir(&db_path);
            runner::run_all_migrations(&url).await;
        }
        _ => {
            create_db_dir(&db_path);
            runner::run_all_migrations(&url).await;
        }
    }
}

fn remove_db_file(path: &str) {
    let db_path = Path::new(path);
    if db_path.exists() {
        fs::remove_file(db_path).expect("Failed to delete DB file");
        println!("Deleted DB: {}", db_path.display());
    } else {
        println!("DB file does not exist: {}", db_path.display());
    }

    if let Ok(storage_root) = env::var("ASSIGNMENT_STORAGE_ROOT") {
        let storage_path = Path::new(&storage_root);
        if storage_path.exists() {
            fs::remove_dir_all(storage_path).expect("Failed to delete assignment files");
            println!("Deleted assignment files: {}", storage_path.display());
        }
    }
}

fn create_db_dir(path: &str) {
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent).expect("Failed to create DB directory");
    }
}
