use std::path::Path;
// Core dependencies
use std::{fs, path::PathBuf};

// use db::models::AssignmentSubmissionOutput;
// External crates
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use util::paths::{attempt_dir, main_dir, makefile_dir, memo_dir, memo_output_dir, overwrite_task_dir};
// Your own modules
use crate::validate_files::validate_memo_files;

// Models
use db::models::assignment::Entity as Assignment;
use db::models::assignment_memo_output::{Column as MemoOutputColumn, Entity as MemoOutputEntity};
use db::models::assignment_task::Model as AssignmentTask;
use reqwest::Client;
use serde_json::json;
use util::execution_config::ExecutionConfig;
use util::config;
pub mod validate_files;

/// Returns the first archive file (".zip", ".tar", ".tgz", ".gz") found in the given directory.
/// Returns an error if the directory does not exist or if no supported archive file is found.
fn first_archive_in<P: AsRef<Path>>(dir: P) -> Result<PathBuf, String> {
    let allowed_exts = ["zip", "tar", "tgz", "gz"];
    let dir = dir.as_ref();
    std::fs::read_dir(dir)
        .map_err(|_| format!("Missing directory: {}", dir.display()))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .find(|p| p
            .extension()
            .and_then(|e| e.to_str())
            .map(|ext| allowed_exts.contains(&ext.to_ascii_lowercase().as_str()))
            .unwrap_or(false)
        )
        .ok_or_else(|| format!("No .zip, .tar, .tgz, or .gz file found in {}", dir.display()))
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

    let tasks = AssignmentTask::get_by_assignment_id(db, assignment_id)
        .await
        .map_err(|e| format!("DB error loading tasks: {}", e))?;

    if tasks.is_empty() {
        println!("No tasks found for assignment {}", assignment_id);
        return Ok(());
    }

    // Prepare HTTP client
    let client = Client::new();

    let host = config::code_manager_host();

    let port = config::code_manager_port();

    let code_manager_url = format!("http://{}:{}", host, port);

    for task in tasks {
        if task.code_coverage {
            // println!(
            //     "Skipping task {} because code_coverage is true",
            //     task.task_number
            // );
            continue;
        }
        let filename = format!("task_{}_output.txt", task.task_number);

        let mut files: Vec<(String, Vec<u8>)> = Vec::new();

        for archive_path in &archive_paths {
            let content = std::fs::read(archive_path)
                .map_err(|e| format!("Failed to read archive file {:?}: {}", archive_path, e))?;
            let file_name = archive_path
                .file_name()
                .and_then(|s| s.to_str())
                .ok_or_else(|| format!("Invalid archive filename: {:?}", archive_path))?
                .to_string();
            files.push((file_name, content));
        }

        let overwrite_dir = overwrite_task_dir(module_id, assignment_id, task.task_number);
        if overwrite_dir.exists() {
            let entries = std::fs::read_dir(&overwrite_dir)
                .map_err(|e| format!("Failed to read overwrite dir: {}", e))?;

            for entry in entries {
                let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
                let path = entry.path();
                if path.is_file() {
                    let content = std::fs::read(&path)
                        .map_err(|e| format!("Failed to read overwrite file {:?}: {}", path, e))?;
                    let file_name = path
                        .file_name()
                        .and_then(|s| s.to_str())
                        .ok_or_else(|| format!("Invalid overwrite filename: {:?}", path))?
                        .to_string();

                    files.retain(|(name, _)| name != &file_name);
                    files.push((file_name, content));
                }
            }
        }

        let commands = vec![task.command.clone()];

        let config_value = serde_json::to_value(&config)
            .map_err(|e| format!("Failed to serialize ExecutionConfig: {}", e))?;

        let request_body = serde_json::json!({
            "config": config_value,
            "commands": commands,
            "files": files,
        });

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

        let resp_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response JSON: {}", e))?;

        let output_vec = resp_json
            .get("output")
            .and_then(|v| v.as_array())
            .ok_or_else(|| "Response missing 'output' array".to_string())?
            .iter()
            .map(|val| val.as_str().unwrap_or("").to_string())
            .collect::<Vec<String>>();

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
        first_archive_in(&submission_path)?,                 // submission archive in attempt dir
        first_archive_in(makefile_dir(module_id, assignment_id))?,
        first_archive_in(main_dir(module_id, assignment_id))?,
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

    // Get tasks
    let tasks = AssignmentTask::get_by_assignment_id(db, assignment_id)
        .await
        .map_err(|e| format!("DB error loading tasks: {}", e))?;

    if tasks.is_empty() {
        println!("No tasks found for assignment {}", assignment_id);
        return Ok(Vec::new());
    }

    // HTTP client setup
    let host =config::code_manager_host();
    let port = config::code_manager_port();
    let code_manager_url = format!("http://{}:{}/run", host, port);
    let client = Client::new();

    // Serialize config
    let config_value = serde_json::to_value(&config)
        .map_err(|e| format!("Failed to serialize execution config: {}", e))?;

    let mut collected: Vec<(i64, String)> = Vec::new();

    for task in tasks {
        let filename = format!(
            "submission_task_{}_user_{}_attempt_{}.txt",
            task.task_number, user_id, attempt_number
        );

        let mut task_files = files.clone();

        let overwrite_dir = overwrite_task_dir(module_id, assignment_id, task.task_number);
        if overwrite_dir.exists() {
            let entries = std::fs::read_dir(&overwrite_dir)
                .map_err(|e| format!("Failed to read overwrite dir: {}", e))?;

            for entry in entries {
                let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
                let path = entry.path();
                if path.is_file() {
                    let content = std::fs::read(&path)
                        .map_err(|e| format!("Failed to read overwrite file {:?}: {}", path, e))?;
                    let file_name = path
                        .file_name()
                        .and_then(|s| s.to_str())
                        .ok_or_else(|| format!("Invalid overwrite filename: {:?}", path))?
                        .to_string();

                    task_files.retain(|(name, _)| name != &file_name);
                    task_files.push((file_name, content));
                }
            }
        }

        // Compose request
        let request_body = json!({
            "config": config_value,
            "commands": [task.command],
            "files": task_files,
        });

        let response = client
            .post(&code_manager_url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed for task {}: {}", task.task_number, e))?;

        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            println!("Code manager error for task {}: {}", task.task_number, text);
            continue;
        }

        let resp_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response JSON: {}", e))?;

        let output_vec = resp_json
            .get("output")
            .and_then(|v| v.as_array())
            .ok_or_else(|| "Response missing 'output' array".to_string())?
            .iter()
            .map(|val| val.as_str().unwrap_or("").to_string())
            .collect::<Vec<String>>();

        let output_combined = output_vec.join("\n");

        if let Err(e) = SubmissionOutputModel::save_file(
            db,
            task.id,
            submission_id,
            &filename,
            output_combined.as_bytes(),
        )
        .await
        {
            println!("Failed to save submission output: {}", e);
        }

        collected.push((task.id, output_combined));
    }

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

    for task in tasks {
        let filename = format!(
            "submission_task_{}_user_{}_attempt_{}.txt",
            task.task_number, user_id, attempt_number
        );

        let mut task_files = files.clone();

        let overwrite_dir = overwrite_task_dir(module_id, assignment_id, task.task_number);
        if overwrite_dir.exists() {
            let entries = std::fs::read_dir(&overwrite_dir)
                .map_err(|e| format!("Failed to read overwrite dir: {}", e))?;

            for entry in entries {
                let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
                let path = entry.path();
                if path.is_file() {
                    let content = std::fs::read(&path)
                        .map_err(|e| format!("Failed to read overwrite file {:?}: {}", path, e))?;
                    let file_name = path
                        .file_name()
                        .and_then(|s| s.to_str())
                        .ok_or_else(|| format!("Invalid overwrite filename: {:?}", path))?
                        .to_string();

                    task_files.retain(|(name, _)| name != &file_name);
                    task_files.push((file_name, content));
                }
            }
        }

        // Compose request
        let request_body = json!({
            "config": config_value,
            "commands": [task.command],
            "files": task_files,
        });

        let response = client
            .post(&code_manager_url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed for task {}: {}", task.task_number, e))?;

        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            println!("Code manager error for task {}: {}", task.task_number, text);
            continue;
        }

        let resp_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response JSON: {}", e))?;

        let output_vec = resp_json
            .get("output")
            .and_then(|v| v.as_array())
            .ok_or_else(|| "Response missing 'output' array".to_string())?
            .iter()
            .map(|val| val.as_str().unwrap_or("").to_string())
            .collect::<Vec<String>>();

        let output_combined = output_vec.join("\n");

        if let Err(e) = SubmissionOutputModel::save_file(
            db,
            task.id,
            submission_id,
            &filename,
            output_combined.as_bytes(),
        )
        .await
        {
            println!("Failed to save submission output: {}", e);
        }
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
    use zip::write::{FileOptions, ZipWriter};
    use util::languages::{LanguageExt}; 

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

    if config.output.retcode == true {
        combined_output = combined_output
            .lines()
            .filter(|line| !line.trim_start().starts_with("Retcode:"))
            .collect::<Vec<_>>()
            .join("\n");
    }

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
