use chrono::Utc;
use code_runner::run_interpreter;
use db::models::assignment::AssignmentType;
use db::models::assignment::{ActiveModel as AssignmentActiveModel, Entity as AssignmentEntity};
use db::models::assignment_submission::{
    ActiveModel as SubmissionActiveModel, Model as SubmissionModel,
};
use db::models::assignment_task::Model as AssignmentTaskModel;
use db::models::module::{ActiveModel as ModuleActiveModel, Entity as ModuleEntity};
use db::models::user::{ActiveModel as UserActiveModel, Entity as UserEntity};
use db::test_utils::setup_test_db;
use sea_orm::ColumnTrait;
use sea_orm::QueryFilter;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};

async fn seed_user(db: &DatabaseConnection) -> i64 {
    let user_id = 1;
    if UserEntity::find_by_id(user_id)
        .one(db)
        .await
        .expect("DB error during user lookup")
        .is_none()
    {
        let user = UserActiveModel {
            id: Set(user_id),
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

async fn seed_module(db: &DatabaseConnection, module_id: i64, code: &str) {
    if ModuleEntity::find_by_id(module_id)
        .one(db)
        .await
        .expect("DB error")
        .is_none()
    {
        let module = ModuleActiveModel {
            id: Set(module_id),
            code: Set(code.to_string()),
            year: Set(2025),
            description: Set(Some(format!("Test module for ID {}", module_id))),
            credits: Set(12),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
        };
        module.insert(db).await.expect("Failed to insert module");
    }
}

async fn seed_assignment(db: &DatabaseConnection, assignment_id: i64, module_id: i64) {
    if AssignmentEntity::find_by_id(assignment_id)
        .one(db)
        .await
        .expect("DB error")
        .is_none()
    {
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
            .expect("Failed to insert assignment");
    }
}

async fn seed_tasks(db: &DatabaseConnection, assignment_id: i64) {
    let mut tasks = vec![(1, "make task1"), (2, "make task2"), (3, "make task3")];
    if assignment_id == 9998 {
        tasks.push((4, "make task4"));
    }
    for (task_number, command) in tasks {
        AssignmentTaskModel::create(db, assignment_id, task_number, "Untitled Task", command)
            .await
            .expect("Failed to create assignment task");
    }
}

async fn seed_submission(
    db: &DatabaseConnection,
    assignment_id: i64,
    assignment_submission_id: i64,
) -> SubmissionModel {
    let user_id = seed_user(db).await;
    let attempt = 1;
    let filename = "submission.zip";

    let assignment = AssignmentEntity::find_by_id(assignment_id)
        .one(db)
        .await
        .expect("Failed to lookup assignment")
        .expect("Assignment not found");

    let module_id = assignment.module_id;

    let submission_dir =
        SubmissionModel::full_directory_path(module_id, assignment_id, user_id, attempt);
    let file_path = submission_dir.join(format!("{}.zip", assignment_submission_id));

    let relative_path = file_path
        .strip_prefix(SubmissionModel::storage_root())
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

async fn seed_interpreter_file(db: &DatabaseConnection, assignment_id: i64, interpreter_id: i64) {
    use db::models::assignment_interpreter::{
        ActiveModel as InterpreterActiveModel, Column as InterpreterColumn,
        Entity as InterpreterEntity,
    };

    let now = Utc::now();

    let assignment = AssignmentEntity::find_by_id(assignment_id)
        .one(db)
        .await
        .expect("Failed to lookup assignment")
        .expect("Assignment not found");

    let module_id = assignment.module_id;

    // The filename on disk is "{interpreter_id}.zip"
    let filename = format!("{}.zip", interpreter_id);

    // Path relative to ASSIGNMENT_STORAGE_ROOT, matching your model's logic:
    let relative_path = format!(
        "module_{}/assignment_{}/interpreter/{}",
        module_id, assignment_id, filename
    );

    // The command to run the interpreter - put your actual interpreter command here
    let command = "g++ /code/interpreter.cpp -o /code/interpreter_exe &&
 /code/interpreter_exe"
        .to_string();

    // Check if an interpreter for this assignment already exists
    if InterpreterEntity::find()
        .filter(InterpreterColumn::AssignmentId.eq(assignment_id))
        .one(db)
        .await
        .expect("DB error")
        .is_none()
    {
        let interpreter = InterpreterActiveModel {
            assignment_id: Set(assignment_id),
            filename: Set(filename),
            path: Set(relative_path),
            command: Set(command),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };
        interpreter
            .insert(db)
            .await
            .expect("Failed to insert interpreter");
    }
}

async fn setup_test_db_for_run_interpreter(
    assignment_id: i64,
    module_id: i64,
    assignment_submission_id: i64,
    interpreter_id: i64,
) -> (DatabaseConnection, i64) {
    let db = setup_test_db().await;

    seed_user(&db).await;
    seed_module(&db, module_id, &format!("COS{}", module_id)).await;
    seed_assignment(&db, assignment_id, module_id).await;
    seed_tasks(&db, assignment_id).await;
    let submission = seed_submission(&db, assignment_id, assignment_submission_id).await;
    seed_interpreter_file(&db, assignment_id, interpreter_id).await;

    (db, submission.id)
}

#[tokio::test]
#[ignore]
async fn test_run_interpreter_9998_cpp() {
    dotenvy::dotenv().ok();

    let assignment_id = 9998;
    let module_id = 9998;
    let assignment_submission_id = 182;
    let interpreter_id = 19;

    let gene_string = "01234";

    let (db, submission_id) = setup_test_db_for_run_interpreter(
        assignment_id,
        module_id,
        assignment_submission_id,
        interpreter_id,
    )
    .await;

    match run_interpreter(&db, submission_id, &gene_string).await {
        Ok(_) => println!("run_interpreter completed successfully for assignment 9998."),
        Err(e) => panic!("run_interpreter failed: {}", e),
    }
}
