// Core dependencies
use std::{env, fs, path::PathBuf};

// External crates
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
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
) -> Result<Vec<(i64, String)>, String>  {
    use crate::validate_files::validate_submission_files;
    use db::models::assignment::Entity as Assignment;
    use db::models::assignment_submission::Entity as AssignmentSubmission;
    use reqwest::Client;
    use sea_orm::EntityTrait;
    use serde_json::json;
    use std::env;
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

    // Validate files
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

    // Load archives
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

    // Get tasks
    let tasks = AssignmentTask::get_by_assignment_id(db, assignment_id)
        .await
        .map_err(|e| format!("DB error loading tasks: {}", e))?;

    if tasks.is_empty() {
        println!("No tasks found for assignment {}", assignment_id);
        return Ok(Vec::new());
    }

    // HTTP client setup
    let host =
        env::var("CODE_MANAGER_HOST").map_err(|_| "CODE_MANAGER_HOST not set".to_string())?;
    let port =
        env::var("CODE_MANAGER_PORT").map_err(|_| "CODE_MANAGER_PORT not set".to_string())?;
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

        let request_body = json!({
            "config": config_value,
            "commands": [task.command],
            "files": files,
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
    use util::execution_config::execution_config::Language;
    use zip::write::{FileOptions, ZipWriter};

    // Fetch submission, assignment, interpreter
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

    let interpreter_bytes = interpreter
        .load_file()
        .map_err(|e| format!("Failed to load interpreter file from disk: {}", e))?;

    let command = format!("{} {}", interpreter.command, generated_string);

    let host =
        env::var("CODE_MANAGER_HOST").map_err(|_| "CODE_MANAGER_HOST not set".to_string())?;
    let port =
        env::var("CODE_MANAGER_PORT").map_err(|_| "CODE_MANAGER_PORT not set".to_string())?;
    let url = format!("http://{}:{}/run", host, port);

    let config = ExecutionConfig::get_execution_config(module_id, assignment_id)
        .map_err(|e| format!("Failed to load execution config: {}", e))?;

    // Determine main file name based on language
    let main_file_name = match config.project.language {
        Language::Cpp => "Main.cpp",
        Language::Java => "Main.java",
        Language::Python => "Main.py",
    };

    let config_value = serde_json::to_value(&config)
        .map_err(|e| format!("Failed to serialize execution config: {}", e))?;

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
        return Err(format!("code_manager returned error: {}", resp.status()));
    }

    #[derive(serde::Deserialize)]
    struct RunResponse {
        output: Vec<String>,
    }

    let run_resp: RunResponse = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse code_manager response: {}", e))?;

    let combined_output = run_resp.output.join("\n");

    let zip_filename = format!(
        "main_interpreted.{}.zip",
        main_file_name.rsplit('.').next().unwrap_or("txt")
    );

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
) -> Result<Vec<(i64, String)>, String> {
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

    // Step 3 — now returns outputs
    let outputs = create_submission_outputs_for_all_tasks(db, submission_id).await?;
    Ok(outputs)
}