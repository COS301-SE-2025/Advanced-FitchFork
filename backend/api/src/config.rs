use once_cell::sync::OnceCell;
use serde::Deserialize;
use std::{env, fs};

#[derive(Debug, Deserialize)]
pub struct ApiConfig {
    pub project_name: String,
    pub log_level: String,
    pub log_file: String,
    pub database_url: String,
    pub host: String,
    pub port: u16,
}

static CONFIG: OnceCell<ApiConfig> = OnceCell::new();

impl ApiConfig {
    pub fn init(env_path: &str) -> &'static Self {
        dotenvy::from_filename(env_path).ok();

        CONFIG.get_or_init(|| {
            let project_name = env::var("PROJECT_NAME").unwrap_or_else(|_| "markr-api".into());
            let log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "debug".into());
            let log_file = env::var("LOG_FILE").unwrap_or_else(|_| "logs/api.log".into());
            let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
            let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".into());
            let port = env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3000);

            if let Some(parent) = std::path::Path::new(&log_file).parent() {
                fs::create_dir_all(parent).expect("Failed to create log directory");
            }

            ApiConfig {
                project_name,
                log_level,
                log_file,
                database_url,
                host,
                port,
            }
        })
    }

    pub fn get() -> &'static Self {
        CONFIG.get().expect("ApiConfig not initialized")
    }
}
