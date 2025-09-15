use migration::Migrator;
use std::{env, fs, path::Path};
use util::{config, paths::storage_root};

mod runner;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let db_path = config::database_path();
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
    // Delete DB file
    let db_path = Path::new(path);
    if db_path.exists() {
        fs::remove_file(db_path).expect("Failed to delete DB file");
        println!("Deleted DB: {}", db_path.display());
    } else {
        println!("DB file does not exist: {}", db_path.display());
    }

    // Delete entire storage tree (single STORAGE_ROOT via util::paths)
    let root = storage_root();
    if root.exists() {
        fs::remove_dir_all(&root).expect("Failed to delete storage root");
        println!("Deleted storage root: {}", root.display());
    } else {
        println!("Storage root does not exist: {}", root.display());
    }
}

fn create_db_dir(path: &str) {
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent).expect("Failed to create DB directory");
    }
}
