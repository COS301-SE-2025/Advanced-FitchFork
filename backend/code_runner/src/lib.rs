// Core dependencies
use std::fs::File;
use std::io::Cursor;
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Stdio,
};

// Async, process, and timing
use tokio::process::Command;
use tokio::time::{Duration, timeout};

// External crates
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use tempfile::tempdir;
use zip::ZipArchive;

// Your own modules
use crate::validate_files::validate_memo_files;

// Models
use db::models::assignment::Entity as Assignment;
use db::models::assignment_memo_output::{Column as MemoOutputColumn, Entity as MemoOutputEntity};
use db::models::assignment_task::Model as AssignmentTask;
use reqwest::Client;
use serde_json::json;
use util::execution_config::ExecutionConfig;

pub mod validate_files;

/// Returns the first archive file (".zip", ".tar", ".tgz", ".gz") found in the given directory.
/// Returns an error if the directory does not exist or if no supported archive file is found.
fn first_archive_in(dir: &PathBuf) -> Result<PathBuf, String> {
    let allowed_exts = ["zip", "tar", "tgz", "gz"];
    std::fs::read_dir(dir)
        .map_err(|_| format!("Missing directory: {}", dir.display()))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .find(|p| {
            if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
                let ext = ext.to_ascii_lowercase();
                if allowed_exts.contains(&ext.as_str()) {
                    return true;
                }
            }
            false
        })
        .ok_or_else(|| {
            format!(
                "No .zip, .tar, .tgz, or .gz file found in {}",
                dir.display()
            )
        })
}

/// Resolves a potentially relative storage root path to an absolute path,
/// assuming the current working directory is the root of the project.
fn resolve_storage_root(storage_root: &str) -> PathBuf {
    let path = PathBuf::from(storage_root);
    if path.is_relative() {
        env::current_dir().unwrap().join(path)
    } else {
        path
    }
}

