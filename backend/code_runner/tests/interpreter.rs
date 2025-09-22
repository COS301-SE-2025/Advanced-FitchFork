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
use util::paths::{
    interpreter_dir, makefile_dir, memo_dir, storage_root, submission_file_path
};
use util::execution_config::ExecutionConfig;
use util::test_helpers::setup_test_storage_root;


fn write_zip(path: &std::path::Path, entries: &[(&str, &[u8])]) -> std::io::Result<()> {
    use std::io::Write;
    use zip::write::SimpleFileOptions;
    let mut buf = Vec::new();
    {
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
        let opts = SimpleFileOptions::default();
        for (name, bytes) in entries {
            zip.start_file(*name, opts)?;
            zip.write_all(bytes)?;
        }
        zip.finish()?;
    }
    if let Some(parent) = path.parent() { std::fs::create_dir_all(parent)?; }
    std::fs::write(path, &buf)
}


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
) -> SubmissionModel {
    let user_id = seed_user(db).await;
    let attempt = 1;

    let assignment = AssignmentEntity::find_by_id(assignment_id)
        .one(db)
        .await
        .expect("Failed to lookup assignment")
        .expect("Assignment not found");

    let module_id = assignment.module_id;

    let now = Utc::now();

    // 1) Insert placeholder to get the submission ID
    let placeholder = SubmissionActiveModel {
        assignment_id: Set(assignment_id),
        user_id: Set(user_id),
        attempt: Set(attempt),
        // Keep the *original uploaded name* user would have provided.
        // Tests can use anything; keep it stable:
        filename: Set("submission.zip".to_string()),
        file_hash: Set("0".to_string()),
        path: Set(String::new()), // will be filled after writing the file
        is_practice: Set(false),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    let inserted: SubmissionModel = placeholder
        .insert(db)
        .await
        .expect("Failed to insert submission placeholder");

    // 2) Build the on-disk path using the REAL submission id and ".zip"
    let file_path = submission_file_path(
        module_id,
        assignment_id,
        user_id,
        attempt,
        inserted.id,
        Some("zip"),
    );

    // 3) Write a minimal valid zip so downstream reads succeed
    write_zip(&file_path, &[("README.txt", b"dummy submission")]).expect("write submission zip");

    // 4) Store the *relative* path in DB
    let rel = file_path
        .strip_prefix(storage_root())
        .expect("strip prefix")
        .to_string_lossy()
        .to_string();

    // 5) Update submission with resolved path (and updated_at)
    let mut update: SubmissionActiveModel = inserted.into();
    update.path = Set(rel);
    update.updated_at = Set(Utc::now());

    update.update(db).await.expect("Failed to update submission")
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

    let filename = format!("{}.zip", interpreter_id);

    // Place on disk using path utils:
    let interp_dir = interpreter_dir(module_id, assignment_id);
    let interp_path = interp_dir.join(&filename);


    // Minimal interpreter payload (content won't be used in compile stopgap case)
    write_zip(&interp_path, &[("interpreter.cpp", b"int main(){return 0;}")])
        .expect("write interpreter zip");

    // Relative DB path:
    let relative_path = interp_path
        .strip_prefix(storage_root())
        .expect("strip prefix")
        .to_string_lossy()
        .to_string();


    // The command; choose something that triggers the compile stopgap if desired
    let command = "g++ Main.cpp -o main && ./main".to_string();

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
        interpreter.insert(db).await.expect("Failed to insert interpreter");
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

    // IMPORTANT: make sure storage root is set and lives long enough
    // e.g., in your test use a TempDir and set the env var before calling this helper.

    seed_user(&db).await;
    seed_module(&db, module_id, &format!("COS{}", module_id)).await;
    seed_assignment(&db, assignment_id, module_id).await;
    seed_tasks(&db, assignment_id).await;

    // Write default execution config
    ExecutionConfig::default_config()
        .save(module_id, assignment_id)
        .expect("save config.json");

    // Satisfy validators: memo + makefile need a .zip present
    {
        let memo_zip = memo_dir(module_id, assignment_id).join("memo.zip");
        write_zip(&memo_zip, &[("memo.txt", b"memo")]).expect("write memo.zip");

        let make_zip = makefile_dir(module_id, assignment_id).join("makefile.zip");
        write_zip(&make_zip, &[("Makefile", b"all:\n\t@echo ok")]).expect("write makefile.zip");
    }

    let submission = seed_submission(&db, assignment_id).await;
    seed_interpreter_file(&db, assignment_id, interpreter_id).await;

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
    // Keep this TempDir alive for the whole test so files remain on disk
    let _tmp = setup_test_storage_root();

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

    // run_interpreter will:
    // 1) synthesize main zip (compile stopgap)
    // 2) validate memo/makefile/main (memo+makefile we seeded, main just created)
    // 3) run tasks and save outputs
    match run_interpreter(&db, submission_id, gene_string).await {
        Ok(_) => println!("run_interpreter completed successfully for assignment 9998."),
        Err(e) => panic!("run_interpreter failed: {}", e),
    }

    // keep `tmp` in scope until here
}

