use chrono::Local;
use fern::Dispatch;
use log::LevelFilter;
use std::fs::{OpenOptions, create_dir_all};
use std::path::Path;
use colored::Colorize; // <-- for color output

pub fn init_logger(log_level: &str, log_file_path: &str) {
    if let Some(parent) = Path::new(log_file_path).parent() {
        if !parent.exists() {
            create_dir_all(parent).expect("Failed to create log directory");
        }
    }

    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file_path)
        .expect("Cannot open log file");

    let level = match log_level.to_lowercase().as_str() {
        "trace" => LevelFilter::Trace,
        "debug" => LevelFilter::Debug,
        "info" => LevelFilter::Info,
        "warn" => LevelFilter::Warn,
        "error" => LevelFilter::Error,
        _ => LevelFilter::Info,
    };

    Dispatch::new()
        .format(|out, message, record| {
            let level_str = match record.level() {
                log::Level::Error => "ERROR".red(),
                log::Level::Warn  => "WARN".yellow(),
                log::Level::Info  => "INFO".green(),
                log::Level::Debug => "DEBUG".cyan(),
                log::Level::Trace => "TRACE".normal(),
            };

            out.finish(format_args!(
                "[{}][{}][{}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                level_str,
                record.target(),
                message
            ))
        })
        .level(level)
        .chain(std::io::stdout())
        .chain(log_file)
        .apply()
        .expect("Failed to initialize logger");
}
