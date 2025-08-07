use std::fs;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use tempfile::tempdir;
use tokio::process::Command;
use tokio::time::timeout;

use util::execution_config::ExecutionConfig;

/// Runs a Docker container with the provided command and input files.
/// If any of the files are zip archives, they are extracted into the execution directory.
pub async fn run_container(
    config: &ExecutionConfig,
    command: String,
    files: Vec<PathBuf>,
) -> Result<String, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let temp_code_dir = tempdir()?;
    let temp_output_dir = tempdir()?;

    let code_path = temp_code_dir.path().to_path_buf();
    let output_path = temp_output_dir.path().to_path_buf();

    for file_path in files {
        if is_zip_file(&file_path) {
            let archive_bytes = std::fs::read(&file_path)?;
            extract_archive_contents(
                &archive_bytes,
                config.execution.max_uncompressed_size,
                &code_path,
            )?;
        } else {
            // Copy regular file into code directory
            let file_name = file_path.file_name().ok_or("Invalid file path")?;
            let destination = code_path.join(file_name);
            fs::copy(&file_path, destination)?;
        }
    }

    let memory_arg = format!("--memory={}b", config.execution.max_memory);
    let cpus_arg = format!("--cpus={}", config.execution.max_cpus);
    let pids_arg = format!("--pids-limit={}", config.execution.max_processes);

    let docker_output = Command::new("docker")
        .arg("run")
        .arg("--rm")
        .arg("--network=none")
        .arg(memory_arg)
        .arg(cpus_arg)
        .arg(pids_arg)
        .arg("--security-opt=no-new-privileges")
        .arg("-v")
        .arg(format!("{}:/code:rw", code_path.display()))
        .arg("-v")
        .arg(format!("{}:/output", output_path.display()))
        .arg("universal-runner")
        .arg("sh")
        .arg("-c")
        .arg(&command)
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
        Ok(stdout.into_owned())
    } else {
        Err(format!(
            "Execution failed (exit code {}):\nSTDOUT:\n{}\nSTDERR:\n{}",
            output.status.code().unwrap_or(-1),
            stdout,
            stderr
        )
        .into())
    }
}

///Helper function to detect if a file is zipped
fn is_zip_file(path: &Path) -> bool {
    path.extension().map(|ext| ext == "zip").unwrap_or(false)
}

///Helper function to extract zipped file
fn extract_archive_contents(
    archive_bytes: &[u8],
    max_uncompressed_size: u64,
    destination_dir: &Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    use std::io::Cursor;
    use zip::read::ZipArchive;

    let reader = Cursor::new(archive_bytes);
    let mut archive = ZipArchive::new(reader)?;

    let mut total_uncompressed_size = 0;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = destination_dir.join(file.name());

        if !outpath.starts_with(destination_dir) {
            return Err("Zip archive contains invalid path (zip slip attack?)".into());
        }

        total_uncompressed_size += file.size();
        if total_uncompressed_size > max_uncompressed_size {
            return Err("Uncompressed zip size exceeds allowed maximum".into());
        }

        if file.name().ends_with('/') {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                std::fs::create_dir_all(p)?;
            }

            let mut outfile = std::fs::File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }

    Ok(())
}
