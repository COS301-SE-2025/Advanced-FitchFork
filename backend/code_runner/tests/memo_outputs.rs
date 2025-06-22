use chrono::Utc;
use code_runner::create_memo_outputs_for_all_tasks;
use db::models::assignment::AssignmentType;
use db::models::assignment::{ActiveModel as AssignmentActiveModel, Entity as AssignmentEntity};
use db::models::assignment_task::Model as AssignmentTaskModel;
use db::models::module::{ActiveModel as ModuleActiveModel, Entity as ModuleEntity};
use db::test_utils::setup_test_db;
use sea_orm::DatabaseConnection;
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

pub async fn setup_test_db_with_seeded_tasks() -> DatabaseConnection {
    let db = setup_test_db().await;

    seed_module_assignment_and_tasks(&db).await;

    db
}

#[tokio::test]
#[ignore]
async fn test_create_memo_outputs_for_all_tasks_9999() {
    dotenv::dotenv().ok();

    let db = setup_test_db_with_seeded_tasks().await;

    let module_id = 9999;
    let assignment_id = 9999;

    match crate::create_memo_outputs_for_all_tasks(&db, module_id, assignment_id).await {
        Ok(_) => println!("Memo outputs generated successfully for all tasks."),
        Err(e) => panic!("Failed to generate memo outputs: {}", e),
    }
}
