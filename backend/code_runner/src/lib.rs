use std::path::Path;
// Core dependencies
use std::{fs, path::PathBuf};

// use db::models::AssignmentSubmissionOutput;
// External crates
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use util::paths::{
    attempt_dir, main_dir, makefile_dir, memo_dir, memo_output_dir, overwrite_task_dir,
};
// Your own modules
use crate::validate_files::validate_memo_files;

// Models
use db::models::assignment::Entity as Assignment;
use db::models::assignment_memo_output::{Column as MemoOutputColumn, Entity as MemoOutputEntity};
use db::models::assignment_task::Model as AssignmentTask;
use reqwest::Client;
use serde_json::json;
use util::code_coverage_report::CoverageProcessor;
use util::config;
use util::execution_config::ExecutionConfig;
pub mod validate_files;

/// Returns true if the provided filename is a Makefile artifact (plain or archived).
fn is_makefile_artifact(name: &str) -> bool {
    return name.to_ascii_lowercase().contains("makefile");
}

/// Returns the first archive file (".zip", ".tar", ".tgz", ".gz") found in the given directory.
/// Returns an error if the directory does not exist or if no supported archive file is found.
fn first_archive_in<P: AsRef<Path>>(dir: P) -> Result<PathBuf, String> {
    let allowed_exts = ["zip", "tar", "tgz", "gz"];
    let dir = dir.as_ref();
    std::fs::read_dir(dir)
        .map_err(|_| format!("Missing directory: {}", dir.display()))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .find(|p| {
            p.extension()
                .and_then(|e| e.to_str())
                .map(|ext| allowed_exts.contains(&ext.to_ascii_lowercase().as_str()))
                .unwrap_or(false)
        })
        .ok_or_else(|| {
            format!(
                "No .zip, .tar, .tgz, or .gz file found in {}",
                dir.display()
            )
        })
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

    // Validate required input files (unchanged)
    validate_memo_files(module_id, assignment_id)?;

    // Load config (unchanged; uses util::execution_config under the hood)
    let config = ExecutionConfig::get_execution_config(module_id, assignment_id)
        .map_err(|e| format!("Failed to load execution config: {}", e))?;

    // Base and subdirs via helpers
    let memo_out_dir = memo_output_dir(module_id, assignment_id);

    // Delete old files on disk
    if memo_out_dir.exists() {
        fs::remove_dir_all(&memo_out_dir)
            .map_err(|e| format!("Failed to delete old memo_output dir: {}", e))?;
    }

    // Delete old entries from DB (unchanged)
    MemoOutputEntity::delete_many()
        .filter(MemoOutputColumn::AssignmentId.eq(assignment_id))
        .exec(db)
        .await
        .map_err(|e| format!("Failed to delete old memo outputs: {}", e))?;

    // Load archives with helpers
    let archive_paths = vec![
        first_archive_in(memo_dir(module_id, assignment_id))?,
        first_archive_in(makefile_dir(module_id, assignment_id))?,
        first_archive_in(main_dir(module_id, assignment_id))?,
    ];

    // Determine the base Makefile archive name so we can avoid overriding it
    let base_makefile_archive_name: String = archive_paths
        .get(1)
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string();

    let tasks = AssignmentTask::get_by_assignment_id(db, assignment_id)
        .await
        .map_err(|e| format!("DB error loading tasks: {}", e))?;

    if tasks.is_empty() {
        return Err("No tasks are defined for this assignment. Add at least one task before generating memo output.".to_string());
    }

    // Prepare HTTP client
    let client = Client::new();

    let host = config::code_manager_host();

    let port = config::code_manager_port();

    let code_manager_url = format!("http://{}:{}", host, port);

    // Read common archives once to avoid repeated disk IO
    let mut base_files: Vec<(String, Vec<u8>)> = Vec::new();
    for archive_path in &archive_paths {
        let content = std::fs::read(archive_path)
            .map_err(|e| format!("Failed to read archive file {:?}: {}", archive_path, e))?;
        let file_name = archive_path
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| format!("Invalid archive filename: {:?}", archive_path))?
            .to_string();
        base_files.push((file_name, content));
    }

    use std::sync::Arc;
    use tokio::sync::Semaphore;
    use tokio::task::JoinSet;
    use tokio::time::{Duration, sleep};

    let max_concurrency = std::cmp::max(
        1,
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
            / 2,
    );
    let semaphore = Arc::new(Semaphore::new(max_concurrency));
    let mut join_set = JoinSet::new();

    for task in tasks.into_iter() {
        if task.code_coverage {
            continue;
        }

        let filename = format!("task_{}_output.txt", task.task_number);
        let task_files_base = base_files.clone();
        let client_cloned = client.clone();
        let cm_url = code_manager_url.clone();
        let config_value = serde_json::to_value(&config)
            .map_err(|e| format!("Failed to serialize ExecutionConfig: {}", e))?;
        let db_cloned = db.clone();
        let sem = semaphore.clone();
        let base_makefile_archive_name_cloned = base_makefile_archive_name.clone();
        join_set.spawn(async move {
            let _permit = sem.acquire_owned().await.ok();
            // Apply overwrites for this task
            let mut files = task_files_base;
            let overwrite_dir = overwrite_task_dir(module_id, assignment_id, task.task_number);
            if overwrite_dir.exists() {
                if let Ok(entries) = std::fs::read_dir(&overwrite_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_file() {
                            if let Ok(content) = std::fs::read(&path) {
                                if let Some(file_name) = path
                                    .file_name()
                                    .and_then(|s| s.to_str())
                                    .map(|s| s.to_string())
                                {
                                    // Allow overriding only if it's the base makefile archive; block other Makefile artifacts
                                    let is_make_art = is_makefile_artifact(&file_name);
                                    let is_base_archive = file_name == base_makefile_archive_name_cloned;
                                    if is_make_art && !is_base_archive {
                                        // ignore attempting to override Makefile outside the base archive
                                    } else {
                                        files.retain(|(name, _)| name != &file_name);
                                        files.push((file_name, content));
                                    }
                                }
                            }
                        }
                    }
                }
            }

            let request_body = serde_json::json!({
                "config": config_value,
                "commands": [task.command.clone()],
                "files": files,
            });

            // Fire request
            let resp_res = client_cloned
                .post(format!("{}/run", cm_url))
                .json(&request_body)
                .send()
                .await;
            let response = match resp_res {
                Ok(r) => r,
                Err(e) => {
                    return Err(format!(
                        "Failed to send request to code_manager (task {}): {}",
                        task.task_number, e
                    ));
                }
            };

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                return Err(format!(
                    "code_manager error for task {}: {} {}",
                    task.task_number, status, text
                ));
            }

            let resp_json: serde_json::Value = match response.json().await {
                Ok(v) => v,
                Err(e) => {
                    return Err(format!(
                        "Failed to parse response JSON for task {}: {}",
                        task.task_number, e
                    ));
                }
            };

            let output_vec = resp_json
                .get("output")
                .and_then(|v| v.as_array())
                .ok_or_else(|| {
                    format!(
                        "Response missing 'output' array for task {}",
                        task.task_number
                    )
                })?
                .iter()
                .map(|val| val.as_str().unwrap_or("").to_string())
                .collect::<Vec<String>>();
            let output_combined = output_vec.join("\n");

            // Save with retries to mitigate transient locks
            for attempt in 0..5 {
                match db::models::assignment_memo_output::Model::save_file(
                    &db_cloned,
                    assignment_id,
                    task.id,
                    &filename,
                    output_combined.as_bytes(),
                )
                .await
                {
                    Ok(_) => return Ok::<(), String>(()),
                    Err(e) => {
                        let backoff_ms = 20u64 * (1 << attempt);
                        println!(
                            "Retry {}/5 saving memo output for task {} ({} ms): {}",
                            attempt + 1,
                            task.task_number,
                            backoff_ms,
                            e
                        );
                        sleep(Duration::from_millis(backoff_ms)).await;
                    }
                }
            }
            Err(format!(
                "Failed to save memo output for task {} after retries",
                task.task_number
            ))
        });
    }

    // Collect errors if any
    let mut first_err: Option<String> = None;
    while let Some(res) = join_set.join_next().await {
        match res {
            Ok(Ok(())) => {}
            Ok(Err(e)) => {
                if first_err.is_none() {
                    first_err = Some(e);
                }
            }
            Err(e) => {
                if first_err.is_none() {
                    first_err = Some(format!("Join error: {}", e));
                }
            }
        }
    }

    if let Some(e) = first_err {
        return Err(e);
    }

    Ok(())
}

