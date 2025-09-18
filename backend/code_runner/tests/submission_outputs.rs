use chrono::Utc;
use code_runner::create_submission_outputs_for_all_tasks;
use db::models::assignment::AssignmentType;
use db::models::assignment::{ActiveModel as AssignmentActiveModel, Entity as AssignmentEntity};
use db::models::assignment_submission::{
    ActiveModel as SubmissionActiveModel, Model as SubmissionModel,
};
use db::models::assignment_task::Model as AssignmentTaskModel;
use db::models::module::{ActiveModel as ModuleActiveModel, Entity as ModuleEntity};
use db::models::user::{ActiveModel as UserActiveModel, Entity as UserEntity};
use db::test_utils::setup_test_db;
use sea_orm::DatabaseConnection;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use util::paths::{attempt_dir, storage_root};

async fn seed_user(db: &DatabaseConnection) -> i64 {
    let user_id = 1;
    if UserEntity::find_by_id(user_id)
        .one(db)
        .await
        .expect("DB error during user lookup")
        .is_none()
    {
        let user = UserActiveModel {
            id: Set(user_id), // explicitly set ID if your DB allows it
            username: Set("u00000001".to_string()),
            email: Set("testuser@example.com".to_string()),
            password_hash: Set("hashedpassword".to_string()),
            admin: Set(false),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            profile_picture_path: Set(None),
        };
        user.insert(db).await.expect("Failed to insert user");
    }
    user_id
}

async fn seed_submission(db: &DatabaseConnection, assignment_id: i64) -> SubmissionModel {
    let user_id = 1;
    let attempt = 1;
    let filename = "submission.zip";

    let assignment = AssignmentEntity::find_by_id(assignment_id)
        .one(db)
        .await
        .expect("Failed to lookup assignment")
        .expect("Assignment not found");

    let module_id = assignment.module_id;

    let submission_dir = attempt_dir(module_id, assignment_id, user_id, attempt);
    std::fs::create_dir_all(&submission_dir).expect("Failed to create attempt dir");

    let file_path = submission_dir.join("481.zip");

    let relative_path = file_path
        .strip_prefix(storage_root())
        .unwrap()
        .to_string_lossy()
        .to_string();

    let now = Utc::now();

    let submission = SubmissionActiveModel {
        assignment_id: Set(assignment_id),
        user_id: Set(user_id),
        attempt: Set(attempt),
        filename: Set(filename.to_string()),
        file_hash: Set("0".to_string()),
        path: Set(relative_path),
        is_practice: Set(false),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    submission
        .insert(db)
        .await
        .expect("Failed to insert submission")
}

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
        AssignmentTaskModel::create(
            db,
            assignment_id,
            task_number,
            "Untitled Task",
            command,
            false,
        )
        .await
        .expect("Failed to create assignment task");
    }
}

async fn setup_test_db_with_seeded_tasks(assignment_id: i64, module_id: i64) -> DatabaseConnection {
    let db = setup_test_db().await;

    seed_user(&db).await;
    seed_module(&db, module_id, &format!("COS{}", module_id)).await;
    seed_assignment(&db, assignment_id, module_id).await;
    seed_tasks(&db, assignment_id).await;
    seed_submission(&db, assignment_id).await;

    db
}

#[tokio::test]
#[ignore]
async fn test_create_submission_outputs_for_all_tasks_9999_java() {
    dotenvy::dotenv().ok();

    let db = setup_test_db_with_seeded_tasks(9999, 9999).await;

    use db::models::assignment_submission::Entity as SubmissionEntity;
    use sea_orm::ColumnTrait;
    use sea_orm::QueryFilter;

    let submission = SubmissionEntity::find()
        .filter(db::models::assignment_submission::Column::AssignmentId.eq(9999))
        .filter(db::models::assignment_submission::Column::UserId.eq(1))
        .filter(db::models::assignment_submission::Column::Attempt.eq(1))
        .one(&db)
        .await
        .expect("Failed to lookup submission")
        .expect("No matching submission found");

    let submission_id = submission.id;

    match create_submission_outputs_for_all_tasks(&db, submission_id).await {
        Ok(_) => {}
        Err(e) => panic!("Failed to generate submission outputs: {}", e),
    }
}

#[tokio::test]
#[ignore]
async fn test_create_submission_outputs_for_all_tasks_9998_cpp() {
    dotenvy::dotenv().ok();

    let db = setup_test_db_with_seeded_tasks(9998, 9998).await;

    use db::models::assignment_submission::Entity as SubmissionEntity;
    use sea_orm::ColumnTrait;
    use sea_orm::QueryFilter;

    let submission = SubmissionEntity::find()
        .filter(db::models::assignment_submission::Column::AssignmentId.eq(9998))
        .filter(db::models::assignment_submission::Column::UserId.eq(1))
        .filter(db::models::assignment_submission::Column::Attempt.eq(1))
        .one(&db)
        .await
        .expect("Failed to lookup submission")
        .expect("No matching submission found");

    let submission_id = submission.id;

    match create_submission_outputs_for_all_tasks(&db, submission_id).await {
        Ok(_) => {}
        Err(e) => panic!(
            "Failed to generate submission outputs for C++ assignment: {}",
            e
        ),
    }
}
