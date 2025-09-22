//! App config: on-demand env getters + optional full snapshot.
//! No global singleton; each call reads current process env.
//! All variables are REQUIRED.

use std::str::FromStr;
use std::sync::Once;

#[inline]
fn ensure_dotenv() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        // Avoid loading .env for unit tests in this crate.
        if !cfg!(test) {
            let _ = dotenvy::dotenv();
        }
    });
}

#[inline]
fn require(k: &'static str) -> String {
    match std::env::var(k) {
        Ok(v) if !v.is_empty() => v,
        _ => panic!("{k} is required"),
    }
}

#[inline]
fn parse<T: FromStr>(s: String, name: &'static str) -> T
where
    <T as FromStr>::Err: std::fmt::Display,
{
    s.parse().unwrap_or_else(|e| panic!("invalid {name}: {e}"))
}

#[inline]
fn parse_bool(s: String, name: &'static str) -> bool {
    match s.to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => true,
        "0" | "false" | "no" | "off" => false,
        other => panic!("invalid {name}: expected boolean, got {other:?}"),
    }
}

/// Full snapshot if you need a bunch of fields at once.
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub env: String,
    pub project_name: String,
    pub log_level: String,
    pub log_file: String,
    pub log_to_stdout: bool,
    pub database_path: String,
    pub storage_root: String,
    pub host: String,
    pub port: u16,
    pub code_manager_host: String,
    pub code_manager_port: u16,
    pub max_number_containers: usize,
    pub jwt_secret: String,
    pub jwt_duration_minutes: u64,
    pub reset_token_expiry_minutes: u64,
    pub max_password_reset_requests_per_hour: u32,
    pub gmail_username: String,
    pub gmail_app_password: String,
    pub frontend_url: String,
    pub email_from_name: String,
    pub gemini_api_key: String,
    pub moss_user_id: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        ensure_dotenv();
        Self {
            env: env(),
            project_name: project_name(),
            log_level: log_level(),
            log_file: log_file(),
            log_to_stdout: log_to_stdout(),
            database_path: database_path(),
            storage_root: storage_root(),
            host: host(),
            port: port(),
            code_manager_host: code_manager_host(),
            code_manager_port: code_manager_port(),
            max_number_containers: max_number_containers(),
            jwt_secret: jwt_secret(),
            jwt_duration_minutes: jwt_duration_minutes(),
            reset_token_expiry_minutes: reset_token_expiry_minutes(),
            max_password_reset_requests_per_hour: max_password_reset_requests_per_hour(),
            gmail_username: gmail_username(),
            gmail_app_password: gmail_app_password(),
            frontend_url: frontend_url(),
            email_from_name: email_from_name(),
            gemini_api_key: gemini_api_key(),
            moss_user_id: moss_user_id(),
        }
    }
}

// ----- Top-level getters under `config::` -----
// Each getter loads only the specific variable (plus a once-only .env load).
// All of these REQUIRE the env var to be set.

pub fn env() -> String {
    ensure_dotenv();
    require("APP_ENV")
}
pub fn project_name() -> String {
    ensure_dotenv();
    require("PROJECT_NAME")
}
pub fn log_level() -> String {
    ensure_dotenv();
    require("LOG_LEVEL")
}
pub fn log_file() -> String {
    ensure_dotenv();
    require("LOG_FILE")
}
pub fn log_to_stdout() -> bool {
    ensure_dotenv();
    parse_bool(require("LOG_TO_STDOUT"), "LOG_TO_STDOUT")
}

pub fn database_path() -> String {
    ensure_dotenv();
    require("DATABASE_PATH")
}
pub fn storage_root() -> String {
    ensure_dotenv();
    require("STORAGE_ROOT")
}

pub fn host() -> String {
    ensure_dotenv();
    require("HOST")
}
pub fn port() -> u16 {
    ensure_dotenv();
    parse(require("PORT"), "PORT")
}
pub fn code_manager_host() -> String {
    ensure_dotenv();
    require("CODE_MANAGER_HOST")
}
pub fn code_manager_port() -> u16 {
    ensure_dotenv();
    parse(require("CODE_MANAGER_PORT"), "CODE_MANAGER_PORT")
}

pub fn max_number_containers() -> usize {
    ensure_dotenv();
    parse(require("MAX_NUM_CONTAINERS"), "MAX_NUM_CONTAINERS")
}