/// Runs all configured tasks for a given assignment ID by:
/// 1. Validating memo files
/// 2. Extracting archive files
/// 3. Running the configured commands inside Docker
/// 4. Saving the resulting output as memo files in the databaseencode
pub async fn create_memo_outputs_for_all_tasks(
    db: &DatabaseConnection,
    assignment_id: i64,
) -> Result<(), String> {
    // Fetch the assignment to get module_id
    let assignment = Assignment::find_by_id(assignment_id)
        .one(db)
        .await
        .map_err(|e| format!("Failed to fetch assignment: {}", e))?
        .ok_or_else(|| format!("Assignment {} not found", assignment_id))?;

    let module_id = assignment.module_id;

    // Validate required input files
    validate_memo_files(module_id, assignment_id)?;

    let config = ExecutionConfig::get_execution_config(module_id, assignment_id)
        .map_err(|e| format!("Failed to load execution config: {}", e))?;

    let storage_root = env::var("ASSIGNMENT_STORAGE_ROOT")
        .map_err(|_| "ASSIGNMENT_STORAGE_ROOT not set".to_string())?;

    let base_path = resolve_storage_root(&storage_root)
        .join(format!("module_{}", module_id))
        .join(format!("assignment_{}", assignment_id));

    let memo_output_dir = base_path.join("memo_output");

    // Delete old files from disk
    if memo_output_dir.exists() {
        fs::remove_dir_all(&memo_output_dir)
            .map_err(|e| format!("Failed to delete old memo_output dir: {}", e))?;
    }

    // Delete old entries from DB
    MemoOutputEntity::delete_many()
        .filter(MemoOutputColumn::AssignmentId.eq(assignment_id))
        .exec(db)
        .await
        .map_err(|e| format!("Failed to delete old memo outputs: {}", e))?;

    // Load archives
    let archive_paths = vec![
        first_archive_in(&base_path.join("memo"))?,
        first_archive_in(&base_path.join("makefile"))?,
        first_archive_in(&base_path.join("main"))?,
    ];

    let tasks = AssignmentTask::get_by_assignment_id(db, assignment_id)
        .await
        .map_err(|e| format!("DB error loading tasks: {}", e))?;

    if tasks.is_empty() {
        println!("No tasks found for assignment {}", assignment_id);
        return Ok(());
    }

    // Prepare HTTP client
    let client = Client::new();

    let host = env::var("CODE_MANAGER_HOST")
        .map_err(|_| "CODE_MANAGER_HOST env var not set".to_string())?;

    let port = env::var("CODE_MANAGER_PORT")
        .map_err(|_| "CODE_MANAGER_PORT env var not set".to_string())?;

    let code_manager_url = format!("http://{}:{}", host, port);

    for task in tasks {
        let filename = format!("task_{}_output.txt", task.task_number);

        // Prepare files to send: read archive contents into Vec<(String, Vec<u8>)>
        let mut files: Vec<(String, Vec<u8>)> = Vec::new();
        for archive_path in &archive_paths {
            let content = std::fs::read(archive_path)
                .map_err(|e| format!("Failed to read archive {:?}: {}", archive_path, e))?;
            let file_name = archive_path
                .file_name()
                .and_then(|s| s.to_str())
                .ok_or_else(|| "Invalid archive filename".to_string())?;
            files.push((file_name.to_string(), content));
        }

        // Wrap command in a vector (assuming task.command is a String)
        let commands = vec![task.command.clone()];

        // Serialize ExecutionConfig into a serde_json::Value
        let config_value = serde_json::to_value(&config)
            .map_err(|e| format!("Failed to serialize ExecutionConfig: {}", e))?;

        // Compose request JSON payload
        let request_body = serde_json::json!({
            "config": config_value,
            "commands": commands,
            "files": files, // Vec<(String, Vec<u8>)> will serialize correctly
        });

        // Note: You must adjust your API to accept files as Vec<(String, base64-string)> or send multipart form data.
        // Here I assume base64 encoding and API adjusted accordingly.

        // Send POST request to /run
        let response = client
            .post(format!("{}/run", code_manager_url))
            .json(&request_body)
            .send()
            .await
            .map_err(|e| format!("Failed to send request to code_manager: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!(
                "code_manager responded with error {}: {}",
                status, text
            ));
        }

        // Parse response JSON: expects { output: Vec<String> }
        let resp_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response JSON: {}", e))?;

        // Extract output vec from response
        let output_vec = resp_json
            .get("output")
            .and_then(|v| v.as_array())
            .ok_or_else(|| "Response missing 'output' array".to_string())?
            .iter()
            .map(|val| val.as_str().unwrap_or("").to_string())
            .collect::<Vec<String>>();

        // Join outputs or handle as needed
        let output_combined = output_vec.join("\n");

        if let Err(e) = db::models::assignment_memo_output::Model::save_file(
            db,
            assignment_id,
            task.id,
            &filename,
            output_combined.as_bytes(),
        )
        .await
        {
            println!("Failed to save output: {}", e);
        }
    }

    Ok(())
}

use db::models::assignment_submission_output::Model as SubmissionOutputModel;

