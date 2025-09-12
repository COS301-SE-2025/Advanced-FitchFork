use chrono::Utc;
use code_runner::create_submission_outputs_for_all_tasks;
use util::state::AppState;
use util::filters::FilterParam;
use services::service::Service;
use services::user::{UserService, CreateUser};
use services::module::{ModuleService, CreateModule};
use services::assignment::{AssignmentService, CreateAssignment};
use services::assignment_submission::{AssignmentSubmissionService, AssignmentSubmission, CreateAssignmentSubmission};
use services::assignment_task::{AssignmentTaskService, CreateAssignmentTask};

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

async fn seed_submission(assignment_id: i64) -> AssignmentSubmission {
    AssignmentSubmissionService::create(
        CreateAssignmentSubmission {
            assignment_id,
            user_id: 1,
            attempt: 1,
            earned: 80,
            total: 100,
            is_practice: false,
            filename: "submission.zip".to_string(),
            file_hash: "0".to_string(),
            bytes: vec![],
        }
    ).await.expect("Failed to insert submission")
}

async fn seed_module(module_id: i64, code: &str) {
    let existing_module = ModuleService::find_by_id(module_id)
        .await
        .expect("DB error during module lookup");

    if existing_module.is_none() {
        let _ = ModuleService::create(
            CreateModule {
                id: Some(module_id),
                code: code.to_string(),
                year: 2025,
                description: Some(format!("Test module for ID {}", module_id)),
                credits: 12,
            }
        ).await.expect(&format!("Failed to insert module with id {}", module_id));
    }
}

async fn seed_assignment(assignment_id: i64, module_id: i64) {
    let existing_assignment = AssignmentService::find_by_id(assignment_id)
        .await
        .expect("DB error during assignment lookup");

    if existing_assignment.is_none() {
        let _ = AssignmentService::create(
            CreateAssignment {
                id: Some(assignment_id),
                module_id,
                name: "Special Assignment".to_string(),
                description: Some("Special assignment for testing".to_string()),
                assignment_type: "assignment".to_string(),
                available_from: Utc::now(),
                due_date: Utc::now(),
            }
        ).await.expect(&format!("Failed to insert assignment with id {}", assignment_id));
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

async fn setup_test_db_with_seeded_tasks(assignment_id: i64, module_id: i64) {
    let _ = AppState::init(false);

    seed_user().await;
    seed_module(module_id, &format!("COS{}", module_id)).await;
    seed_assignment(assignment_id, module_id).await;
    seed_tasks(assignment_id).await;
    seed_submission(assignment_id).await;
}

#[tokio::test]
#[ignore]
async fn test_create_submission_outputs_for_all_tasks_9999_java() {
    dotenvy::dotenv().ok();

    setup_test_db_with_seeded_tasks(9999, 9999).await;

    let filters = vec![
        FilterParam::eq("assignment_id", 9999),
        FilterParam::eq("user_id", 1),
        FilterParam::eq("attempt", 1),
    ];
    let submission = AssignmentSubmissionService::find_one(&filters, None)
        .await
        .expect("DB error during submission lookup")
        .expect("No matching submission found");

    match create_submission_outputs_for_all_tasks(submission.id).await {
        Ok(_) => {}
        Err(e) => panic!("Failed to generate submission outputs: {}", e),
    }
}

#[tokio::test]
#[ignore]
async fn test_create_submission_outputs_for_all_tasks_9998_cpp() {
    dotenvy::dotenv().ok();

    setup_test_db_with_seeded_tasks(9998, 9998).await;

    let filters = vec![
        FilterParam::eq("assignment_id", 9998),
        FilterParam::eq("user_id", 1),
        FilterParam::eq("attempt", 1),
    ];
    let submission = AssignmentSubmissionService::find_one(&filters, None)
        .await
        .expect("DB error during submission lookup")
        .expect("No matching submission found");

    match create_submission_outputs_for_all_tasks(submission.id).await {
        Ok(_) => {}
        Err(e) => panic!(
            "Failed to generate submission outputs for C++ assignment: {}",
            e
        ),
    }
}