use db::models::assignment_submission_output::Model as SubmissionOutputModel;

/// Runs all configured tasks for a given assignment ID and student attempt by:
/// 1. Validating submission files
/// 2. Extracting archive files (submission, makefile, main)
/// 3. Running the configured commands inside Docker
/// 4. Saving the output to disk and database as `assignment_submission_output`
pub async fn create_submission_outputs_for_all_tasks_for_interpreter(
    db: &DatabaseConnection,
    submission_id: i64,
) -> Result<Vec<(i64, String)>, String> {
    use crate::validate_files::validate_submission_files;
    use db::models::assignment::Entity as Assignment;
    use db::models::assignment_submission::Entity as AssignmentSubmission;
    use reqwest::Client;
    use sea_orm::EntityTrait;
    use serde_json::json;
    use tokio::fs::read;

    SubmissionOutputModel::delete_for_submission(db, submission_id)
        .await
        .map_err(|e| format!("Failed to fetch submission: {}", e))?;

    // Fetch submission
    let submission = AssignmentSubmission::find_by_id(submission_id)
        .one(db)
        .await
        .map_err(|e| format!("Failed to fetch submission: {}", e))?
        .ok_or_else(|| format!("Submission {} not found", submission_id))?;

    let assignment_id = submission.assignment_id;
    let user_id = submission.user_id;
    let attempt_number = submission.attempt;

    // Fetch assignment
    let assignment = Assignment::find_by_id(assignment_id)
        .one(db)
        .await
        .map_err(|e| format!("Failed to fetch assignment: {}", e))?
        .ok_or_else(|| format!("Assignment {} not found", assignment_id))?;

    let module_id = assignment.module_id;

    // Validate files (unchanged)
    validate_submission_files(module_id, assignment_id, user_id, attempt_number)?;

    // Load config (unchanged)
    let config = ExecutionConfig::get_execution_config(module_id, assignment_id)
        .map_err(|e| format!("Failed to load execution config: {}", e))?;

    // Helper-based dirs
    let submission_path = attempt_dir(module_id, assignment_id, user_id, attempt_number);

    // Archives
    let archive_paths = vec![
        first_archive_in(&submission_path)?, // submission archive in attempt dir
        first_archive_in(makefile_dir(module_id, assignment_id))?,
        first_archive_in(main_dir(module_id, assignment_id))?,
    ];

    // Determine the base Makefile archive name so we can avoid overriding it
    let base_makefile_archive_name: String = archive_paths
        .get(1)
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string();

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

    // Get tasks
    let tasks = AssignmentTask::get_by_assignment_id(db, assignment_id)
        .await
        .map_err(|e| format!("DB error loading tasks: {}", e))?;

    if tasks.is_empty() {
        println!("No tasks found for assignment {}", assignment_id);
        return Ok(Vec::new());
    }

    // HTTP client setup
    let host = config::code_manager_host();
    let port = config::code_manager_port();
    let code_manager_url = format!("http://{}:{}/run", host, port);
    let client = Client::new();

    // Serialize config
    let config_value = serde_json::to_value(&config)
        .map_err(|e| format!("Failed to serialize execution config: {}", e))?;
    // Run all tasks concurrently while preserving original order by index
    use std::sync::Arc;
    use tokio::sync::Semaphore;
    use tokio::task::JoinSet;
    use tokio::time::{Duration, sleep};
    let mut join_set = JoinSet::new();

    // Bounded concurrency to avoid DB locking or overwhelming code manager
    let max_concurrency = std::cmp::max(
        1,
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
            / 2,
    );
    let semaphore = Arc::new(Semaphore::new(max_concurrency));

    // enumerate to preserve order
    for (idx, task) in tasks.into_iter().enumerate() {
        let filename = format!(
            "submission_task_{}_user_{}_attempt_{}.txt",
            task.task_number, user_id, attempt_number
        );

        let task_files_base = files.clone();
        let cm_url = code_manager_url.clone();
        let client_cloned = client.clone();
        let config_value_cloned = config_value.clone();
        let db_cloned = db.clone();
        let module_id_cloned = module_id;
        let assignment_id_cloned = assignment_id;
        let submission_path_cloned = submission_path.clone();
        let base_makefile_archive_name_cloned = base_makefile_archive_name.clone();

        let sem = semaphore.clone();
        join_set.spawn(async move {
            let _permit = sem.acquire_owned().await.ok();
            // Prepare task-specific files (apply overwrites)
            let mut task_files = task_files_base;
            let overwrite_dir =
                overwrite_task_dir(module_id_cloned, assignment_id_cloned, task.task_number);
            if overwrite_dir.exists() {
                if let Ok(entries) = std::fs::read_dir(&overwrite_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_file() {
                            if let Ok(content) = std::fs::read(&path) {
                                if let Some(file_name) = path
                                    .file_name()
                                    .and_then(|s| s.to_str())
                                    .map(|s| s.to_string())
                                {
                                    // Allow overriding only if it's the base makefile archive; block other Makefile artifacts
                                    let is_make_art = is_makefile_artifact(&file_name);
                                    let is_base_archive = file_name == base_makefile_archive_name_cloned;
                                    if is_make_art && !is_base_archive {
                                        // ignore attempting to override Makefile outside the base archive
                                    } else {
                                        task_files.retain(|(name, _)| name != &file_name);
                                        task_files.push((file_name, content));
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Compose request
            let request_body = json!({
                "config": config_value_cloned,
                "commands": [task.command.clone()],
                "files": task_files,
            });

            // Send request
            let response_res = client_cloned.post(&cm_url).json(&request_body).send().await;
            match response_res {
                Err(e) => {
                    println!("HTTP request failed for task {}: {}", task.task_number, e);
                    return None as Option<(usize, i64, String, String)>;
                }
                Ok(response) => {
                    if !response.status().is_success() {
                        let text = response.text().await.unwrap_or_default();
                        println!("Code manager error for task {}: {}", task.task_number, text);
                        return None;
                    }

                    let resp_json: serde_json::Value = match response.json().await {
                        Ok(v) => v,
                        Err(e) => {
                            println!(
                                "Failed to parse response JSON for task {}: {}",
                                task.task_number, e
                            );
                            return None;
                        }
                    };

                    let output_vec = match resp_json.get("output").and_then(|v| v.as_array()) {
                        Some(arr) => arr
                            .iter()
                            .map(|val| val.as_str().unwrap_or("").to_string())
                            .collect::<Vec<String>>(),
                        None => {
                            println!(
                                "Response missing 'output' array for task {}",
                                task.task_number
                            );
                            Vec::new()
                        }
                    };

                    let output_combined = output_vec.join("\n");

                    if task.code_coverage {
                        match CoverageProcessor::process_report(
                            config.project.language,
                            &output_combined,
                        ) {
                            Ok(coverage_json) => {
                                let coverage_report_path =
                                    submission_path_cloned.join("coverage_report.json");
                                if let Err(e) =
                                    std::fs::write(&coverage_report_path, &coverage_json)
                                {
                                    println!(
                                        "Failed to save coverage report to attempt directory: {}",
                                        e
                                    );
                                } else {
                                    println!(
                                        "Coverage report saved to: {:?}",
                                        coverage_report_path
                                    );
                                }
                            }
                            Err(e) => {
                                println!(
                                    "Failed to process coverage report for task {}: {}",
                                    task.task_number, e
                                );
                            }
                        }
                    } else {
                        // Save file with simple retry (helps with SQLite write locks)
                        let mut saved = false;
                        for attempt in 0..5 {
                            match SubmissionOutputModel::save_file(
                                &db_cloned,
                                task.id,
                                submission_id,
                                &filename,
                                output_combined.as_bytes(),
                            )
                            .await
                            {
                                Ok(_) => {
                                    saved = true;
                                    break;
                                }
                                Err(e) => {
                                    let backoff_ms = 20u64 * (1 << attempt);
                                    println!(
                                        "Retry {}/5 saving output for task {} ({} ms): {}",
                                        attempt + 1,
                                        task.task_number,
                                        backoff_ms,
                                        e
                                    );
                                    sleep(Duration::from_millis(backoff_ms)).await;
                                }
                            }
                        }
                        if !saved {
                            println!(
                                "Failed to save submission output for task {} after retries",
                                task.task_number
                            );
                        }
                    }

                    Some((idx, task.id, filename, output_combined))
                }
            }
        });
    }

    // Collect results preserving order by idx
    let mut results: Vec<(usize, i64, String, String)> = Vec::new();
    while let Some(res) = join_set.join_next().await {
        if let Ok(Some(tuple)) = res {
            results.push(tuple);
        }
    }
    results.sort_by_key(|(idx, _, _, _)| *idx);

    // Return collected outputs in original order (task_id, output)
    let collected: Vec<(i64, String)> = results
        .into_iter()
        .map(|(_, task_id, _fname, output)| (task_id, output))
        .collect();

    Ok(collected)
}

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
    use tokio::fs::read;

    // Remove any existing outputs for this submission to avoid stale DB rows
    SubmissionOutputModel::delete_for_submission(db, submission_id)
        .await
        .map_err(|e| format!("Failed to clear old submission outputs: {}", e))?;

    // Fetch submission
    let submission = AssignmentSubmission::find_by_id(submission_id)
        .one(db)
        .await
        .map_err(|e| format!("Failed to fetch submission: {}", e))?
        .ok_or_else(|| format!("Submission {} not found", submission_id))?;

    let assignment_id = submission.assignment_id;
    let user_id = submission.user_id;
    let attempt_number = submission.attempt;

    // Fetch assignment
    let assignment = Assignment::find_by_id(assignment_id)
        .one(db)
        .await
        .map_err(|e| format!("Failed to fetch assignment: {}", e))?
        .ok_or_else(|| format!("Assignment {} not found", assignment_id))?;

    let module_id = assignment.module_id;

    // Validate files (unchanged)
    validate_submission_files(module_id, assignment_id, user_id, attempt_number)?;

    // Load config (unchanged)
    let config = ExecutionConfig::get_execution_config(module_id, assignment_id)
        .map_err(|e| format!("Failed to load execution config: {}", e))?;

    // Paths via helpers
    let submission_path = attempt_dir(module_id, assignment_id, user_id, attempt_number);

    let archive_paths = vec![
        first_archive_in(&submission_path)?,
        first_archive_in(makefile_dir(module_id, assignment_id))?,
        first_archive_in(main_dir(module_id, assignment_id))?,
    ];

    // Determine the base Makefile archive name so we can avoid overriding it
    let base_makefile_archive_name: String = archive_paths
        .get(1)
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string();

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

    // Get tasks
    let tasks = AssignmentTask::get_by_assignment_id(db, assignment_id)
        .await
        .map_err(|e| format!("DB error loading tasks: {}", e))?;

    if tasks.is_empty() {
        println!("No tasks found for assignment {}", assignment_id);
        return Ok(());
    }

    // HTTP client setup
    let host = config::code_manager_host();
    let port = config::code_manager_port();
    let code_manager_url = format!("http://{}:{}/run", host, port);
    let client = Client::new();

    // Serialize config
    let config_value = serde_json::to_value(&config)
        .map_err(|e| format!("Failed to serialize execution config: {}", e))?;

    // Run tasks concurrently
    use std::sync::Arc;
    use tokio::sync::Semaphore;
    use tokio::task::JoinSet;
    use tokio::time::{Duration, sleep};
    let mut join_set = JoinSet::new();

    // Bounded concurrency to avoid DB write contention
    let max_concurrency = std::cmp::max(
        1,
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
            / 2,
    );
    let semaphore = Arc::new(Semaphore::new(max_concurrency));

    for task in tasks {
        let filename = format!(
            "submission_task_{}_user_{}_attempt_{}.txt",
            task.task_number, user_id, attempt_number
        );
        let task_files_base = files.clone();
        let cm_url = code_manager_url.clone();
        let client_cloned = client.clone();
        let config_value_cloned = config_value.clone();
        let db_cloned = db.clone();
        let module_id_cloned = module_id;
        let assignment_id_cloned = assignment_id;
        let submission_path_cloned = submission_path.clone();
        let base_makefile_archive_name_cloned = base_makefile_archive_name.clone();

        let sem = semaphore.clone();
        join_set.spawn(async move {
            let _permit = sem.acquire_owned().await.ok();
            // Prepare task-specific files (apply overwrites)
            let mut task_files = task_files_base;
            let overwrite_dir =
                overwrite_task_dir(module_id_cloned, assignment_id_cloned, task.task_number);
            if overwrite_dir.exists() {
                if let Ok(entries) = std::fs::read_dir(&overwrite_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_file() {
                            if let Ok(content) = std::fs::read(&path) {
                                if let Some(file_name) = path
                                    .file_name()
                                    .and_then(|s| s.to_str())
                                    .map(|s| s.to_string())
                                {
                                    // Allow overriding only if it's the base makefile archive; block other Makefile artifacts
                                    let is_make_art = is_makefile_artifact(&file_name);
                                    let is_base_archive = file_name == base_makefile_archive_name_cloned;
                                    if is_make_art && !is_base_archive {
                                        // ignore attempting to override Makefile outside the base archive
                                    } else {
                                        task_files.retain(|(name, _)| name != &file_name);
                                        task_files.push((file_name, content));
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Compose request
            let request_body = json!({
                "config": config_value_cloned,
                "commands": [task.command.clone()],
                "files": task_files,
            });

            let response_res = client_cloned.post(&cm_url).json(&request_body).send().await;
            match response_res {
                Err(e) => {
                    println!("HTTP request failed for task {}: {}", task.task_number, e);
                    return false;
                }
                Ok(response) => {
                    if !response.status().is_success() {
                        let text = response.text().await.unwrap_or_default();
                        println!("Code manager error for task {}: {}", task.task_number, text);
                        return false;
                    }

                    let resp_json: serde_json::Value = match response.json().await {
                        Ok(v) => v,
                        Err(e) => {
                            println!(
                                "Failed to parse response JSON for task {}: {}",
                                task.task_number, e
                            );
                            return false;
                        }
                    };

                    let output_vec = match resp_json.get("output").and_then(|v| v.as_array()) {
                        Some(arr) => arr
                            .iter()
                            .map(|val| val.as_str().unwrap_or("").to_string())
                            .collect::<Vec<String>>(),
                        None => {
                            println!(
                                "Response missing 'output' array for task {}",
                                task.task_number
                            );
                            Vec::new()
                        }
                    };

                    let output_combined = output_vec.join("\n");

                    if task.code_coverage {
                        match CoverageProcessor::process_report(config.project.language, &output_combined) {
                            Ok(coverage_json) => {
                                let coverage_report_path = submission_path_cloned.join("coverage_report.json");
                                match std::fs::write(&coverage_report_path, &coverage_json) {
                                    Ok(_) => {
                                        return true;
                                    }
                                    Err(e) => {
                                        println!("Failed to save coverage report to attempt directory: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                println!("Failed to process coverage report for task {}: {}", task.task_number, e);
                            }
                        }
                    } else {
                        // Save with retry to mitigate transient DB locks
                        for attempt in 0..5 {
                            match SubmissionOutputModel::save_file(
                                &db_cloned,
                                task.id,
                                submission_id,
                                &filename,
                                output_combined.as_bytes(),
                            )
                            .await
                            {
                                Ok(_) => {
                                    return true;
                                }
                                Err(e) => {
                                    let backoff_ms = 20u64 * (1 << attempt);
                                    println!(
                                        "Retry {}/5 saving output for task {} ({} ms): {}",
                                        attempt + 1,
                                        task.task_number,
                                        backoff_ms,
                                        e
                                    );
                                    sleep(Duration::from_millis(backoff_ms)).await;
                                }
                            }
                        }
                        println!(
                            "Failed to save submission output for task {} after retries",
                            task.task_number
                        );
                    }
                    false
                }
            }
        });
    }

    // Drain all tasks
    let mut saved_any = 0usize;
    while let Some(res) = join_set.join_next().await {
        if let Ok(saved) = res {
            if saved {
                saved_any += 1;
            }
        }
    }

    if saved_any == 0 {
        return Err("No submission outputs were generated".to_string());
    }

    Ok(())
}

pub async fn create_main_from_interpreter(
    db: &DatabaseConnection,
    submission_id: i64,
    generated_string: &str,
) -> Result<(), String> {
    use db::models::assignment::Entity as AssignmentEntity;
    use db::models::assignment_file::{FileType, Model as AssignmentFileModel};
    use db::models::assignment_interpreter::{
        Column as InterpreterColumn, Entity as AssignmentInterpreterEntity,
    };
    use db::models::assignment_submission::Entity as AssignmentSubmissionEntity;

    use reqwest::Client;
    use serde_json::json;
    use std::env;
    use std::io::Write;
    use util::execution_config::ExecutionConfig;
    use util::languages::LanguageExt;
    use zip::write::{FileOptions, ZipWriter};

    // --- Fetch submission, assignment, interpreter rows ---
    let submission = AssignmentSubmissionEntity::find_by_id(submission_id)
        .one(db)
        .await
        .map_err(|e| format!("Failed to fetch submission: {}", e))?
        .ok_or_else(|| format!("Submission {} not found", submission_id))?;

    let assignment_id = submission.assignment_id;

    let interpreter = AssignmentInterpreterEntity::find()
        .filter(InterpreterColumn::AssignmentId.eq(assignment_id))
        .one(db)
        .await
        .map_err(|e| format!("Failed to fetch interpreter: {}", e))?
        .ok_or_else(|| "Interpreter not found".to_string())?;

    let assignment = AssignmentEntity::find_by_id(assignment_id)
        .one(db)
        .await
        .map_err(|e| format!("Failed to fetch assignment: {}", e))?
        .ok_or_else(|| format!("Assignment {} not found", assignment_id))?;

    let module_id = assignment.module_id;

    // // Debug: show the interpreter command & payload
    // eprintln!("Using interpreter: {}", interpreter.command);
    // eprintln!("Assignment name: {}", assignment.name);
    // if config::ga_debug_print() == Some("1") {
    //     eprintln!("[DEBUG] generated_string = {}", generated_string);
    // }

    // Load full execution config (includes language)
    let config = ExecutionConfig::get_execution_config(module_id, assignment_id)
        .map_err(|e| format!("Failed to load execution config: {}", e))?;

    // Determine main file name from language
    let lang = config.project.language;
    let main_file_name = lang.main_filename();

    // Heuristic: if the "interpreter" is actually a compile/run line (e.g., g++ Main.cpp),
    // then there's no source to compile yet. Synthesize a Main.cpp (or Main.*)
    // from the generated_string and save it as the main archive locally.
    let looks_like_compile = lang.is_compile_cmd(&interpreter.command);

    if looks_like_compile {
        // --- STOPGAP BRANCH ---
        // Build a simple source file from `generated_string`.
        // Adjust templates per language as needed.
        let synthesized = lang
            .synthesize_program(generated_string)
            .unwrap_or_else(|| {
                // very safe fallback (keeps old behavior working even if a new lang lacks a template)
                format!("// synthesized stub\n// {}\n", generated_string)
            });

        // Zip and save as the "main" archive
        let zip_ext = std::path::Path::new(main_file_name)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("txt");
        let zip_filename = format!("main_interpreted.{}.zip", zip_ext);

        let mut zip_data = Vec::new();
        {
            let mut zip_writer = ZipWriter::new(std::io::Cursor::new(&mut zip_data));
            zip_writer
                .start_file(main_file_name, FileOptions::<()>::default())
                .map_err(|e| format!("zip start_file failed: {}", e))?;
            zip_writer
                .write_all(synthesized.as_bytes())
                .map_err(|e| format!("zip write failed: {}", e))?;
            zip_writer
                .finish()
                .map_err(|e| format!("zip finish failed: {}", e))?;
        }

        AssignmentFileModel::save_file(
            db,
            assignment_id,
            module_id,
            FileType::Main,
            &zip_filename,
            &zip_data,
        )
        .await
        .map_err(|e| format!("Failed to save synthesized main zip: {}", e))?;

        if env::var("GA_DEBUG_PRINT").ok().as_deref() == Some("1") {
            eprintln!(
                "[DEBUG] synthesized {} ({} bytes) into {}",
                main_file_name,
                zip_data.len(),
                zip_filename
            );
        }

        return Ok(());
    }

    // --- GENERATOR BRANCH (original intent) ---
    // The interpreter is a true generator: run it and expect source code on stdout.
    let interpreter_bytes = interpreter
        .load_file()
        .map_err(|e| format!("Failed to load interpreter file from disk: {}", e))?;

    // Combine the interpreter command with the GA-produced string.
    // e.g., "python3 interpreter.py <args>"
    let command = format!("{} \"{}\"", interpreter.command, generated_string);

    let host = config::code_manager_host();
    let port = config::code_manager_port();
    let url = format!("http://{}:{}/run", host, port);

    let config_value = serde_json::to_value(&config)
        .map_err(|e| format!("Failed to serialize execution config: {}", e))?;

    // Send interpreter.zip + command to the code manager
    let client = Client::new();
    let payload = json!({
        "config": config_value,
        "commands": [command],
        "files": [("interpreter.zip", interpreter_bytes)],
    });

    let resp = client
        .post(&url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Failed to send request to code_manager: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!(
            "code_manager returned error status {}: Response body: {}",
            status, body
        ));
    }

    #[derive(serde::Deserialize)]
    struct RunResponse {
        output: Vec<String>,
    }

    let run_resp: RunResponse = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse code_manager response: {}", e))?;

    let mut combined_output = run_resp.output.join("\n");

    if env::var("GA_DEBUG_PRINT").ok().as_deref() == Some("1") {
        eprintln!(
            "[DEBUG] generator output preview = {}",
            &combined_output.chars().take(20000).collect::<String>()
        );
    }

    // Sanity-check: generator should produce plausible source
    let looks_like_source = lang.looks_like_source(&combined_output);

    if !looks_like_source {
        println!(
            "[DEBUG] generator output does not look like source code: {}",
            combined_output
        );
        return Err("Interpreter did not return plausible source code".to_string());
    }

    combined_output = combined_output
        .lines()
        .filter(|line| !line.trim_start().starts_with("Retcode:"))
        .collect::<Vec<_>>()
        .join("\n");

    // Zip the generated source as Main.*
    let zip_ext = std::path::Path::new(main_file_name)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("txt");
    let zip_filename = format!("main_interpreted.{}.zip", zip_ext);

    let mut zip_data = Vec::new();
    {
        let mut zip_writer = ZipWriter::new(std::io::Cursor::new(&mut zip_data));
        zip_writer
            .start_file(main_file_name, FileOptions::<()>::default())
            .map_err(|e| format!("Failed to start file in zip: {}", e))?;
        zip_writer
            .write_all(combined_output.as_bytes())
            .map_err(|e| format!("Failed to write to zip: {}", e))?;
        zip_writer
            .finish()
            .map_err(|e| format!("Failed to finish zip: {}", e))?;
    }

    AssignmentFileModel::save_file(
        db,
        assignment_id,
        module_id,
        FileType::Main,
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
    db: &sea_orm::DatabaseConnection,
    submission_id: i64,
    generated_string: &str,
) -> Result<(), String> {
    use db::models::assignment_submission::Entity as AssignmentSubmission;

    let submission = AssignmentSubmission::find_by_id(submission_id)
        .one(db)
        .await
        .map_err(|e| format!("Failed to fetch submission: {}", e))?
        .ok_or_else(|| format!("Submission {} not found", submission_id))?;

    let assignment_id = submission.assignment_id;

    // Step 1
    create_main_from_interpreter(db, submission_id, generated_string).await?;

    // Step 2
    create_memo_outputs_for_all_tasks(db, assignment_id).await?;

    // Step 3
    create_submission_outputs_for_all_tasks(db, submission_id).await?;

    Ok(())
}
