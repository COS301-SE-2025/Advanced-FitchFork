use chrono::Utc;
use code_runner::run_interpreter;
use services::service::Service;
use services::user::{UserService, CreateUser};
use services::module::{ModuleService, CreateModule};
use services::assignment::{AssignmentService, AssignmentType, CreateAssignment};
use services::assignment_task::{AssignmentTaskService, CreateAssignmentTask};
use services::assignment_submission::{AssignmentSubmissionService, AssignmentSubmission, CreateAssignmentSubmission};
use services::assignment_interpreter::{AssignmentInterpreterService, CreateAssignmentInterpreter};
use util::filters::FilterParam;
use util::state::AppState;
use zip::write::FileOptions;
use std::io::Write;

async fn seed_user() -> i64 {
    let user_id = 1;
    if UserService::find_by_id(user_id).await.expect("DB error during user lookup").is_none() {
        let _ = UserService::create(
            CreateUser {
                id: Some(user_id),
                username: "u00000001".to_string(),
                email: "testuser@example.com".to_string(),
                password: "hashedpassword".to_string(),
                admin: false,
            }
        ).await.expect("Failed to insert user");
    }
    user_id
}

async fn seed_module(module_id: i64, code: &str) {
    if ModuleService::find_by_id(module_id)
        .await
        .expect("DB error")
        .is_none()
    {
        let _ = ModuleService::create(
            CreateModule {
                id: Some(module_id),
                code: code.to_string(),
                year: 2025,
                description: Some(format!("Test module for ID {}", module_id)),
                credits: 12,
            }
        ).await.expect("Failed to insert module");
    }
}

async fn seed_assignment(assignment_id: i64, module_id: i64) {
    if AssignmentService::find_by_id(assignment_id)
        .await
        .expect("DB error")
        .is_none()
    {
        let _ = AssignmentService::create(
            CreateAssignment {
                id: Some(assignment_id),
                module_id,
                name: "Special Assignment".to_string(),
                description: Some("Special assignment for testing".to_string()),
                assignment_type: AssignmentType::Assignment,
                available_from: Utc::now(),
                due_date: Utc::now(),
            }
        ).await.expect("Failed to insert assignment");
    }
}

async fn seed_tasks(assignment_id: i64) {
    let mut tasks = vec![(1, "make task1"), (2, "make task2"), (3, "make task3")];
    if assignment_id == 9998 {
        tasks.push((4, "make task4"));
    }
    for (task_number, command) in tasks {
        let _ = AssignmentTaskService::create(
            CreateAssignmentTask {
                assignment_id,
                task_number,
                name: "Untitled Task".to_string(),
                command: command.to_string(),
                code_coverage: false,
            }
        ).await.expect("Failed to create assignment task");
    }
}

async fn seed_submission(
    assignment_id: i64,
) -> AssignmentSubmission {
    AssignmentSubmissionService::create(
        CreateAssignmentSubmission {
            assignment_id: assignment_id,
            user_id: seed_user().await,
            attempt: 1,
            earned: 50,
            total: 100,
            is_practice: false,
            filename: "submission.zip".to_string(),
            file_hash: "0".to_string(),
            bytes: create_dummy_zip(),
        }
    ).await.expect("Failed to insert submission")
}

async fn seed_interpreter_file(assignment_id: i64, interpreter_id: i64) {
    let assignment = AssignmentService::find_by_id(assignment_id)
        .await
        .expect("Failed to lookup assignment")
        .expect("Assignment not found");

    // The filename on disk is "{interpreter_id}.zip"
    let filename = format!("{}.zip", interpreter_id);

    // The command to run the interpreter - put your actual interpreter command here
    let command = "g++ /code/interpreter.cpp -o /code/interpreter_exe &&
 /code/interpreter_exe"
        .to_string();

    // Check if an interpreter for this assignment already exists
    if AssignmentInterpreterService::find_one(
        &vec![
            FilterParam::eq("assignment_id", assignment_id),
        ],
        &vec![],
        None,
    ).await
    .expect("DB error")
    .is_none()
    {
        let _ = AssignmentInterpreterService::create(
            CreateAssignmentInterpreter {
                assignment_id: assignment_id,
                module_id: assignment.module_id,
                filename: filename,
                command: command,
                bytes: create_dummy_zip(),
            }
        ).await.expect("Failed to create assignment interpreter");
    }
}

async fn setup_test_db_for_run_interpreter(
    assignment_id: i64,
    module_id: i64,
    interpreter_id: i64,
) -> i64 {
    seed_user().await;
    seed_module(module_id, &format!("COS{}", module_id)).await;
    seed_assignment(assignment_id, module_id).await;
    seed_tasks(assignment_id).await;
    let submission = seed_submission(assignment_id).await;
    seed_interpreter_file(assignment_id, interpreter_id).await;

    submission.id
}

fn create_dummy_zip() -> Vec<u8> {
    let mut buf = Vec::new();
    {
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));

        let options = FileOptions::<()>::default()
            .compression_method(zip::CompressionMethod::Stored);

        // Add a dummy file inside the zip
        zip.start_file("dummy.txt", options).unwrap();
        zip.write_all(b"Hello, world!").unwrap();

        // Finish writing zip (important: writes central directory + EOCD)
        zip.finish().unwrap();
    }
    buf
}

#[tokio::test]
#[ignore]
async fn test_run_interpreter_9998_cpp() {
    dotenvy::dotenv().ok();
    let _ = AppState::init(false);

    let assignment_id = 9998;
    let module_id = 9998;
    let interpreter_id = 19;

    let gene_string = "01234";

    let submission_id = setup_test_db_for_run_interpreter(
        assignment_id,
        module_id,
        interpreter_id,
    )
    .await;

    match run_interpreter(submission_id, &gene_string).await {
        Ok(_) => println!("run_interpreter completed successfully for assignment 9998."),
        Err(e) => panic!("run_interpreter failed: {}", e),
    }
}
