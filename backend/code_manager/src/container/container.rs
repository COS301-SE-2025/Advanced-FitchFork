//container/container.rs
use std::fs;
use std::path::Path;
use std::process::Stdio;
use std::time::Duration;
// use tempfile::tempdir;
use tempdir::TempDir;
use tokio::process::Command;
use tokio::time::timeout;
use util::execution_config::ExecutionConfig;

use crate::utils::compression::{extract_archive_contents, is_supported_archive};

pub async fn run_container(
    config: &ExecutionConfig,
    commands: Vec<String>,
    files: Vec<(String, Vec<u8>)>,
    interpreter: bool,
) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let temp_code_dir = TempDir::new("code")?;
    let temp_output_dir = TempDir::new("output")?;

    let code_path = temp_code_dir.path().to_path_buf();
    let output_path = temp_output_dir.path().to_path_buf();

    for (file_name, contents) in files {
        let file_path = code_path.join(&file_name);

        if is_supported_archive(Path::new(&file_name)) {
            extract_archive_contents(
                Path::new(&file_name),
                &contents,
                config.execution.max_uncompressed_size,
                &code_path,
            )?;
        } else {
            fs::write(&file_path, &contents)?;
        }
    }

    let memory_arg = format!("--memory={}b", config.execution.max_memory);
    let cpus_arg = format!("--cpus={}", config.execution.max_cpus);
    let pids_arg = format!("--pids-limit={}", config.execution.max_processes);

    let mut outputs = Vec::new();

    for cmd in commands {
        let docker_output = Command::new("docker")
            .arg("run")
            .arg("--rm")
            .arg("--network=none")
            .arg(&memory_arg)
            .arg(&cpus_arg)
            .arg(&pids_arg)
            .arg("--security-opt=no-new-privileges")
            .arg("-v")
            .arg(format!("{}:/code:rw", code_path.display()))
            .arg("-v")
            .arg(format!("{}:/output", output_path.display()))
            .arg("universal-runner")
            .arg("sh")
            .arg("-c")
            .arg(&cmd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let output_result = timeout(
            Duration::from_secs(config.execution.timeout_secs),
            docker_output.wait_with_output(),
        )
        .await;

        let combined_output = match output_result {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
                let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
                let retcode = output.status.code().unwrap_or(-1);

                if interpreter {
                    // For interpreters: return raw stdout only
                    stdout
                } else {
                    // For normal execution: include markers
                    let mut combined = String::new();
                    combined.push_str(&stdout);
                    combined.push_str("&FITCHFORK&StandardError\n");
                    if !combined.is_empty() {
                        combined.push('\n');
                    }
                    combined.push_str(&stderr);
                    combined.push_str("&FITCHFORK&ReturnCode\n");
                    if !combined.is_empty() {
                        combined.push('\n');
                    }
                    combined.push_str(&format!("Retcode: {}", retcode));
                    combined
                }
            }
            Ok(Err(e)) => {
                if interpreter {
                    format!("Interpreter failed: {}", e)
                } else {
                    format!("&FITCHFORK&Error\nCommand failed: {}", e)
                }
            }
            Err(_) => {
                if interpreter {
                    "Interpreter timed out (possible infinite loop)".to_string()
                } else {
                    "&FITCHFORK&Error\nCommand timed out (possible infinite loop)".to_string()
                }
            }
        };

        outputs.push(combined_output);
    }

    Ok(outputs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    fn create_test_zip() -> Vec<u8> {
        use std::io::Write;
        use zip::write::FileOptions;
        let mut buffer = Vec::new();
        {
            let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buffer));
            let options = FileOptions::<()>::default();

            zip.start_file("hello.txt", options).unwrap();
            zip.write_all(b"Hello, Zip!").unwrap();
            zip.finish().unwrap();
        }
        buffer
    }

    #[tokio::test]
    async fn test_run_container_single_command_single_file() {
        let config = ExecutionConfig::default_config();

        let filename = "testfile.txt".to_string();
        let contents = b"echo Hello from file > output.txt".to_vec();

        let outputs = run_container(
            &config,
            vec![
                "sh /code/testfile.txt".to_string(),
                "cat /code/output.txt".to_string(),
            ],
            vec![(filename, contents)],
            false,
        )
        .await
        .expect("run_container failed");

        assert_eq!(outputs.len(), 2);
        assert!(outputs[1].contains("Hello from file"));
    }

    #[tokio::test]
    async fn test_run_container_multiple_commands() {
        let config = ExecutionConfig::default_config();

        let filename = "testfile.txt".to_string();
        let contents = b"echo line1 > output.txt".to_vec();

        let commands = vec![
            "sh /code/testfile.txt".to_string(),
            "echo line2 >> /code/output.txt".to_string(),
            "cat /code/output.txt".to_string(),
        ];

        let outputs = run_container(&config, commands, vec![(filename, contents)], false)
            .await
            .expect("run_container failed");

        assert_eq!(outputs.len(), 3);
        assert!(outputs[2].contains("line1"));
        assert!(outputs[2].contains("line2"));
    }

    #[tokio::test]
    async fn test_run_container_with_zip_file() {
        let config = ExecutionConfig::default_config();

        let zip_bytes = create_test_zip();

        let commands = vec!["cat hello.txt".to_string()];

        let outputs = run_container(
            &config,
            commands,
            vec![("test.zip".to_string(), zip_bytes)],
            false,
        )
        .await
        .expect("run_container failed");

        assert_eq!(outputs.len(), 1);
        assert!(outputs[0].contains("Hello, Zip!"));
    }
}
