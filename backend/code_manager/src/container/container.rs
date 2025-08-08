use std::fs;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use tempfile::tempdir;
use tokio::process::Command;
use tokio::time::timeout;
use util::execution_config::ExecutionConfig;

use crate::container::compression::{extract_archive_contents, is_supported_archive};

pub async fn run_container(
    config: &ExecutionConfig,
    commands: Vec<String>,
    files: Vec<PathBuf>,
) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let temp_code_dir = tempdir()?;
    let temp_output_dir = tempdir()?;

    let code_path = temp_code_dir.path().to_path_buf();
    let output_path = temp_output_dir.path().to_path_buf();

    for file_path in files {
        if is_supported_archive(&file_path) {
            let archive_bytes = std::fs::read(&file_path)?;
            extract_archive_contents(
                &file_path,
                &archive_bytes,
                config.execution.max_uncompressed_size,
                &code_path,
            )?;
        } else {
            let file_name = file_path.file_name().ok_or("Invalid file path")?;
            let destination = code_path.join(file_name);
            fs::copy(&file_path, destination)?;
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
            .arg(memory_arg.clone())
            .arg(cpus_arg.clone())
            .arg(pids_arg.clone())
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

        let output = timeout(
            Duration::from_secs(config.execution.timeout_secs),
            docker_output.wait_with_output(),
        )
        .await??;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if output.status.success() {
            outputs.push(stdout.into_owned());
        } else {
            return Err(format!(
                "Execution failed (exit code {}):\nSTDOUT:\n{}\nSTDERR:\n{}",
                output.status.code().unwrap_or(-1),
                stdout,
                stderr
            )
            .into());
        }
    }

    Ok(outputs)
}