/// Runs all configured tasks for a given assignment ID and student attempt by:
/// 1. Validating submission files
/// 2. Extracting archive files (submission, makefile, main)
/// 3. Running the configured commands inside Docker
/// 4. Saving the output to disk and database as `assignment_submission_output`
pub async fn create_submission_outputs_for_all_tasks(
    db: &DatabaseConnection,
    submission_id: i64,
) -> Result<(), String> {
    use crate::validate_files::validate_submission_files;
    use db::models::assignment::Entity as Assignment;
    use db::models::assignment_submission::Entity as AssignmentSubmission;
    use reqwest::Client;
    use sea_orm::EntityTrait;
    use serde_json::json;
    use std::env;
    use tokio::fs::read;

    // Fetch the submission
    let submission = AssignmentSubmission::find_by_id(submission_id)
        .one(db)
        .await
        .map_err(|e| format!("Failed to fetch submission: {}", e))?
        .ok_or_else(|| format!("Submission {} not found", submission_id))?;

    let assignment_id = submission.assignment_id;
    let user_id = submission.user_id;
    let attempt_number = submission.attempt;

    // Fetch the assignment to get module_id
    let assignment = Assignment::find_by_id(assignment_id)
        .one(db)
        .await
        .map_err(|e| format!("Failed to fetch assignment: {}", e))?
        .ok_or_else(|| format!("Assignment {} not found", assignment_id))?;

    let module_id = assignment.module_id;

    // Validate submission-related files
    validate_submission_files(module_id, assignment_id, user_id, attempt_number)?;

    let config = ExecutionConfig::get_execution_config(module_id, assignment_id)
        .map_err(|e| format!("Failed to load execution config: {}", e))?;

    let storage_root = env::var("ASSIGNMENT_STORAGE_ROOT")
        .map_err(|_| "ASSIGNMENT_STORAGE_ROOT not set".to_string())?;

    let base_path = resolve_storage_root(&storage_root)
        .join(format!("module_{}", module_id))
        .join(format!("assignment_{}", assignment_id));

    let submission_path = base_path
        .join("assignment_submissions")
        .join(format!("user_{}", user_id))
        .join(format!("attempt_{}", attempt_number));

    // Load archive files into memory
    let archive_paths = vec![
        first_archive_in(&submission_path)?,
        first_archive_in(&base_path.join("makefile"))?,
        first_archive_in(&base_path.join("main"))?,
    ];

    let mut files = Vec::new();
    for path in &archive_paths {
        let content = read(path)
            .await
            .map_err(|e| format!("Failed to read file {:?}: {}", path, e))?;
        let filename = path
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| format!("Invalid filename: {:?}", path))?
            .to_string();
        files.push((filename, content));
    }

    // Load all tasks
    let tasks = AssignmentTask::get_by_assignment_id(db, assignment_id)
        .await
        .map_err(|e| format!("DB error loading tasks: {}", e))?;

    if tasks.is_empty() {
        println!("No tasks found for assignment {}", assignment_id);
        return Ok(());
    }

    // Prepare HTTP client and endpoint URL
    let host =
        env::var("CODE_MANAGER_HOST").map_err(|_| "CODE_MANAGER_HOST not set".to_string())?;
    let port =
        env::var("CODE_MANAGER_PORT").map_err(|_| "CODE_MANAGER_PORT not set".to_string())?;
    let code_manager_url = format!("http://{}:{}/run", host, port);
    let client = Client::new();

    // Serialize config to JSON
    let config_value = serde_json::to_value(config)
        .map_err(|e| format!("Failed to serialize execution config: {}", e))?;

    for task in tasks {
        let filename = format!(
            "submission_task_{}_user_{}_attempt_{}.txt",
            task.task_number, user_id, attempt_number
        );

        let request_body = json!({
            "config": config_value,
            "commands": [task.command],
            "files": files,
        });

        let output = match client
            .post(&code_manager_url)
            .json(&request_body)
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    response
                        .text()
                        .await
                        .unwrap_or_else(|_| "Failed to read response body".to_string())
                } else {
                    format!("Code manager returned error: {}", response.status())
                }
            }
            Err(e) => {
                format!(
                    "HTTP request to code manager failed for task {}: {}",
                    task.task_number, e
                )
            }
        };

        if let Err(e) = SubmissionOutputModel::save_file(
            db,
            task.id,
            submission_id,
            &filename,
            output.as_bytes(),
        )
        .await
        {
            println!("Failed to save submission output: {}", e);
        }
    }

    Ok(())
}

