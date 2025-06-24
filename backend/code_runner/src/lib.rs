use db::models::assignment_memo_output::Model as MemoOutputModel;
use db::models::assignment_task::Model as AssignmentTask;
use sea_orm::DatabaseConnection;
use std::env;
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

/// Returns the first `.zip` file found in the given directory.
/// Returns an error if the directory does not exist or if no zip file is found.
fn first_zip_in(dir: &PathBuf) -> Result<PathBuf, String> {
    std::fs::read_dir(dir)
        .map_err(|_| format!("Missing directory: {}", dir.display()))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .find(|p| p.extension().map(|ext| ext == "zip").unwrap_or(false))
        .ok_or_else(|| format!("No .zip file found in {}", dir.display()))
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
/// 2. Extracting zip files
/// 3. Running the configured commands inside Docker
/// 4. Saving the resulting output as memo files in the database
pub async fn create_memo_outputs_for_all_tasks(
    db: &DatabaseConnection,
    assignment_id: i64,
) -> Result<(), String> {
    use crate::execution_config::ExecutionConfig;
    use crate::validate_files::validate_memo_files;
    use db::models::assignment::Entity as Assignment;
    use sea_orm::EntityTrait;

    // Fetch the assignment to get module_id
    let assignment = Assignment::find_by_id(assignment_id)
        .one(db)
        .await
        .map_err(|e| format!("Failed to fetch assignment: {}", e))?
        .ok_or_else(|| format!("Assignment {} not found", assignment_id))?;

    let module_id = assignment.module_id;

    validate_memo_files(module_id, assignment_id)?;

    let config = ExecutionConfig::get_execution_config(module_id, assignment_id)
        .map_err(|e| format!("Failed to load execution config: {}", e))?;

    let storage_root = env::var("ASSIGNMENT_STORAGE_ROOT")
        .map_err(|_| "ASSIGNMENT_STORAGE_ROOT not set".to_string())?;

    let base_path = resolve_storage_root(&storage_root)
        .join(format!("module_{}", module_id))
        .join(format!("assignment_{}", assignment_id));

    let zip_paths = vec![
        first_zip_in(&base_path.join("memo"))?,
        first_zip_in(&base_path.join("makefile"))?,
        first_zip_in(&base_path.join("main"))?,
    ];

    let tasks = AssignmentTask::get_by_assignment_id(db, assignment_id)
        .await
        .map_err(|e| format!("DB error loading tasks: {}", e))?;

    if tasks.is_empty() {
        println!("No tasks found for assignment {}", assignment_id);
        return Ok(());
    }

    for task in tasks {
        // println!(
        //     "\n--- Executing Task {}: {} ---",
        //     task.task_number, task.command
        // );

        match run_all_zips_with_command(zip_paths.clone(), &config, &task.command).await {
            Ok(output) => {
                // println!("Task {} output captured.", task.task_number);

                let filename = format!("task_{}_output.txt", task.task_number);
                match MemoOutputModel::save_file(
                    db,
                    assignment_id,
                    task.task_number,
                    &filename,
                    output.as_bytes(),
                )
                .await
                {
                    // Ok(saved) => println!("Saved output to: {}", saved.full_path().display()),
                    Ok(_) => {}
                    Err(e) => println!("Failed to save output: {}", e),
                }
            }
            Err(err) => {
                println!("Task {} failed:\n{}", task.task_number, err);
            }
        }
    }

    Ok(())
}

use db::models::assignment_submission_output::Model as SubmissionOutputModel;

/// Runs all configured tasks for a given assignment ID and student attempt by:
/// 1. Validating submission files
/// 2. Extracting zip files (submission, makefile, main)
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

    let zip_paths = vec![
        first_zip_in(&submission_path)?,
        first_zip_in(&base_path.join("makefile"))?,
        first_zip_in(&base_path.join("main"))?,
    ];

    let tasks = AssignmentTask::get_by_assignment_id(db, assignment_id)
        .await
        .map_err(|e| format!("DB error loading tasks: {}", e))?;

    if tasks.is_empty() {
        println!("No tasks found for assignment {}", assignment_id);
        return Ok(());
    }

    for task in tasks {
        match run_all_zips_with_command(zip_paths.clone(), &config, &task.command).await {
            Ok(output) => {
                let filename = format!(
                    "submission_task_{}_user_{}_attempt_{}.txt",
                    task.task_number, user_id, attempt_number
                );

                match SubmissionOutputModel::save_file(
                    db,
                    task.task_number,
                    submission_id,
                    &filename,
                    output.as_bytes(),
                )
                .await
                {
                    Ok(_) => {}
                    Err(e) => {
                        println!("Failed to save submission output: {}", e);
                    }
                }
            }
            Err(err) => {
                println!(
                    "Task {} failed for user {} attempt {}:\n{}",
                    task.task_number, user_id, attempt_number, err
                );
            }
        }
    }

    Ok(())
}

/// Executes a set of zip files inside a Docker container using the specified command.
/// Captures and returns stdout output if successful, or full error output if not.
pub async fn run_all_zips_with_command(
    zip_paths: Vec<PathBuf>,
    config: &ExecutionConfig,
    custom_command: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let temp_code_dir = tempdir()?;
    let temp_output_dir = tempdir()?;

    let code_path = temp_code_dir.path().to_path_buf();
    let output_path = temp_output_dir.path().to_path_buf();

    for zip_path in zip_paths {
        let zip_bytes = std::fs::read(&zip_path)?;
        extract_zip_contents(&zip_bytes, config.max_uncompressed_size, &code_path)?;
    }

    let full_command = custom_command.to_string();

    let docker_output = Command::new("docker")
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
        .arg("universal-runner")
        .arg("sh")
        .arg("-c")
        .arg(&full_command)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let output = timeout(
        Duration::from_secs(config.timeout_secs),
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

/// Extracts the contents of a zip archive into the given output directory,
/// while checking for total uncompressed size and zip slip vulnerabilities.
fn extract_zip_contents(
    zip_bytes: &[u8],
    max_total_uncompressed: u64,
    output_dir: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

        if raw_name.contains("..") || raw_name.starts_with('/') || raw_name.contains('\\') {
            return Err(format!("Invalid file path in zip: {}", raw_name).into());
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

//TODO - Add testing for bad code in new refactored environment -> Richard will do
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
