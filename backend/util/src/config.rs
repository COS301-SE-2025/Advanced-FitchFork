//! Global application configuration manager.
//!
//! `AppConfig` is a lazily initialized, globally accessible singleton containing
//! runtime configuration values loaded from environment variables. It provides
//! thread-safe access and mutation for testing or overrides in runtime environments.

use std::env;
use std::sync::{OnceLock, RwLock};

/// Represents the complete application configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub env: String,
    pub project_name: String,
    pub log_level: String,
    pub log_file: String,
    pub log_to_stdout: bool,
    pub database_path: String,
    pub assignment_storage_root: String,
    pub host: String,
    pub port: u16,
    pub code_manager_host: String,
    pub code_manager_port: u16,
    pub jwt_secret: String,
    pub jwt_duration_minutes: u64,
    pub reset_token_expiry_minutes: u64,
    pub max_password_reset_requests_per_hour: u32,
    pub gmail_username: String,
    pub gmail_app_password: String,
    pub frontend_url: String,
    pub email_from_name: String,
    pub gemini_api_key: String,
}

/// Lazily-initialized, thread-safe singleton instance of `AppConfig`.
static CONFIG_INSTANCE: OnceLock<RwLock<AppConfig>> = OnceLock::new();

impl AppConfig {
    /// Loads the configuration from `.env` and environment variables.
    ///
    /// This method is used internally to populate the singleton. It panics
    /// if required variables are missing or improperly formatted.
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        Self {
            env: env::var("APP_ENV").unwrap_or_else(|_| "development".into()),
            project_name: env::var("PROJECT_NAME").unwrap_or_else(|_| "fitch-fork".into()),
            log_level: env::var("LOG_LEVEL").unwrap_or_else(|_| "api=info".into()),
            log_file: env::var("LOG_FILE").unwrap_or_else(|_| "api.log".into()),
            log_to_stdout: env::var("LOG_TO_STDOUT").unwrap_or_else(|_| "false".into()) == "true",
            database_path: env::var("DATABASE_PATH").expect("DATABASE_PATH is required"),
            assignment_storage_root: env::var("ASSIGNMENT_STORAGE_ROOT")
                .expect("ASSIGNMENT_STORAGE_ROOT is required"),
            host: env::var("HOST").unwrap_or_else(|_| "127.0.0.1".into()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "3000".into())
                .parse()
                .unwrap(),
            code_manager_host: env::var("CODE_MANAGE_HOST").unwrap_or_else(|_| "127.0.0.1".into()),
            code_manager_port: env::var("CODE_MANAGE_PORT")
                .unwrap_or_else(|_| "3001".into())
                .parse()
                .unwrap(),
            jwt_secret: env::var("JWT_SECRET").expect("JWT_SECRET is required"),
            jwt_duration_minutes: env::var("JWT_DURATION_MINUTES")
                .unwrap_or("60".into())
                .parse()
                .unwrap(),
            reset_token_expiry_minutes: env::var("RESET_TOKEN_EXPIRY_MINUTES")
                .unwrap_or("15".into())
                .parse()
                .unwrap(),
            max_password_reset_requests_per_hour: env::var("MAX_PASSWORD_RESET_REQUESTS_PER_HOUR")
                .unwrap_or("3".into())
                .parse()
                .unwrap(),
            gmail_username: env::var("GMAIL_USERNAME").unwrap_or_default(),
            gmail_app_password: env::var("GMAIL_APP_PASSWORD").unwrap_or_default(),
            frontend_url: env::var("FRONTEND_URL").unwrap_or_default(),
            email_from_name: env::var("EMAIL_FROM_NAME").unwrap_or_else(|_| "FitchFork".into()),
            gemini_api_key: env::var("GEMINI_API_KEY").unwrap_or_default(),
        }
    }

    /// Returns a shared reference to the global configuration.
    ///
    /// # Panics
    /// Panics if the lock cannot be acquired.
    pub fn global() -> std::sync::RwLockReadGuard<'static, AppConfig> {
        CONFIG_INSTANCE
            .get_or_init(|| RwLock::new(AppConfig::from_env()))
            .read()
            .expect("Failed to acquire AppConfig read lock")
    }

    /// Resets the configuration by reloading from environment variables.
    ///
    /// Useful in tests to clear overrides.
    pub fn reset() {
        if let Some(lock) = CONFIG_INSTANCE.get() {
            let mut guard = lock.write().unwrap();
            *guard = AppConfig::from_env();
        }
    }

    /// Generic internal setter for any field in the config.
    ///
    /// Used by public per-field setter methods.
    fn set_field<F>(setter: F)
    where
        F: FnOnce(&mut AppConfig),
    {
        let lock = CONFIG_INSTANCE.get_or_init(|| RwLock::new(AppConfig::from_env()));
        let mut guard = lock
            .write()
            .expect("Failed to acquire AppConfig write lock");
        setter(&mut guard);
    }

    // --- Per-field setters below ---

    /// Override `env` value.
    pub fn set_env(value: impl Into<String>) {
        AppConfig::set_field(|cfg| cfg.env = value.into());
    }

    pub fn set_project_name(value: impl Into<String>) {
        AppConfig::set_field(|cfg| cfg.project_name = value.into());
    }

    pub fn set_log_level(value: impl Into<String>) {
        AppConfig::set_field(|cfg| cfg.log_level = value.into());
    }

    pub fn set_log_file(value: impl Into<String>) {
        AppConfig::set_field(|cfg| cfg.log_file = value.into());
    }

    pub fn set_log_to_stdout(value: bool) {
        AppConfig::set_field(|cfg| cfg.log_to_stdout = value);
    }

    pub fn set_database_path(value: impl Into<String>) {
        AppConfig::set_field(|cfg| cfg.database_path = value.into());
    }

    pub fn set_assignment_storage_root(value: impl Into<String>) {
        AppConfig::set_field(|cfg| cfg.assignment_storage_root = value.into());
    }

    pub fn set_host(value: impl Into<String>) {
        AppConfig::set_field(|cfg| cfg.host = value.into());
    }

    pub fn set_port(value: u16) {
        AppConfig::set_field(|cfg| cfg.port = value);
    }

    pub fn set_jwt_secret(value: impl Into<String>) {
        AppConfig::set_field(|cfg| cfg.jwt_secret = value.into());
    }

    pub fn set_jwt_duration_minutes(value: impl Into<u64>) {
        AppConfig::set_field(|cfg| cfg.jwt_duration_minutes = value.into());
    }

    pub fn set_reset_token_expiry_minutes(value: impl Into<u64>) {
        AppConfig::set_field(|cfg| cfg.reset_token_expiry_minutes = value.into());
    }

    pub fn set_max_password_reset_requests_per_hour(value: impl Into<u32>) {
        AppConfig::set_field(|cfg| cfg.max_password_reset_requests_per_hour = value.into());
    }

    pub fn set_gmail_username(value: impl Into<String>) {
        AppConfig::set_field(|cfg| cfg.gmail_username = value.into());
    }

    pub fn set_gmail_app_password(value: impl Into<String>) {
        AppConfig::set_field(|cfg| cfg.gmail_app_password = value.into());
    }

    pub fn set_frontend_url(value: impl Into<String>) {
        AppConfig::set_field(|cfg| cfg.frontend_url = value.into());
    }

    pub fn set_email_from_name(value: impl Into<String>) {
        AppConfig::set_field(|cfg| cfg.email_from_name = value.into());
    }

    pub fn set_gemini_api_key(value: impl Into<String>) {
        AppConfig::set_field(|cfg| cfg.gemini_api_key = value.into());
    }
}
