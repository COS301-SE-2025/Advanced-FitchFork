use std::{fs::File, io::Cursor, path::PathBuf, process::Stdio};
use tempfile::tempdir;
use tokio::{
    process::Command,
    time::{Duration, timeout},
};
use zip::ZipArchive;

pub mod execution_config;
use crate::execution_config::ExecutionConfig;
pub mod validate_files;
use crate::validate_files::validate_memo_files;

/// Contains language-specific settings for Docker and execution.
pub struct LanguageConfig {
    pub name: &'static str,
    pub file_extensions: &'static [&'static str],
    pub docker_image: &'static str,
    pub run_command: &'static str,
}

/// Map string language to LanguageConfig.
fn get_language_config(lang: &str) -> Option<LanguageConfig> {
    match lang {
        "java" => Some(LanguageConfig {
            name: "Java",
            file_extensions: &[".java"],
            docker_image: "openjdk:17-slim",
            run_command: "javac -d /output /code/*.java && java -cp /output Main",
        }),
        "cpp" => Some(LanguageConfig {
            name: "C++",
            file_extensions: &[".cpp", ".h"],
            docker_image: "gcc:13",
            run_command: "find /code -name '*.cpp' -exec g++ -o /output/app {} + && /output/app",
        }),
        "python" => Some(LanguageConfig {
            name: "Python",
            file_extensions: &[".py"],
            docker_image: "python:3.11-slim",
            run_command: "python3 /code/main.py",
        }),
        _ => None,
    }
}

pub async fn run_zip_files(
    module_id: i64,
    assignment_id: i64,
    zip_paths: Vec<PathBuf>,
) -> Result<String, String> {
    validate_memo_files(module_id, assignment_id)
        .map_err(|e| format!("Config validation failed: {}", e))?;

    let config = ExecutionConfig::get_execution_config(assignment_id)
        .map_err(|e| format!("Failed to load execution config: {}", e))?;

    let lang_cfg = get_language_config(&config.language)
        .ok_or_else(|| format!("Unsupported language: {}", config.language))?;

    run_all_zips(zip_paths, &lang_cfg, &config)
        .await
        .map_err(|e| format!("Execution error: {}", e))
}

async fn run_all_zips(
    zip_paths: Vec<PathBuf>,
    lang_cfg: &LanguageConfig,
    config: &ExecutionConfig,
) -> Result<String, Box<dyn std::error::Error>> {
    let temp_code_dir = tempdir()?;
    let temp_output_dir = tempdir()?;

    let code_path = temp_code_dir.path();
    let output_path = temp_output_dir.path();

    for zip_path in zip_paths {
        let zip_bytes = std::fs::read(&zip_path)?;
        extract_zip_contents(
            &zip_bytes,
            lang_cfg,
            config.max_uncompressed_size,
            code_path,
        )?;
    }

    let output = Command::new("docker")
        .arg("run")
        .arg("--rm")
        .arg("--network=none")
        .arg(format!("--memory={}", config.max_memory))
        .arg(format!("--cpus={}", config.max_cpus))
        .arg(format!("--pids-limit={}", config.max_processes))
        .arg("--security-opt=no-new-privileges")
        .arg("--user=1000:1000")
        .arg("-v")
        .arg(format!("{}:/code:ro", code_path.display()))
        .arg("-v")
        .arg(format!("{}:/output", output_path.display()))
        .arg(&lang_cfg.docker_image)
        .arg("sh")
        .arg("-c")
        .arg(&lang_cfg.run_command)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let output = timeout(
        Duration::from_secs(config.timeout_secs),
        output.wait_with_output(),
    )
    .await??;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        let err = String::from_utf8_lossy(&output.stderr);
        Err(format!("Execution failed:\n{}", err).into())
    }
}

fn extract_zip_contents(
    zip_bytes: &[u8],
    lang_cfg: &LanguageConfig,
    max_total_uncompressed: u64,
    output_dir: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut archive = ZipArchive::new(Cursor::new(zip_bytes))?;
    let mut total_uncompressed = 0;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        total_uncompressed += file.size();

        if total_uncompressed > max_total_uncompressed {
            return Err("Zip file too large when decompressed".into());
        }

        let raw_name = file.name();
        if raw_name.contains("..") || raw_name.starts_with('/') || raw_name.contains('\\') {
            return Err(format!("Invalid file path in zip: {}", raw_name).into());
        }

        let is_valid = lang_cfg
            .file_extensions
            .iter()
            .any(|ext| raw_name.ends_with(ext) || raw_name.ends_with('/'));

        if !is_valid {
            return Err(format!("Unsupported file type in zip: {}", raw_name).into());
        }

        let outpath = output_dir.join(raw_name);
        if file.name().ends_with('/') {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut outfile = File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }

    Ok(())
}

//TODO - Fix github actions to be able to run docker containers with all the languages
// The problem with these tests is that they fail with github actions
// That is why they are ignored

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::path::PathBuf;