fn extract_zip(
    zip_bytes: &[u8],
    max_total_uncompressed: u64,
    output_dir: &Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut archive = ZipArchive::new(Cursor::new(zip_bytes))?;
    let mut total_uncompressed = 0;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        total_uncompressed += file.size();

        if total_uncompressed > max_total_uncompressed {
            return Err("Archive too large when decompressed".into());
        }

        let raw_name = file.name();

        if raw_name.contains("..")
            || raw_name.starts_with('/')
            || raw_name.starts_with('\\')
            || raw_name.contains(':')
        {
            return Err(format!("Invalid file path in archive: {}", raw_name).into());
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

use db::models::assignment_file::{Column as AssignmentFileColumn, Entity as AssignmentFileEntity};
use db::models::assignment_submission::Entity as AssignmentSubmission;
use tokio::fs as async_fs;
use tokio::io::AsyncReadExt;
pub async fn create_main_from_interpreter(
    db: &DatabaseConnection,
    submission_id: i64,
    interpreter_cmd: &str,
    main_file_name: &str,
) -> Result<(), String> {
    let submission = AssignmentSubmission::find_by_id(submission_id)
        .one(db)
        .await
        .map_err(|e| format!("Failed to fetch submission: {}", e))?
        .ok_or_else(|| format!("Submission {} not found", submission_id))?;

    let assignment_id = submission.assignment_id;

    let interpreter_file = AssignmentFileEntity::find()
        .filter(AssignmentFileColumn::AssignmentId.eq(assignment_id))
        .filter(AssignmentFileColumn::FileType.eq("interpreter"))
        .one(db)
        .await
        .map_err(|e| format!("Failed to fetch interpreter file: {}", e))?
        .ok_or("Interpreter file not found")?;

    let assignment = Assignment::find_by_id(assignment_id)
        .one(db)
        .await
        .map_err(|e| format!("Failed to fetch assignment: {}", e))?
        .ok_or_else(|| format!("Assignment {} not found", assignment_id))?;

    let module_id = assignment.module_id;

    let storage_root = std::env::var("ASSIGNMENT_STORAGE_ROOT")
        .map_err(|_| "ASSIGNMENT_STORAGE_ROOT not set".to_string())?;
    let interpreter_path = PathBuf::from(&storage_root).join(&interpreter_file.path);

    if !interpreter_path.exists() {
        return Err(format!(
            "Interpreter file does not exist on disk: {}",
            interpreter_path.display()
        ));
    }

    let interpreter_bytes = async_fs::read(&interpreter_path)
        .await
        .map_err(|e| format!("Failed to read interpreter file: {}", e))?;

    let temp_dir = tempdir().map_err(|e| format!("Failed to create tempdir: {}", e))?;
    let temp_path = temp_dir.path();

    extract_zip(&interpreter_bytes, 1_000_000_000, temp_path)
        .map_err(|e| format!("Failed to extract interpreter archive: {}", e))?;

    let memory_arg = format!("--memory={}b", 500_000_000);
    let cpus_arg = format!("--cpus=1");
    let pids_arg = format!("--pids-limit=64");

    let mut docker_cmd = Command::new("docker");
    docker_cmd
        .arg("run")
        .arg("--rm")
        .arg("--network=none")
        .arg(memory_arg)
        .arg(cpus_arg)
        .arg(pids_arg)
        .arg("--security-opt=no-new-privileges")
        .arg("-v")
        .arg(format!("{}:/code:rw", temp_path.display()))
        .arg("universal-runner")
        .arg("sh")
        .arg("-c")
        .arg(interpreter_cmd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let docker_process = docker_cmd
        .spawn()
        .map_err(|e| format!("Failed to spawn docker: {}", e))?;

    let output = timeout(Duration::from_secs(60), docker_process.wait_with_output())
        .await
        .map_err(|_| "Docker command timed out".to_string())?
        .map_err(|e| format!("Docker command failed: {}", e))?;

    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let stderr_str = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        return Err(format!(
            "Interpreter Docker run failed:\nSTDOUT:\n{}\nSTDERR:\n{}",
            stdout_str, stderr_str
        ));
    }

    let mut generated_main = temp_path.join(main_file_name);
    if !generated_main.exists() {
        let lower_name = main_file_name.to_ascii_lowercase();
        let generated_main_lower = temp_path.join(&lower_name);
        if generated_main_lower.exists() {
            generated_main = generated_main_lower;
        } else {
            return Err(format!(
                "Interpreter did not produce expected main file: {} or {}",
                generated_main.display(),
                generated_main_lower.display()
            ));
        }
    }

    let mut main_content = Vec::new();
    async_fs::File::open(&generated_main)
        .await
        .map_err(|e| format!("Failed to open generated main file: {}", e))?
        .read_to_end(&mut main_content)
        .await
        .map_err(|e| format!("Failed to read generated main file: {}", e))?;

    let zip_filename = format!(
        "main_interpreted.{}.zip",
        main_file_name.rsplitn(2, '.').next().unwrap_or("txt")
    );

    use std::io::Write;
    use zip::write::{FileOptions, ZipWriter};

    let mut zip_data = Vec::new();
    {
        let mut zip_writer = ZipWriter::new(std::io::Cursor::new(&mut zip_data));
        zip_writer
            .start_file(main_file_name, FileOptions::<'_, ()>::default())
            .map_err(|e| format!("Failed to start file in zip: {}", e))?;
        zip_writer
            .write_all(&main_content)
            .map_err(|e| format!("Failed to write main file to zip: {}", e))?;
        zip_writer
            .finish()
            .map_err(|e| format!("Failed to finish zip: {}", e))?;
    }

    db::models::assignment_file::Model::save_file(
        db,
        assignment_id,
        module_id,
        db::models::assignment_file::FileType::Main,
        &zip_filename,
        &zip_data,
    )
    .await
    .map_err(|e| format!("Failed to save zipped main file: {}", e))?;

    Ok(())
}

/// Runs the interpreter for a given submission, generating and processing
/// the main file and outputs for all tasks associated with the assignment.
///
/// # Arguments
/// * `db` - Reference to the database connection for querying and saving data.
/// * `submission_id` - The unique ID of the submission to process.
/// * `interpreter_cmd` - The shell command to run the interpreter inside Docker or similar.
/// * `main_file_name` - The expected filename of the generated main file (e.g., "main.cpp").
///
/// # Returns
/// * `Result<(), String>` - Returns Ok(()) if all steps succeed, or an error message string on failure.
///
/// # Workflow:
/// 1. Fetches the submission by `submission_id` from the database to get its `assignment_id`.
/// 2. Calls `create_main_from_interpreter` to:
///     - Run the interpreter command inside Docker.
///     - Extract and read the generated main file named `main_file_name`.
///     - Zip and save this main file into the database as an assignment file.
/// 3. Calls `create_memo_outputs_for_all_tasks` to generate memo outputs for every task
///    associated with the assignment (using the assignment_id).
/// 4. Calls `create_submission_outputs_for_all_tasks` to generate outputs for every task
///    specifically for the given submission.
/// 5. Returns `Ok(())` on success or an error if any step fails.
///
/// This function coordinates the entire workflow for interpreting and processing
/// a student's submission according to the assignment tasks.
pub async fn run_interpreter(
    db: &DatabaseConnection,
    submission_id: i64,
    interpreter_cmd: &str,
    main_file_name: &str,
) -> Result<(), String> {
    use db::models::assignment_submission::Entity as AssignmentSubmission;
    let submission = AssignmentSubmission::find_by_id(submission_id)
        .one(db)
        .await
        .map_err(|e| format!("Failed to fetch submission: {}", e))?
        .ok_or_else(|| format!("Submission {} not found", submission_id))?;

    let assignment_id = submission.assignment_id;

    // Step 1: Create main zip from interpreter
    create_main_from_interpreter(db, submission_id, interpreter_cmd, main_file_name).await?;

    // Step 2: Create memo outputs for all tasks of this assignment
    create_memo_outputs_for_all_tasks(db, assignment_id).await?;

    // Step 3: Create submission outputs for all tasks for this submission
    create_submission_outputs_for_all_tasks(db, submission_id).await?;

    Ok(())
}
