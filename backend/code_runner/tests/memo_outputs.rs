use chrono::Utc;
use code_runner::create_memo_outputs_for_all_tasks;
use db::models::assignment::AssignmentType;
use db::models::assignment::{ActiveModel as AssignmentActiveModel, Entity as AssignmentEntity};
use db::models::assignment_task::Model as AssignmentTaskModel;
use db::models::module::{ActiveModel as ModuleActiveModel, Entity as ModuleEntity};
use db::test_utils::setup_test_db;
use sea_orm::DatabaseConnection;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};

async fn seed_module(db: &DatabaseConnection, module_id: i64, code: &str) {
    let existing_module = ModuleEntity::find_by_id(module_id)
        .one(db)
        .await
        .expect("DB error during module lookup");

    if existing_module.is_none() {
        let module = ModuleActiveModel {
            id: Set(module_id),
            code: Set(code.to_string()),
            year: Set(2025),
            description: Set(Some(format!("Test module for ID {}", module_id))),
            credits: Set(12),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
        };

        module
            .insert(db)
            .await
            .expect(&format!("Failed to insert module with id {}", module_id));
    }
}

async fn seed_assignment(db: &DatabaseConnection, assignment_id: i64, module_id: i64) {
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

        assignment.insert(db).await.expect(&format!(
            "Failed to insert assignment with id {}",
            assignment_id
        ));
    }
}

async fn seed_tasks(db: &DatabaseConnection, assignment_id: i64) {
    let mut tasks = vec![(1, "make task1"), (2, "make task2"), (3, "make task3")];

    if assignment_id == 9998 {
        tasks.push((4, "make task4"));
    }

    for (task_number, command) in tasks {
        AssignmentTaskModel::create(db, assignment_id, task_number, command)
            .await
            .expect("Failed to create assignment task");
    }
}

async fn setup_test_db_with_seeded_tasks(
    module_id: i64,
    assignment_id: i64,
    code: &str,
) -> DatabaseConnection {
    let db = setup_test_db().await;

    seed_module(&db, module_id, code).await;
    seed_assignment(&db, assignment_id, module_id).await;
    seed_tasks(&db, assignment_id).await;

    db
}

#[tokio::test]
#[ignore]
async fn test_create_memo_outputs_for_all_tasks_9999_java() {
    dotenv::dotenv().ok();

    let db = setup_test_db_with_seeded_tasks(9999, 9999, "COS999").await;

    let assignment_id = 9999;

    match create_memo_outputs_for_all_tasks(&db, assignment_id).await {
        Ok(_) => println!("Memo outputs generated successfully for all tasks (Java 9999)."),
        Err(e) => panic!("Failed to generate memo outputs: {}", e),
    }
}

#[tokio::test]
#[ignore]
async fn test_create_memo_outputs_for_all_tasks_9998_cpp() {
    dotenv::dotenv().ok();

    let db = setup_test_db_with_seeded_tasks(9998, 9998, "COS998").await;

    let assignment_id = 9998;

    match create_memo_outputs_for_all_tasks(&db, assignment_id).await {
        Ok(_) => println!("Memo outputs generated successfully for all tasks (C++ 9998)."),
        Err(e) => panic!("Failed to generate memo outputs: {}", e),
    }
}