//     fn test_zip_path(name: &str) -> PathBuf {
//         PathBuf::from(format!("src/test_files/{}", name))
//     }

//     fn test_config() -> ExecutionConfig {
//         ExecutionConfig {
//             timeout_secs: 30,
//             max_memory: "128m",
//             max_cpus: "1",
//             max_processes: 64,
//             max_uncompressed_size: 50 * 1024 * 1024,
//         }
//     }

//     fn get_test_language_config(lang: &str) -> LanguageConfig {
//         get_language_config(lang).expect("Language config not found")
//     }

//     #[tokio::test]
//     #[ignore]
//     async fn test_good_java_example_succeeds() {
//         let lang_cfg = get_test_language_config("java");
//         let result = run_all_zips(
//             vec![test_zip_path("good_java_example.zip")],
//             &lang_cfg,
//             &test_config(),
//         )
//         .await;

//         let output = result.expect("Expected success, got error");
//         assert!(!output.trim().is_empty(), "Expected non-empty output");
//     }

//     #[tokio::test]
//     #[ignore]
//     async fn test_good_cpp_example_succeeds() {
//         let lang_cfg = get_test_language_config("cpp");
//         let result = run_all_zips(
//             vec![test_zip_path("good_cpp_example.zip")],
//             &lang_cfg,
//             &test_config(),
//         )
//         .await;

//         assert!(result.is_ok(), "Expected successful C++ run");
//     }

//     #[tokio::test]
//     #[ignore]
//     async fn test_good_python_example_succeeds() {
//         let lang_cfg = get_test_language_config("python");
//         let result = run_all_zips(
//             vec![test_zip_path("good_python_example.zip")],
//             &lang_cfg,
//             &test_config(),
//         )
//         .await;

//         assert!(result.is_ok(), "Expected successful Python run");
//     }

//     #[tokio::test]
//     #[ignore]
//     async fn test_infinite_loop_fails() {
//         let lang_cfg = get_test_language_config("java");
//         let result = run_all_zips(
//             vec![test_zip_path("infinite_loop_java_example.zip")],
//             &lang_cfg,
//             &test_config(),
//         )
//         .await;

//         assert!(result.is_err(), "Infinite loop should timeout or fail");
//     }

//     #[tokio::test]
//     #[ignore]
//     async fn test_memory_overflow_fails() {
//         let lang_cfg = get_test_language_config("java");
//         let result = run_all_zips(
//             vec![test_zip_path("memory_overflow_java_example.zip")],
//             &lang_cfg,
//             &test_config(),
//         )
//         .await;

//         assert!(result.is_err(), "Memory overflow should fail");
//     }

//     #[tokio::test]
//     #[ignore]
//     async fn test_fork_bomb_fails() {
//         let lang_cfg = get_test_language_config("java");
//         let result = run_all_zips(
//             vec![test_zip_path("fork_bomb_java_example.zip")],
//             &lang_cfg,
//             &test_config(),
//         )
//         .await;

//         assert!(result.is_err(), "Fork bomb should fail");
//     }

//     #[tokio::test]
//     #[ignore]
//     async fn test_edit_code_fails() {
//         let lang_cfg = get_test_language_config("java");
//         let result = run_all_zips(
//             vec![test_zip_path("edit_code_java_example.zip")],
//             &lang_cfg,
//             &test_config(),
//         )
//         .await;

//         match result {
//             Ok(output) => {
//                 assert!(
//                     output.contains("Read-only file system"),
//                     "Expected sandbox to block file edits, but got:\n{}",
//                     output
//                 );
//             }
//             Err(err) => {
//                 let msg = err.to_string();
//                 assert!(
//                     msg.contains("Read-only file system") || msg.contains("Permission denied"),
//                     "Expected read-only FS error, but got: {}",
//                     msg
//                 );
//             }
//         }
//     }

//     #[tokio::test]
//     #[ignore]
//     async fn test_privilege_escalation_fails() {
//         let lang_cfg = get_test_language_config("java");
//         let result = run_all_zips(
//             vec![test_zip_path("priviledge_escalation_java_example.zip")],
//             &lang_cfg,
//             &test_config(),
//         )
//         .await;

//         match result {
//             Ok(output) => {
//                 assert!(
//                     output.contains("uid=1000"),
//                     "Should not run with root privileges"
//                 );
//             }
//             Err(_) => {
//                 // Error is also acceptable
//             }
//         }
//     }

//     #[tokio::test]
//     #[ignore]
//     async fn test_network_access_fails() {
//         let lang_cfg = get_test_language_config("java");
//         let result = run_all_zips(
//             vec![test_zip_path("access_network_java_example.zip")],
//             &lang_cfg,
//             &test_config(),
//         )
//         .await;

//         assert!(
//             result.is_err()
//                 || result
//                     .as_ref()
//                     .map(|s| s.contains("Network access blocked"))
//                     .unwrap_or(false),
//             "Network access should not be allowed"
//         );
//     }
// }
