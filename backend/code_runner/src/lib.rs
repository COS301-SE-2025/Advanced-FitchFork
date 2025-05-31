use std::{fs::File, io::Cursor, path::PathBuf, process::Stdio};
use tempfile::tempdir;
use tokio::{
    process::Command,
    time::{Duration, timeout},
};
use zip::ZipArchive;

/// Configuration for runtime environment limits (used for Docker container).
pub struct ExecutionConfig {
    pub timeout_secs: u64,          // Max execution time
    pub max_memory: &'static str,   // Max memory (e.g., "128m")
    pub max_cpus: &'static str,     // CPU cores (e.g., "1.0")
    pub max_uncompressed_size: u64, // Max total size of decompressed files (zip bomb protection)
    pub max_processes: u32,         // Max number of processes inside container
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        ExecutionConfig {
            timeout_secs: 20,                        // 20 seconds max runtime
            max_memory: "128m",                      // 128 MB memory limit
            max_cpus: "1.0",                         // 1 CPU core
            max_uncompressed_size: 50 * 1024 * 1024, // 50 MB max decompressed size
            max_processes: 64,                       // Max 64 processes inside container
        }
    }
}

/// Contains language-specific settings for Docker and execution.
pub struct LanguageConfig {
    pub name: &'static str,                       // Human-readable name
    pub file_extensions: &'static [&'static str], // Allowed file extensions
    pub docker_image: &'static str,               // Docker image to use
    pub run_command: &'static str,                // Command to compile and/or run code
}

/// Map string language to LanguageConfig.
/// Add additional languages as needed.
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

/// Main entry point: run multiple student zip files as a single combined codebase.
/// Assumes each zip contains part of a valid program (e.g., Java modules).
///
/// # Arguments
/// * `zip_paths` - Vector of zip file paths to combine and execute.
/// * `lang` - Programming language identifier string.
/// * `config` - Execution configuration with runtime restrictions and security.
///
/// # Returns
/// * `Ok(String)` with the combined program output if execution succeeds.
/// * `Err(String)` with an error message if execution fails or language is unsupported.
pub async fn run_zip_files(
    zip_paths: Vec<PathBuf>,
    lang: &str,
    config: Option<ExecutionConfig>,
) -> Result<String, String> {
    let config = config.unwrap_or_default();
    match get_language_config(lang) {
        Some(lang_cfg) => match run_all_zips(zip_paths, &lang_cfg, &config).await {
            Ok(output) => Ok(output),
            Err(e) => Err(format!("Execution error: {}", e)),
        },
        None => Err(format!("Unsupported language: {}", lang)),
    }
}

