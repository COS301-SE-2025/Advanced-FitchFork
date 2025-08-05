// Core dependencies
use std::fs::File;
use std::io::{Cursor, Read};
use std::{
    env, fs,
    path::{Component, Path, PathBuf},
    process::Stdio,
};

// Async, process, and timing
use tokio::process::Command;
use tokio::time::{Duration, timeout};

// External crates
use flate2::read::GzDecoder;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use tar::Archive as TarArchive;
use tempfile::tempdir;
use zip::ZipArchive;

// Your own modules
use crate::validate_files::validate_memo_files;

// Models
use db::models::assignment::Entity as Assignment;
use db::models::assignment_memo_output::{Column as MemoOutputColumn, Entity as MemoOutputEntity};
use db::models::assignment_task::Model as AssignmentTask;

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
        let mut abs_path = std::env::current_dir().unwrap();
        abs_path.pop();
        abs_path.push(path);
        abs_path
    } else {
        path
    }
}

/// Runs all configured tasks for a given assignment ID by:
/// 1. Validating memo files
/// 2. Extracting archive files
/// 3. Running the configured commands inside Docker
/// 4. Saving the resulting output as memo files in the database
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

    for task in tasks {
        let filename = format!("task_{}_output.txt", task.task_number);

        let output = match run_all_archives_with_command(
            archive_paths.clone(),
            &config,
            &task.command,
        )
        .await
        {
            Ok(output) => output,
            Err(err) => {
                println!("Task {} failed:\n{}", task.task_number, err);
                err.to_string() // Save error as output
            }
        };

        if let Err(e) = db::models::assignment_memo_output::Model::save_file(
            db,
            assignment_id,
            task.task_number,
            &filename,
            output.as_bytes(),
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

    use sea_orm::EntityTrait;

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

    let archive_paths = vec![
        first_archive_in(&submission_path)?,
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

    for task in tasks {
        let filename = format!(
            "submission_task_{}_user_{}_attempt_{}.txt",
            task.task_number, user_id, attempt_number
        );

        let output = match run_all_archives_with_command(
            archive_paths.clone(),
            &config,
            &task.command,
        )
        .await
        {
            Ok(output) => output,
            Err(err) => {
                println!(
                    "Task {} failed for user {} attempt {}:\n{}",
                    task.task_number, user_id, attempt_number, err
                );
                err.to_string() // Save error as file content
            }
        };

        if let Err(e) = SubmissionOutputModel::save_file(
            db,
            task.task_number,
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

/// Executes a set of archive files inside a Docker container using the specified command.
/// Captures and returns stdout output if successful, or full error output if not.
pub async fn run_all_archives_with_command(
    archive_paths: Vec<PathBuf>,
    config: &ExecutionConfig,
    custom_command: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let temp_code_dir = tempdir()?;
    let temp_output_dir = tempdir()?;

    let code_path = temp_code_dir.path().to_path_buf();
    let output_path = temp_output_dir.path().to_path_buf();

    for archive_path in archive_paths {
        let archive_bytes = std::fs::read(&archive_path)?;
        extract_archive_contents(
            &archive_bytes,
            config.execution.max_uncompressed_size,
            &code_path,
        )?;
    }

    let full_command = custom_command.to_string();

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
        .arg(&full_command)
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
        // println!("--- Output directory contents ---");
        // for entry in fs::read_dir(&output_path)? {
        //     let entry = entry?;
        //     let path = entry.path();
        // println!("File: {}", path.display());

        // if path
        //     .extension()
        //     .map(|ext| ext == "txt" || ext == "log")
        //     .unwrap_or(false)
        // {
        //     let content = fs::read_to_string(&path)?;
        //     println!("Content:\n{}", content);
        // }
        // }

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

/// Extracts the contents of a archive archive into the given output directory,
/// while checking for total uncompressed size and archive slip vulnerabilities.
fn extract_archive_contents(
    archive_bytes: &[u8],
    max_total_uncompressed: u64,
    output_dir: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if archive_bytes.starts_with(b"\x1F\x8B") {
        let mut decoder = GzDecoder::new(archive_bytes);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;

        if let Err(e) = extract_tar(
            Cursor::new(&decompressed),
            max_total_uncompressed,
            output_dir,
        ) {
            if e.downcast_ref::<std::io::Error>()
                .map(|ioe| ioe.kind() == std::io::ErrorKind::InvalidData)
                .unwrap_or(false)
            {
                extract_single_file(
                    &decompressed,
                    max_total_uncompressed,
                    output_dir,
                    "decompressed",
                )
            } else {
                Err(e)
            }
        } else {
            Ok(())
        }
    } else if archive_bytes.starts_with(b"PK") {
        extract_zip(archive_bytes, max_total_uncompressed, output_dir)
    } else {
        extract_tar(
            Cursor::new(archive_bytes),
            max_total_uncompressed,
            output_dir,
        )
    }
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

fn extract_tar<R: Read>(
    reader: R,
    max_total_uncompressed: u64,
    output_dir: &Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut archive = TarArchive::new(reader);
    let mut total_uncompressed = 0;

    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?.into_owned();
        let size = entry.size();

        if path.is_absolute() {
            return Err(format!("Absolute path in archive: {:?}", path).into());
        }

        if path.components().any(|c| matches!(c, Component::ParentDir)) {
            return Err(format!("Path traversal attempt: {:?}", path).into());
        }

        total_uncompressed += size;
        if total_uncompressed > max_total_uncompressed {
            return Err("Archive too large when decompressed".into());
        }

        let outpath = output_dir.join(&path);
        if entry.header().entry_type().is_dir() {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut outfile = File::create(&outpath)?;
            std::io::copy(&mut entry, &mut outfile)?;
        }
    }

    Ok(())
}

fn extract_single_file(
    contents: &[u8],
    max_total_uncompressed: u64,
    output_dir: &Path,
    filename: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let size = contents.len() as u64;
    if size > max_total_uncompressed {
        return Err("File too large when decompressed".into());
    }

    let outpath = output_dir.join(filename);
    std::fs::write(outpath, contents)?;
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

// Required imports
use k8s_openapi::api::batch::v1::Job;
use k8s_openapi::api::core::v1::{Container, PodSpec, PodTemplateSpec};
use kube::api::{DeleteParams, ListParams, PostParams};
use kube::{Api, Client, ResourceExt};
use tokio::time::sleep;
use uuid::Uuid;

/// Launches a Kubernetes Job in k3s that prints "Hello World", waits for it to complete,
/// and prints the logs to the main console.
pub async fn run_hello_world_k8s_job() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = Client::try_default().await?;
    let namespace = "default";
    let jobs: Api<Job> = Api::namespaced(client.clone(), namespace);

    let job_name = format!("job-{}", Uuid::new_v4().simple());

    // Define the job
    let job = Job {
        metadata: kube::api::ObjectMeta {
            name: Some(job_name.clone()),
            ..Default::default()
        },
        spec: Some(k8s_openapi::api::batch::v1::JobSpec {
            template: PodTemplateSpec {
                metadata: Some(kube::api::ObjectMeta {
                    name: Some(format!("pod-{}", job_name)),
                    ..Default::default()
                }),
                spec: Some(PodSpec {
                    containers: vec![Container {
                        name: "hello".to_string(),
                        image: Some("alpine".to_string()),
                        command: Some(vec![
                            "sh".to_string(),
                            "-c".to_string(),
                            "echo Hello World".to_string(),
                        ]),
                        ..Default::default()
                    }],
                    restart_policy: Some("Never".to_string()),
                    ..Default::default()
                }),
            },
            backoff_limit: Some(0),
            ..Default::default()
        }),
        status: None,
    };

    // Create job
    let pp = PostParams::default();
    jobs.create(&pp, &job).await?;

    // Wait for job to complete
    let mut attempts = 0;
    loop {
        let job_status = jobs.get_status(&job_name).await?;
        if let Some(status) = job_status.status {
            if let Some(conditions) = status.conditions {
                if conditions
                    .iter()
                    .any(|c| c.type_ == "Complete" && c.status == "True")
                {
                    break;
                }
            }
        }

        attempts += 1;
        if attempts > 20 {
            return Err("Job did not complete in time".into());
        }
        sleep(Duration::from_secs(1)).await;
    }

    // Fetch logs from the pod
    let pods: Api<k8s_openapi::api::core::v1::Pod> = Api::namespaced(client.clone(), namespace);
    let lp = ListParams::default().labels(&format!("job-name={}", job_name));
    let pod_list = pods.list(&lp).await?;

    if let Some(pod) = pod_list.items.first() {
        let pod_name = pod.name_any();
        let logs = pods.logs(&pod_name, &Default::default()).await?;

        println!("Log output from pod: {}", logs);
    } else {
        return Err("No pod found for job".into());
    }

    // Clean up
    jobs.delete(&job_name, &DeleteParams::background()).await?;

    Ok(())
}

#[tokio::test]
async fn test_run_hello_world_k8s_job() {
    match run_hello_world_k8s_job().await {
        Ok(_) => println!("Kubernetes job ran successfully!"),
        Err(e) => panic!("Failed to run Kubernetes job: {}", e),
    }
}