pub fn jwt_secret() -> String {
    ensure_dotenv();
    require("JWT_SECRET")
}
pub fn jwt_duration_minutes() -> u64 {
    ensure_dotenv();
    parse(require("JWT_DURATION_MINUTES"), "JWT_DURATION_MINUTES")
}
pub fn reset_token_expiry_minutes() -> u64 {
    ensure_dotenv();
    parse(
        require("RESET_TOKEN_EXPIRY_MINUTES"),
        "RESET_TOKEN_EXPIRY_MINUTES",
    )
}
pub fn max_password_reset_requests_per_hour() -> u32 {
    ensure_dotenv();
    parse(
        require("MAX_PASSWORD_RESET_REQUESTS_PER_HOUR"),
        "MAX_PASSWORD_RESET_REQUESTS_PER_HOUR",
    )
}

pub fn gmail_username() -> String {
    ensure_dotenv();
    require("GMAIL_USERNAME")
}
pub fn gmail_app_password() -> String {
    ensure_dotenv();
    require("GMAIL_APP_PASSWORD")
}
pub fn frontend_url() -> String {
    ensure_dotenv();
    require("FRONTEND_URL")
}
pub fn email_from_name() -> String {
    ensure_dotenv();
    require("EMAIL_FROM_NAME")
}
pub fn gemini_api_key() -> String {
    ensure_dotenv();
    require("GEMINI_API_KEY")
}
pub fn moss_user_id() -> String {
    ensure_dotenv();
    require("MOSS_USER_ID")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::panic;

    // Keep this in sync with the getters / from_env.
    const ALL_VARS: &[&str] = &[
        "APP_ENV",
        "PROJECT_NAME",
        "LOG_LEVEL",
        "LOG_FILE",
        "LOG_TO_STDOUT",
        "DATABASE_PATH",
        "STORAGE_ROOT",
        "HOST",
        "PORT",
        "CODE_MANAGER_HOST",
        "CODE_MANAGER_PORT",
        "MAX_NUM_CONTAINERS",
        "JWT_SECRET",
        "JWT_DURATION_MINUTES",
        "RESET_TOKEN_EXPIRY_MINUTES",
        "MAX_PASSWORD_RESET_REQUESTS_PER_HOUR",
        "GMAIL_USERNAME",
        "GMAIL_APP_PASSWORD",
        "FRONTEND_URL",
        "EMAIL_FROM_NAME",
        "GEMINI_API_KEY",
        "MOSS_USER_ID",
    ];

    fn clear_all_env() {
        for k in ALL_VARS {
            // On Unix multi-threaded programs, set/remove are unsafe.
            unsafe { std::env::remove_var(k) };
        }
    }

    fn set_all_env_sample() {
        unsafe {
            std::env::set_var("APP_ENV", "test");
            std::env::set_var("PROJECT_NAME", "proj");
            std::env::set_var("LOG_LEVEL", "debug");
            std::env::set_var("LOG_FILE", "server.log");
            std::env::set_var("LOG_TO_STDOUT", "true");

            std::env::set_var("DATABASE_PATH", "/tmp/app.db");
            std::env::set_var("STORAGE_ROOT", "/tmp/storage");

            std::env::set_var("HOST", "0.0.0.0");
            std::env::set_var("PORT", "8080");

            std::env::set_var("CODE_MANAGER_HOST", "127.0.0.1");
            std::env::set_var("CODE_MANAGER_PORT", "5050");

            std::env::set_var("MAX_NUM_CONTAINERS", "42");

            std::env::set_var("JWT_SECRET", "sekret");
            std::env::set_var("JWT_DURATION_MINUTES", "120");
            std::env::set_var("RESET_TOKEN_EXPIRY_MINUTES", "30");
            std::env::set_var("MAX_PASSWORD_RESET_REQUESTS_PER_HOUR", "7");

            std::env::set_var("GMAIL_USERNAME", "user@example.com");
            std::env::set_var("GMAIL_APP_PASSWORD", "app-pass");
            std::env::set_var("FRONTEND_URL", "https://frontend.local");
            std::env::set_var("EMAIL_FROM_NAME", "FitchFork");
            std::env::set_var("GEMINI_API_KEY", "g-abc");
            std::env::set_var("MOSS_USER_ID", "123");
        }
    }

    #[test]
    #[serial]
    fn getters_individual_ok() {
        clear_all_env();

        unsafe {
            std::env::set_var("APP_ENV", "foo");
        }
        assert_eq!(super::env(), "foo");

        unsafe {
            std::env::set_var("PROJECT_NAME", "p");
        }
        assert_eq!(super::project_name(), "p");

        unsafe {
            std::env::set_var("LOG_LEVEL", "api=info");
        }
        assert_eq!(super::log_level(), "api=info");

        unsafe {
            std::env::set_var("LOG_FILE", "api.log");
        }
        assert_eq!(super::log_file(), "api.log");

        unsafe {
            std::env::set_var("LOG_TO_STDOUT", "yes");
        }
        assert_eq!(super::log_to_stdout(), true);

        unsafe {
            std::env::set_var("HOST", "127.0.0.1");
        }
        assert_eq!(super::host(), "127.0.0.1");

        unsafe {
            std::env::set_var("PORT", "3001");
        }
        assert_eq!(super::port(), 3001);

        unsafe {
            std::env::set_var("CODE_MANAGER_HOST", "127.0.0.2");
        }
        assert_eq!(super::code_manager_host(), "127.0.0.2");

        unsafe {
            std::env::set_var("CODE_MANAGER_PORT", "5001");
        }
        assert_eq!(super::code_manager_port(), 5001);

        unsafe {
            std::env::set_var("MAX_NUM_CONTAINERS", "5");
        }
        assert_eq!(super::max_number_containers(), 5);

        unsafe {
            std::env::set_var("JWT_SECRET", "abc");
        }
        assert_eq!(super::jwt_secret(), "abc");

        unsafe {
            std::env::set_var("JWT_DURATION_MINUTES", "10");
        }
        assert_eq!(super::jwt_duration_minutes(), 10);

        unsafe {
            std::env::set_var("RESET_TOKEN_EXPIRY_MINUTES", "99");
        }
        assert_eq!(super::reset_token_expiry_minutes(), 99);

        unsafe {
            std::env::set_var("MAX_PASSWORD_RESET_REQUESTS_PER_HOUR", "2");
        }
        assert_eq!(super::max_password_reset_requests_per_hour(), 2);

        unsafe {
            std::env::set_var("DATABASE_PATH", "/tmp/x.db");
        }
        assert_eq!(super::database_path(), "/tmp/x.db");

        unsafe {
            std::env::set_var("STORAGE_ROOT", "/tmp/s");
        }
        assert_eq!(super::storage_root(), "/tmp/s");

        unsafe {
            std::env::set_var("GMAIL_USERNAME", "u");
        }
        assert_eq!(super::gmail_username(), "u");

        unsafe {
            std::env::set_var("GMAIL_APP_PASSWORD", "pw");
        }
        assert_eq!(super::gmail_app_password(), "pw");

        unsafe {
            std::env::set_var("FRONTEND_URL", "https://x");
        }
        assert_eq!(super::frontend_url(), "https://x");

        unsafe {
            std::env::set_var("EMAIL_FROM_NAME", "X");
        }
        assert_eq!(super::email_from_name(), "X");

        unsafe {
            std::env::set_var("GEMINI_API_KEY", "g");
        }
        assert_eq!(super::gemini_api_key(), "g");

        unsafe {
            std::env::set_var("MOSS_USER_ID", "mid");
        }
        assert_eq!(super::moss_user_id(), "mid");
    }

    #[test]
    #[serial]
    fn log_to_stdout_boolean_parsing() {
        clear_all_env();

        unsafe {
            std::env::set_var("LOG_TO_STDOUT", "on");
        }
        assert!(super::log_to_stdout());

        unsafe {
            std::env::set_var("LOG_TO_STDOUT", "off");
        }
        assert!(!super::log_to_stdout());

        unsafe {
            std::env::set_var("LOG_TO_STDOUT", "TRUE");
        }
        assert!(super::log_to_stdout());

        unsafe {
            std::env::set_var("LOG_TO_STDOUT", "0");
        }
        assert!(!super::log_to_stdout());
    }

    #[test]
    #[serial]
    fn invalid_boolean_panics() {
        clear_all_env();
        unsafe {
            std::env::set_var("LOG_TO_STDOUT", "maybe");
        }
        let res = panic::catch_unwind(|| {
            let _ = super::log_to_stdout();
        });
        assert!(res.is_err());
    }

    #[test]
    #[serial]
    fn invalid_numeric_panics() {
        clear_all_env();

        unsafe {
            std::env::set_var("PORT", "not-a-number");
        }
        let res = panic::catch_unwind(|| {
            let _ = super::port();
        });
        assert!(res.is_err());

        unsafe {
            std::env::set_var("CODE_MANAGER_PORT", "NaN");
        }
        let res = panic::catch_unwind(|| {
            let _ = super::code_manager_port();
        });
        assert!(res.is_err());

        unsafe {
            std::env::set_var("MAX_NUM_CONTAINERS", "forty-two");
        }
        let res = panic::catch_unwind(|| {
            let _ = super::max_number_containers();
        });
        assert!(res.is_err());
    }

    #[test]
    #[serial]
    fn missing_required_panics() {
        clear_all_env();

        // Each getter should panic if its var is missing.
        let res = panic::catch_unwind(|| {
            let _ = super::jwt_secret();
        });
        assert!(res.is_err());

        let res = panic::catch_unwind(|| {
            let _ = super::database_path();
        });
        assert!(res.is_err());

        let res = panic::catch_unwind(|| {
            let _ = super::storage_root();
        });
        assert!(res.is_err());
    }

    #[test]
    #[serial]
    fn full_snapshot_reads_all() {
        clear_all_env();
        set_all_env_sample();

        let cfg = AppConfig::from_env();

        assert_eq!(cfg.env, "test");
        assert_eq!(cfg.project_name, "proj");
        assert_eq!(cfg.log_level, "debug");
        assert_eq!(cfg.log_file, "server.log");
        assert_eq!(cfg.log_to_stdout, true);

        assert_eq!(cfg.database_path, "/tmp/app.db");
        assert_eq!(cfg.storage_root, "/tmp/storage");

        assert_eq!(cfg.host, "0.0.0.0");
        assert_eq!(cfg.port, 8080);

        assert_eq!(cfg.code_manager_host, "127.0.0.1");
        assert_eq!(cfg.code_manager_port, 5050);

        assert_eq!(cfg.max_number_containers, 42);

        assert_eq!(cfg.jwt_secret, "sekret");
        assert_eq!(cfg.jwt_duration_minutes, 120);
        assert_eq!(cfg.reset_token_expiry_minutes, 30);
        assert_eq!(cfg.max_password_reset_requests_per_hour, 7);

        assert_eq!(cfg.gmail_username, "user@example.com");
        assert_eq!(cfg.gmail_app_password, "app-pass");
        assert_eq!(cfg.frontend_url, "https://frontend.local");
        assert_eq!(cfg.email_from_name, "FitchFork");
        assert_eq!(cfg.gemini_api_key, "g-abc");
        assert_eq!(cfg.moss_user_id, "123");
    }

    #[test]
    #[serial]
    fn full_snapshot_missing_var_panics() {
        clear_all_env();
        // Intentionally set almost everything except one to ensure from_env() panics.
        unsafe {
            std::env::set_var("APP_ENV", "test");
            std::env::set_var("PROJECT_NAME", "proj");
            std::env::set_var("LOG_LEVEL", "debug");
            std::env::set_var("LOG_FILE", "server.log");
            std::env::set_var("LOG_TO_STDOUT", "true");

            std::env::set_var("DATABASE_PATH", "/tmp/app.db");
            std::env::set_var("STORAGE_ROOT", "/tmp/storage");

            std::env::set_var("HOST", "0.0.0.0");
            std::env::set_var("PORT", "8080");

            std::env::set_var("CODE_MANAGER_HOST", "127.0.0.1");
            std::env::set_var("CODE_MANAGER_PORT", "5050");

            std::env::set_var("MAX_NUM_CONTAINERS", "42");

            std::env::set_var("JWT_SECRET", "sekret");
            std::env::set_var("JWT_DURATION_MINUTES", "120");
            std::env::set_var("RESET_TOKEN_EXPIRY_MINUTES", "30");
            std::env::set_var("MAX_PASSWORD_RESET_REQUESTS_PER_HOUR", "7");

            std::env::set_var("GMAIL_USERNAME", "user@example.com");
            std::env::set_var("GMAIL_APP_PASSWORD", "app-pass");
            std::env::set_var("FRONTEND_URL", "https://frontend.local");
            std::env::set_var("EMAIL_FROM_NAME", "FitchFork");
            std::env::set_var("GEMINI_API_KEY", "g-abc");
            // MOSS_USER_ID intentionally missing
        }

        let res = panic::catch_unwind(|| {
            let _ = AppConfig::from_env();
        });
        assert!(
            res.is_err(),
            "from_env() should panic when any var is missing"
        );
    }
}