/// Extracts contents of all zip files, verifies them, then runs them in a Docker container.
/// Ensures strict resource limits and security.
///
/// # Returns
/// Standard output of the program if successful; otherwise an error.
async fn run_all_zips(
    zip_paths: Vec<PathBuf>,
    lang_cfg: &LanguageConfig,
    config: &ExecutionConfig,
) -> Result<String, Box<dyn std::error::Error>> {
    // Create temporary directories for input code and compiled/built output
    let temp_code_dir = tempdir()?;
    let temp_output_dir = tempdir()?;

    let code_path = temp_code_dir.path();
    let output_path = temp_output_dir.path();

    // Process each zip file: validate, decompress, filter only allowed extensions
    for zip_path in zip_paths {
        let zip_bytes = std::fs::read(&zip_path)?;
        extract_zip_contents(
            &zip_bytes,
            lang_cfg,
            config.max_uncompressed_size,
            code_path,
        )?;
    }

    // Launch Docker container with strict runtime limits and no network access
    let output = Command::new("docker")
        .arg("run")
        .arg("--rm") // Remove container after execution
        .arg("--network=none") // No network access (security)
        .arg(format!("--memory={}", config.max_memory)) // RAM limit
        .arg(format!("--cpus={}", config.max_cpus)) // CPU core limit
        .arg(format!("--pids-limit={}", config.max_processes)) // Process limit
        .arg("--security-opt=no-new-privileges") // Prevent privilege escalation
        .arg("--user=1000:1000") // Run as non-root user
        .arg("-v")
        .arg(format!("{}:/code:ro", code_path.display())) // Mount code directory read-only
        .arg("-v")
        .arg(format!("{}:/output", output_path.display())) // Mount output directory
        .arg(&lang_cfg.docker_image) // Base Docker image
        .arg("sh")
        .arg("-c")
        .arg(&lang_cfg.run_command) // Run/compile inside container
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    // Enforce timeout on Docker execution
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

/// Extracts a zip archive to the provided output directory.
/// Ensures all files are of valid types, checks for zip-slip, and enforces size limits.
///
/// # Security Measures
/// - Blocks dangerous paths (`..`, absolute paths, backslashes)
/// - Limits total decompressed size (defends against zip bombs)
/// - Rejects unexpected file extensions
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

        // Zip bomb protection: limit total extracted size
        if total_uncompressed > max_total_uncompressed {
            return Err("Zip file too large when decompressed".into());
        }

        let raw_name = file.name();

        // Zip slip protection: reject dangerous paths
        if raw_name.contains("..") || raw_name.starts_with('/') || raw_name.contains('\\') {
            return Err(format!("Invalid file path in zip: {}", raw_name).into());
        }

        // Validate file extension
        let is_valid = lang_cfg
            .file_extensions
            .iter()
            .any(|ext| raw_name.ends_with(ext) || raw_name.ends_with('/'));

        if !is_valid {
            return Err(format!("Unsupported file type in zip: {}", raw_name).into());
        }

        let outpath = output_dir.join(raw_name);

        // Ensure parent directories exist before writing file
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn test_zip_path(name: &str) -> PathBuf {
        PathBuf::from(format!("src/test_files/{}", name))
    }

    fn test_config() -> ExecutionConfig {
        ExecutionConfig {
            timeout_secs: 30,
            max_memory: "128m",
            max_cpus: "1",
            max_processes: 64,
            max_uncompressed_size: 50 * 1024 * 1024,
        }
    }

    fn get_test_language_config(lang: &str) -> LanguageConfig {
        get_language_config(lang).expect("Language config not found")
    }

    #[tokio::test]
    #[ignore]
    async fn test_good_java_example_succeeds() {
        let lang_cfg = get_test_language_config("java");
        let result = run_all_zips(
            vec![test_zip_path("good_java_example.zip")],
            &lang_cfg,
            &test_config(),
        )
        .await;

        let output = result.expect("Expected success, got error");
        assert!(!output.trim().is_empty(), "Expected non-empty output");
    }

    #[tokio::test]
    #[ignore]
    async fn test_good_cpp_example_succeeds() {
        let lang_cfg = get_test_language_config("cpp");
        let result = run_all_zips(
            vec![test_zip_path("good_cpp_example.zip")],
            &lang_cfg,
            &test_config(),
        )
        .await;

        assert!(result.is_ok(), "Expected successful C++ run");
    }

    #[tokio::test]
    #[ignore]
    async fn test_good_python_example_succeeds() {
        let lang_cfg = get_test_language_config("python");
        let result = run_all_zips(
            vec![test_zip_path("good_python_example.zip")],
            &lang_cfg,
            &test_config(),
        )
        .await;

        assert!(result.is_ok(), "Expected successful Python run");
    }

    #[tokio::test]
    #[ignore]
    async fn test_infinite_loop_fails() {
        let lang_cfg = get_test_language_config("java");
        let result = run_all_zips(
            vec![test_zip_path("infinite_loop_java_example.zip")],
            &lang_cfg,
            &test_config(),
        )
        .await;

        assert!(result.is_err(), "Infinite loop should timeout or fail");
    }

    #[tokio::test]
    #[ignore]
    async fn test_memory_overflow_fails() {
        let lang_cfg = get_test_language_config("java");
        let result = run_all_zips(
            vec![test_zip_path("memory_overflow_java_example.zip")],
            &lang_cfg,
            &test_config(),
        )
        .await;

        assert!(result.is_err(), "Memory overflow should fail");
    }

    #[tokio::test]
    #[ignore]
    async fn test_fork_bomb_fails() {
        let lang_cfg = get_test_language_config("java");
        let result = run_all_zips(
            vec![test_zip_path("fork_bomb_java_example.zip")],
            &lang_cfg,
            &test_config(),
        )
        .await;

        assert!(result.is_err(), "Fork bomb should fail");
    }

    #[tokio::test]
    #[ignore]
    async fn test_edit_code_fails() {
        let lang_cfg = get_test_language_config("java");
        let result = run_all_zips(
            vec![test_zip_path("edit_code_java_example.zip")],
            &lang_cfg,
            &test_config(),
        )
        .await;

        match result {
            Ok(output) => {
                assert!(
                    output.contains("Read-only file system"),
                    "Expected sandbox to block file edits, but got:\n{}",
                    output
                );
            }
            Err(err) => {
                let msg = err.to_string();
                assert!(
                    msg.contains("Read-only file system") || msg.contains("Permission denied"),
                    "Expected read-only FS error, but got: {}",
                    msg
                );
            }
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_privilege_escalation_fails() {
        let lang_cfg = get_test_language_config("java");
        let result = run_all_zips(
            vec![test_zip_path("priviledge_escalation_java_example.zip")],
            &lang_cfg,
            &test_config(),
        )
        .await;

        match result {
            Ok(output) => {
                assert!(
                    output.contains("uid=1000"),
                    "Should not run with root privileges"
                );
            }
            Err(_) => {
                // Error is also acceptable
            }
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_network_access_fails() {
        let lang_cfg = get_test_language_config("java");
        let result = run_all_zips(
            vec![test_zip_path("access_network_java_example.zip")],
            &lang_cfg,
            &test_config(),
        )
        .await;

        assert!(
            result.is_err()
                || result
                    .as_ref()
                    .map(|s| s.contains("Network access blocked"))
                    .unwrap_or(false),
            "Network access should not be allowed"
        );
    }
}
