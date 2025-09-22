use chrono::Utc;
use code_runner::create_memo_outputs_for_all_tasks;
use services::assignment::{AssignmentService, AssignmentType, CreateAssignment};
use services::assignment_task::{AssignmentTaskService, CreateAssignmentTask};
use services::module::{CreateModule, ModuleService};
use services::service::Service;
use util::state::AppState;

async fn seed_module(module_id: i64, code: &str) {
    let existing_module = ModuleService::find_by_id(module_id)
        .await
        .expect("DB error during module lookup");

    if existing_module.is_none() {
        let _ = ModuleService::create(CreateModule {
            id: Some(module_id),
            code: code.to_string(),
            year: 2025,
            description: Some(format!("Test module for ID {}", module_id)),
            credits: 12,
        })
        .await
        .expect("Failed to insert module");
    }
}

async fn seed_assignment(assignment_id: i64, module_id: i64) {
    let existing_assignment = AssignmentService::find_by_id(assignment_id)
        .await
        .expect("DB error during assignment lookup");

    if existing_assignment.is_none() {
        let _ = AssignmentService::create(CreateAssignment {
            id: Some(assignment_id),
            module_id: module_id,
            name: "Special Assignment".to_string(),
            description: Some("Special assignment for testing".to_string()),
            assignment_type: AssignmentType::Assignment,
            available_from: Utc::now(),
            due_date: Utc::now(),
        })
        .await
        .expect("Failed to insert assignment");
    }
}

async fn seed_tasks(assignment_id: i64) {
    let mut tasks = vec![(1, "make task1"), (2, "make task2"), (3, "make task3")];

    if assignment_id == 9998 {
        tasks.push((4, "make task4"));
    }

    for (task_number, command) in tasks {
        let _ = AssignmentTaskService::create(CreateAssignmentTask {
            assignment_id,
            task_number,
            name: "Untitled Task".to_string(),
            command: command.to_string(),
            code_coverage: false,
        })
        .await
        .expect("Failed to create assignment task");
    }
}

async fn setup_test_db_with_seeded_tasks(module_id: i64, assignment_id: i64, code: &str) {
    let _ = AppState::init(true);
    seed_module(module_id, code).await;
    seed_assignment(assignment_id, module_id).await;
    seed_tasks(assignment_id).await;
}

#[tokio::test]
#[ignore]
async fn test_create_memo_outputs_for_all_tasks_9999_java() {
    dotenvy::dotenv().ok();

    setup_test_db_with_seeded_tasks(9999, 9999, "COS999").await;

    match create_memo_outputs_for_all_tasks(9999).await {
        Ok(_) => println!("Memo outputs generated successfully for all tasks (Java 9999)."),
        Err(e) => panic!("Failed to generate memo outputs: {}", e),
    }
}

#[tokio::test]
#[ignore]
async fn test_create_memo_outputs_for_all_tasks_9998_cpp() {
    dotenvy::dotenv().ok();

    setup_test_db_with_seeded_tasks(9998, 9998, "COS998").await;

    match create_memo_outputs_for_all_tasks(9998).await {
        Ok(_) => println!("Memo outputs generated successfully for all tasks (C++ 9998)."),
        Err(e) => panic!("Failed to generate memo outputs: {}", e),
    }
}
