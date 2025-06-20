use db::models::assignment::AssignmentType;
use std::env;
use std::{fs, fs::File, io::Cursor, path::PathBuf, process::Stdio};
use tempfile::tempdir;
use tokio::{
    process::Command,
    time::{Duration, timeout},
};
use zip::ZipArchive;

pub mod execution_config;
use crate::execution_config::ExecutionConfig;
pub mod validate_files;

fn first_zip_in(dir: &PathBuf) -> Result<PathBuf, String> {
    std::fs::read_dir(dir)
        .map_err(|_| format!("Missing directory: {}", dir.display()))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .find(|p| p.extension().map(|ext| ext == "zip").unwrap_or(false))
        .ok_or_else(|| format!("No .zip file found in {}", dir.display()))
}

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

use db::models::assignment_task::Model as AssignmentTask;
use sea_orm::DatabaseConnection;

pub async fn create_memo_outputs_for_all_tasks(
    db: &DatabaseConnection,
    module_id: i64,
    assignment_id: i64,
) -> Result<(), String> {
    use crate::execution_config::ExecutionConfig;
    use crate::validate_files::validate_memo_files;

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
        println!(
            "\n--- Executing Task {}: {} ---",
            task.task_number, task.command
        );
        match run_all_zips_with_command(zip_paths.clone(), &config, &task.command).await {
            Ok(output) => println!("Output:\n{}", output),
            Err(err) => println!("Task {} failed:\n{}", task.task_number, err),
        }
    }

    Ok(())
}

async fn run_all_zips_with_command(
    zip_paths: Vec<PathBuf>,
    config: &ExecutionConfig,
    custom_command: &str,
) -> Result<String, Box<dyn std::error::Error>> {
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
        println!("--- Output directory contents ---");
        for entry in fs::read_dir(&output_path)? {
            let entry = entry?;
            let path = entry.path();
            println!("File: {}", path.display());

            if path
                .extension()
                .map(|ext| ext == "txt" || ext == "log")
                .unwrap_or(false)
            {
                let content = fs::read_to_string(&path)?;
                println!("Content:\n{}", content);
            }
        }

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

fn extract_zip_contents(
    zip_bytes: &[u8],
    max_total_uncompressed: u64,
    output_dir: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
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

use chrono::Utc;
use db::models::assignment::{ActiveModel as AssignmentActiveModel, Entity as AssignmentEntity};
use db::models::assignment_task::Model as AssignmentTaskModel;
use db::models::module::{ActiveModel as ModuleActiveModel, Entity as ModuleEntity};
use db::test_utils::setup_test_db;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};

async fn seed_module(db: &DatabaseConnection) {
    let module_id = 9999;

    let existing_module = ModuleEntity::find_by_id(module_id)
        .one(db)
        .await
        .expect("DB error during module lookup");

    if existing_module.is_none() {
        let module = ModuleActiveModel {
            id: Set(module_id),
            code: Set("COS999".to_string()),
            year: Set(2025),
            description: Set(Some("Test module for ID 9999".to_string())),
            credits: Set(12),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
        };

        module
            .insert(db)
            .await
            .expect("Failed to insert module with id 9999");
    }
}

async fn seed_assignment(db: &DatabaseConnection) {
    let assignment_id = 9999;
    let module_id = 9999;

    let existing_assignment = AssignmentEntity::find_by_id(assignment_id)
        .one(db)
        .await
        .expect("DB error during assignment lookup");

    if existing_assignment.is_none() {
        let assignment = AssignmentActiveModel {
            id: Set(assignment_id),
            module_id: Set(module_id),
            name: Set("Special Assignment".to_string()),
            description: Set(Some("Special assignment for testing".to_string())),
            assignment_type: Set(AssignmentType::Assignment),
            available_from: Set(Utc::now()),
            due_date: Set(Utc::now()),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            ..Default::default()
        };

        assignment
            .insert(db)
            .await
            .expect("Failed to insert assignment with id 9999");
    }
}

async fn seed_tasks(db: &DatabaseConnection) {
    let assignment_id = 9999;
    let tasks = vec![(1, "make task1"), (2, "make task2"), (3, "make task3")];

    for (task_number, command) in tasks {
        AssignmentTaskModel::create(db, assignment_id, task_number, command)
            .await
            .expect("Failed to create assignment task");
    }
}

pub async fn seed_module_assignment_and_tasks(db: &DatabaseConnection) {
    seed_module(db).await;
    seed_assignment(db).await;
    seed_tasks(db).await;
}

#[allow(dead_code)]
pub async fn setup_test_db_with_seeded_tasks() -> DatabaseConnection {
    let db = setup_test_db().await;

    seed_module_assignment_and_tasks(&db).await;

    db
}

#[tokio::test]
async fn test_create_memo_outputs_for_all_tasks_9999() {
    dotenv::dotenv().ok();

    let db = setup_test_db_with_seeded_tasks().await;

    let module_id = 9999;
    let assignment_id = 9999;

    match crate::create_memo_outputs_for_all_tasks(&db, module_id, assignment_id).await {
        Ok(_) => println!("✅ Memo outputs generated successfully for all tasks."),
        Err(e) => panic!("❌ Failed to generate memo outputs: {}", e),
    }
}

//TODO - Fix github actions to be able to run docker containers with all the languages
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
