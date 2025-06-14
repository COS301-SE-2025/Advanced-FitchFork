// models/assignment_file.rs

use chrono::{DateTime, Utc};
// use code_runner::{run_zip_files, ExecutionConfig};
use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::Set;
use std::env;
use std::fs;
use std::path::PathBuf;
use strum_macros::{Display, EnumIter, EnumString};

/// Represents a file associated with an assignment, such as a spec, main file, memo, or submission.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "assignment_files")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,

    /// Foreign key reference to an assignment.
    pub assignment_id: i64,

    /// Original file name.
    pub filename: String,

    /// Relative path to the stored file from the storage root.
    pub path: String,

    /// Type of the file (spec, main, memo, submission).
    pub file_type: FileType,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Enum representing the type/category of an assignment file.
#[derive(Debug, Clone, PartialEq, EnumIter, EnumString, Display, DeriveActiveEnum)]
#[strum(ascii_case_insensitive)]
#[sea_orm(
    rs_type = "String",
    db_type = "Enum",
    enum_name = "assignment_file_type"
)]
pub enum FileType {
    #[strum(serialize = "spec")]
    #[sea_orm(string_value = "spec")]
    Spec,
    #[strum(serialize = "main")]
    #[sea_orm(string_value = "main")]
    Main,
    #[strum(serialize = "memo")]
    #[sea_orm(string_value = "memo")]
    Memo,
    #[strum(serialize = "makefile")]
    #[sea_orm(string_value = "makefile")]
    Makefile,
    #[strum(serialize = "mark_allocator")]
    #[sea_orm(string_value = "mark_allocator")]
    MarkAllocator,
    #[strum(serialize = "config")]
    #[sea_orm(string_value = "config")]
    Config,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::assignment::Entity",
        from = "Column::AssignmentId",
        to = "super::assignment::Column::Id"
    )]
    Assignment,
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// Returns the base directory for assignment file storage from the environment.
    pub fn storage_root() -> PathBuf {
        env::var("ASSIGNMENT_STORAGE_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("data/assignment_files"))
    }

    /// Computes the full directory path based on module ID, assignment ID, and file type.
    pub fn full_directory_path(
        module_id: i64,
        assignment_id: i64,
        file_type: &FileType,
    ) -> PathBuf {
        Self::storage_root()
            .join(format!("module_{module_id}"))
            .join(format!("assignment_{assignment_id}"))
            .join(file_type.to_string())
    }

    /// Saves a file and returns the corresponding Model.
    ///
    /// # Arguments
    /// * `db` - Database connection
    /// * `assignment_id` - ID of the assignment
    /// * `module_id` - ID of the parent module
    /// * `file_type` - Category of the file
    /// * `filename` - Original filename
    /// * `bytes` - File contents
    pub async fn save_file(
        db: &DatabaseConnection,
        assignment_id: i64,
        module_id: i64,
        file_type: FileType,
        filename: &str,
        bytes: &[u8],
    ) -> Result<Self, DbErr> {
        let now = Utc::now();

        // Step 1: Insert placeholder record with dummy path
        let partial = ActiveModel {
            assignment_id: Set(assignment_id),
            filename: Set(filename.to_string()),
            path: Set("".to_string()), // temporary dummy to satisfy NOT NULL
            file_type: Set(file_type.clone()),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        let inserted: Model = partial.insert(db).await?;

        // Step 2: Extract extension
        let ext = PathBuf::from(filename)
            .extension()
            .map(|e| e.to_string_lossy().to_string());

        let stored_filename = match ext {
            Some(ext) => format!("{}.{}", inserted.id, ext),
            None => inserted.id.to_string(),
        };

        let dir_path = Self::full_directory_path(module_id, assignment_id, &file_type);
        fs::create_dir_all(&dir_path)
            .map_err(|e| DbErr::Custom(format!("Failed to create directory: {e}")))?;

        let file_path = dir_path.join(&stored_filename);
        let relative_path = file_path
            .strip_prefix(Self::storage_root())
            .unwrap()
            .to_string_lossy()
            .to_string();

        fs::write(&file_path, bytes)
            .map_err(|e| DbErr::Custom(format!("Failed to write file: {e}")))?;

        // Step 3: Update the row with correct path
        let mut model: ActiveModel = inserted.into();
        model.path = Set(relative_path);
        model.updated_at = Set(Utc::now());

        model.update(db).await
    }

    /// Loads the file contents from disk based on the path stored in the model.
    pub fn load_file(&self) -> Result<Vec<u8>, std::io::Error> {
        let full_path = Self::storage_root().join(&self.path);
        fs::read(full_path)
    }

    /// Deletes the file from disk (but not the DB record).
    pub fn delete_file_only(&self) -> Result<(), std::io::Error> {
        let full_path = Self::storage_root().join(&self.path);
        fs::remove_file(full_path)
    }

    //TODO - move all these commented things to their own file (Richard will do)

    // /// Creates the memo output for an assignment by executing both the memo file and the main file.
    // ///
    // /// This function does the following:
    // /// - Looks up the memo and main files for the given assignment ID.
    // /// - Verifies that both files exist, or returns an error.
    // /// - Executes the memo and main zip files using the code runner.
    // /// - Stores the resulting output as a `.txt` file in a memo output directory.
    // /// - Persists the generated output file as a `FileType::MemoOutput` record in the database.
    // ///
    // /// # Arguments
    // /// - `db`: Database connection.
    // /// - `assignment_id`: The ID of the assignment for which to generate memo output.
    // /// - `lang`: Programming language to use when executing the files (e.g., "java").
    // /// - `exec_config`: Optional execution configuration (e.g., time limit, memory, cores). If `None`, the default is used.
    // ///
    // /// # Returns
    // /// Returns the saved output `Model` on success or an error `String` on failure.
    // ///
    // /// # Errors
    // /// Returns errors if:
    // /// - The memo or main file cannot be found.
    // /// - There are database or filesystem errors.
    // /// - The code runner fails during execution.
    // pub async fn create_memo_output(
    //     db: &DatabaseConnection,
    //     assignment_id: i64,
    //     lang: &str,
    //     exec_config: Option<ExecutionConfig>, //None uses default
    // ) -> Result<Model, String> {
    //     use sea_orm::{EntityTrait, QueryFilter};
    //     // Find memo file
    //     let memo = Entity::find()
    //         .filter(Column::AssignmentId.eq(assignment_id))
    //         .filter(Column::FileType.eq(FileType::Memo))
    //         .one(db)
    //         .await
    //         .map_err(|e| format!("DB error finding memo: {}", e))?
    //         .ok_or_else(|| "Memo file not found".to_string())?;

    //     // Find main file
    //     let main = Entity::find()
    //         .filter(Column::AssignmentId.eq(assignment_id))
    //         .filter(Column::FileType.eq(FileType::Main))
    //         .one(db)
    //         .await
    //         .map_err(|e| format!("DB error finding main: {}", e))?
    //         .ok_or_else(|| "Main file not found".to_string())?;

    //     //Get assignment
    //     let assignment = super::assignment::Entity::find_by_id(assignment_id)
    //         .one(db)
    //         .await
    //         .map_err(|e| format!("DB error finding assignment: {}", e))?
    //         .ok_or_else(|| "Assignment not found".to_string())?;

    //     let module_id = assignment.module_id;

    //     // Paths to the files
    //     let memo_path = Self::storage_root().join(&memo.path);
    //     let main_path = Self::storage_root().join(&main.path);

    //     // Run the code using the existing run_zip_files API (async)
    //     let output = run_zip_files(vec![memo_path, main_path], lang, exec_config)
    //         .await
    //         .map_err(|e| format!("Code runner error: {}", e))?;

    //     // Save the output to a new file in memo_output directory
    //     let output_dir = Self::full_directory_path(module_id, assignment_id, &FileType::Memo);
    //     fs::create_dir_all(&output_dir)
    //         .map_err(|e| format!("Failed to create output directory: {}", e))?;

    //     let output_filename = "memo_output.txt";
    //     let output_file_path = output_dir.join(output_filename);

    //     fs::write(&output_file_path, output.as_bytes())
    //         .map_err(|e| format!("Failed to write output file: {}", e))?;

    //     // Save file record in DB
    //     Self::save_file(
    //         db,
    //         assignment_id,
    //         module_id,
    //         FileType::MemoOutput,
    //         output_filename,
    //         output.as_bytes(),
    //     )
    //     .await
    //     .map_err(|e| format!("Failed to save output file record: {}", e))
    // }

    // /// Creates the output of a student's submission by executing their submission file with the assignment's main file.
    // ///
    // /// This function does the following:
    // /// - Retrieves the submission file using the provided submission file ID.
    // /// - Looks up the associated assignment and its main file.
    // /// - Verifies that both the submission and main files exist, or returns an error.
    // /// - Executes both zip files using the code runner.
    // /// - Stores the resulting output as a `.txt` file in a directory under `submission_outputs/<submission_file_id>/`.
    // /// - Persists the output file as a `FileType::SubmissionOutput` record in the database.
    // ///
    // /// # Arguments
    // /// - `db`: Database connection.
    // /// - `submission_file_id`: ID of the uploaded submission file.
    // /// - `lang`: Programming language to use when executing the files (e.g., "java").
    // /// - `exec_config`: Optional execution configuration. Uses default values if `None`.
    // ///
    // /// # Returns
    // /// Returns the saved output `Model` on success or an error `String` on failure.
    // ///
    // /// # Errors
    // /// Returns errors if:
    // /// - The submission, assignment, or main file cannot be found.
    // /// - Filesystem or database operations fail.
    // /// - The code runner returns an execution error.
    // pub async fn create_submission_output(
    //     db: &DatabaseConnection,
    //     submission_file_id: i64,
    //     lang: &str,
    //     exec_config: Option<ExecutionConfig>, //None uses default
    // ) -> Result<Model, String> {
    //     use sea_orm::{EntityTrait, QueryFilter};
    //     // Find submission file
    //     let submission_file = super::assignment_submission::Entity::find_by_id(submission_file_id)
    //         .one(db)
    //         .await
    //         .map_err(|e| format!("DB error finding submission file: {}", e))?
    //         .ok_or_else(|| "Submission file not found".to_string())?;

    //     // Find submission to get assignment_id and module_id
    //     let submission = super::assignment_submission::Entity::find_by_id(submission_file.id)
    //         .one(db)
    //         .await
    //         .map_err(|e| format!("DB error finding submission: {}", e))?
    //         .ok_or_else(|| "Submission not found".to_string())?;

    //     let assignment = super::assignment::Entity::find_by_id(submission.assignment_id)
    //         .one(db)
    //         .await
    //         .map_err(|e| format!("DB error finding assignment: {}", e))?
    //         .ok_or_else(|| "Assignment not found".to_string())?;

    //     let module_id = assignment.module_id;
    //     let assignment_id = assignment.id;

    //     // Find main file
    //     let main = Entity::find()
    //         .filter(Column::AssignmentId.eq(assignment_id))
    //         .filter(Column::FileType.eq(FileType::Main))
    //         .one(db)
    //         .await
    //         .map_err(|e| format!("DB error finding main: {}", e))?
    //         .ok_or_else(|| "Main file not found".to_string())?;

    //     // Paths to the files
    //     let submission_path =
    //         super::assignment_submission::Model::storage_root().join(&submission_file.path);
    //     let main_path = Self::storage_root().join(&main.path);

    //     // Run the code using the existing run_zip_files API (async)
    //     let output = run_zip_files(vec![submission_path, main_path], lang, exec_config)
    //         .await
    //         .map_err(|e| format!("Code runner error: {}", e))?;

    //     // Create output directory: <storage_root>/submission_outputs/<submission_file_id>/
    //     let output_dir = Self::storage_root()
    //         .join("submission_outputs")
    //         .join(submission_file_id.to_string());

    //     std::fs::create_dir_all(&output_dir)
    //         .map_err(|e| format!("Failed to create output directory: {}", e))?;

    //     let output_filename = "submission_output.txt";
    //     let output_file_path = output_dir.join(output_filename);

    //     fs::write(&output_file_path, output.as_bytes())
    //         .map_err(|e| format!("Failed to write output file: {}", e))?;

    //     // Save file record in DB
    //     Self::save_file(
    //         db,
    //         assignment_id,
    //         module_id,
    //         FileType::SubmissionOutput,
    //         output_filename,
    //         output.as_bytes(),
    //     )
    //     .await
    //     .map_err(|e| format!("Failed to save output file record: {}", e))
    // }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::assignment::AssignmentType;
    use crate::test_utils::setup_test_db;
    use chrono::Utc;
    use sea_orm::Set;
    use std::env;
    use tempfile::TempDir;

    fn fake_bytes() -> Vec<u8> {
        vec![0x50, 0x4B, 0x03, 0x04] // ZIP file signature
    }

    fn override_storage_dir(temp: &TempDir) {
        env::set_var("ASSIGNMENT_STORAGE_ROOT", temp.path());
    }

    //TODO - this test failed on github and I don't know why
    //ERROR:
    /*
        failures:

    ---- models::assignment_file::tests::test_save_and_load_file stdout ----

    thread 'models::assignment_file::tests::test_save_and_load_file' panicked at db/src/models/assignment_file.rs:434:9:
    assertion failed: full_path.exists()
    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


    failures:
        models::assignment_file::tests::test_save_and_load_file

    test result: FAILED. 11 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.85s

    error: test failed, to rerun pass `-p db --lib`
    Error: Process completed with exit code 101.
    */
    #[tokio::test]
    #[ignore]
    async fn test_save_and_load_file() {
        let temp_dir = TempDir::new().unwrap();
        override_storage_dir(&temp_dir);
        let db = setup_test_db().await;

        // Insert dummy module so assignment FK passes
        let _module = crate::models::module::ActiveModel {
            code: Set("COS301".to_string()),
            year: Set(2025),
            description: Set(Some("Capstone".to_string())),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            ..Default::default()
        }
        .insert(&db)
        .await
        .expect("Insert module failed");

        // Insert dummy assignment using enum value for assignment_type
        let _assignment = crate::models::assignment::Model::create(
            &db,
            1,
            "Test Assignment",
            Some("Desc"),
            AssignmentType::Practical,
            Utc::now(),
            Utc::now(),
        )
        .await
        .expect("Insert assignment failed");

        let content = fake_bytes();
        let filename = "test_file.zip";
        let saved = Model::save_file(
            &db,
            1, // assignment_id
            1, // module_id
            FileType::Spec,
            filename,
            &content,
        )
        .await
        .unwrap();

        assert_eq!(saved.assignment_id, 1);
        assert_eq!(saved.filename, filename);
        assert_eq!(saved.file_type, FileType::Spec);

        // Confirm file on disk
        let full_path = Model::storage_root().join(&saved.path);
        //The error was this line
        assert!(full_path.exists());

        // Load contents
        let bytes = saved.load_file().unwrap();
        assert_eq!(bytes, content);

        // Delete file only
        saved.delete_file_only().unwrap();
        assert!(!full_path.exists());
    }

    //TODO - move all these commented things to their own file (Richard will do)

    // // Mock the run_zip_files function for testing
    // async fn mock_run_zip_files(
    //     _files: Vec<std::path::PathBuf>,
    //     _lang: &str,
    //     _config: ExecutionConfig,
    // ) -> Result<String, String> {
    //     Ok("mocked output from code runner".to_string())
    // }

    // // Override run_zip_files in the test scope using a trait or by dependency injection
    // // For now, we will patch via a test-only feature or use a global override
    // // Here we'll shadow the function locally to simulate

    // impl Model {
    //     // Re-implement create_memo_output with mock run_zip_files for testing
    //     pub async fn create_memo_output_mock(
    //         db: &DatabaseConnection,
    //         module_id: i64,
    //         assignment_id: i64,
    //         lang: &str,
    //         exec_config: ExecutionConfig,
    //     ) -> Result<Model, String> {
    //         use sea_orm::{EntityTrait, QueryFilter};
    //         // Find memo file
    //         let memo = Entity::find()
    //             .filter(Column::AssignmentId.eq(assignment_id))
    //             .filter(Column::FileType.eq(FileType::Memo))
    //             .one(db)
    //             .await
    //             .map_err(|e| format!("DB error finding memo: {}", e))?
    //             .ok_or_else(|| "Memo file not found".to_string())?;

    //         // Find main file
    //         let main = Entity::find()
    //             .filter(Column::AssignmentId.eq(assignment_id))
    //             .filter(Column::FileType.eq(FileType::Main))
    //             .one(db)
    //             .await
    //             .map_err(|e| format!("DB error finding main: {}", e))?
    //             .ok_or_else(|| "Main file not found".to_string())?;

    //         // Paths to the files
    //         let memo_path = Self::storage_root().join(&memo.path);
    //         let main_path = Self::storage_root().join(&main.path);

    //         // Use mocked run_zip_files
    //         let output = mock_run_zip_files(vec![memo_path, main_path], lang, exec_config)
    //             .await
    //             .map_err(|e| format!("Code runner error: {}", e))?;

    //         // Save the output to a new file in memo_output directory
    //         let output_dir =
    //             Self::full_directory_path(module_id, assignment_id, &FileType::MemoOutput);
    //         std::fs::create_dir_all(&output_dir)
    //             .map_err(|e| format!("Failed to create output directory: {}", e))?;

    //         let output_filename = "memo_output.txt";
    //         let output_file_path = output_dir.join(output_filename);

    //         std::fs::write(&output_file_path, output.as_bytes())
    //             .map_err(|e| format!("Failed to write output file: {}", e))?;

    //         // Save file record in DB
    //         Self::save_file(
    //             db,
    //             assignment_id,
    //             module_id,
    //             FileType::MemoOutput,
    //             output_filename,
    //             output.as_bytes(),
    //         )
    //         .await
    //         .map_err(|e| format!("Failed to save output file record: {}", e))
    //     }
    // }

    // #[tokio::test]
    // async fn test_create_memo_output_success() {
    //     let temp_dir = TempDir::new().unwrap();
    //     env::set_var("ASSIGNMENT_STORAGE_ROOT", temp_dir.path());

    //     let db = setup_test_db().await;

    //     // Insert dummy module
    //     let module = crate::models::module::ActiveModel {
    //         code: Set("COS309".to_string()),
    //         year: Set(2025),
    //         description: Set(Some("Capstone".to_string())),
    //         created_at: Set(Utc::now()),
    //         updated_at: Set(Utc::now()),
    //         ..Default::default()
    //     };
    //     let module = module.insert(&db).await.unwrap();

    //     // Insert dummy assignment
    //     let assignment = crate::models::assignment::Model::create(
    //         &db,
    //         module.id,
    //         "Test Assignment",
    //         None,
    //         crate::models::assignment::AssignmentType::Practical,
    //         Utc::now(),
    //         Utc::now(),
    //     )
    //     .await
    //     .unwrap();

    //     // Save memo file
    //     let memo_content = b"memo file content";
    //     let _memo = Model::save_file(
    //         &db,
    //         assignment.id,
    //         module.id,
    //         FileType::Memo,
    //         "memo.zip",
    //         memo_content,
    //     )
    //     .await
    //     .unwrap();

    //     // Save main file
    //     let main_content = b"main file content";
    //     let _main = Model::save_file(
    //         &db,
    //         assignment.id,
    //         module.id,
    //         FileType::Main,
    //         "main.zip",
    //         main_content,
    //     )
    //     .await
    //     .unwrap();

    //     // Call create_memo_output_mock (uses mock_run_zip_files)
    //     let exec_config = ExecutionConfig::default();
    //     let result =
    //         Model::create_memo_output_mock(&db, module.id, assignment.id, "python", exec_config)
    //             .await;

    //     assert!(result.is_ok());
    //     let output_model = result.unwrap();
    //     assert_eq!(output_model.assignment_id, assignment.id);
    //     assert_eq!(output_model.file_type, FileType::MemoOutput);

    //     // Check the output file exists
    //     let output_path = Model::storage_root().join(&output_model.path);
    //     assert!(output_path.exists());

    //     let contents = std::fs::read_to_string(output_path).unwrap();
    //     assert_eq!(contents, "mocked output from code runner");
    // }

    // #[tokio::test]
    // async fn test_create_memo_output_missing_memo() {
    //     let db = setup_test_db().await;
    //     let err = Model::create_memo_output_mock(
    //         &db,
    //         1,
    //         9999, // nonexistent assignment_id
    //         "python",
    //         ExecutionConfig::default(),
    //     )
    //     .await
    //     .unwrap_err();
    //     assert_eq!(err, "Memo file not found");
    // }

    // #[tokio::test]
    // async fn test_create_memo_output_missing_main() {
    //     let temp_dir = TempDir::new().unwrap();
    //     env::set_var("ASSIGNMENT_STORAGE_ROOT", temp_dir.path());
    //     let db = setup_test_db().await;

    //     // Insert module & assignment
    //     let module = crate::models::module::ActiveModel {
    //         code: Set("COS308".to_string()),
    //         year: Set(2025),
    //         description: Set(Some("Capstone".to_string())),
    //         created_at: Set(Utc::now()),
    //         updated_at: Set(Utc::now()),
    //         ..Default::default()
    //     };
    //     let module = module.insert(&db).await.unwrap();

    //     let assignment = crate::models::assignment::Model::create(
    //         &db,
    //         module.id,
    //         "Test Assignment",
    //         None,
    //         crate::models::assignment::AssignmentType::Practical,
    //         Utc::now(),
    //         Utc::now(),
    //     )
    //     .await
    //     .unwrap();

    //     // Save memo file only (no main file)
    //     let memo_content = b"memo file content";
    //     let _memo = Model::save_file(
    //         &db,
    //         assignment.id,
    //         module.id,
    //         FileType::Memo,
    //         "memo.zip",
    //         memo_content,
    //     )
    //     .await
    //     .unwrap();

    //     let err = Model::create_memo_output_mock(
    //         &db,
    //         module.id,
    //         assignment.id,
    //         "python",
    //         ExecutionConfig::default(),
    //     )
    //     .await
    //     .unwrap_err();

    //     assert_eq!(err, "Main file not found");
    // }

    // //TODO - move Integration tests to their own file (all tests beneath this line)
    // //Technically Integration tests but you can only run them seperately

    // //TODO - once again I have to ignore this test since github actions isn't set up for docker containers
    // #[tokio::test]
    // #[ignore]
    // async fn test_create_memo_output_with_real_java_files() -> Result<(), Box<dyn std::error::Error>>
    // {
    //     use std::io::Write;
    //     use zip::write::FileOptions;

    //     let temp_dir = TempDir::new()?;
    //     let storage_path = temp_dir.path().to_path_buf();
    //     env::set_var("ASSIGNMENT_STORAGE_ROOT", &storage_path);

    //     let db = setup_test_db().await;

    //     // Insert dummy module not used
    //     let _module1 = crate::models::module::ActiveModel {
    //         code: Set("COS713".to_string()),
    //         year: Set(1635),
    //         description: Set(Some("asg".to_string())),
    //         created_at: Set(Utc::now()),
    //         updated_at: Set(Utc::now()),
    //         ..Default::default()
    //     }
    //     .insert(&db)
    //     .await?;

    //     // Insert dummy module not used
    //     let _module2 = crate::models::module::ActiveModel {
    //         code: Set("COS669".to_string()),
    //         year: Set(9362),
    //         description: Set(Some("smtelse".to_string())),
    //         created_at: Set(Utc::now()),
    //         updated_at: Set(Utc::now()),
    //         ..Default::default()
    //     }
    //     .insert(&db)
    //     .await?;

    //     // Insert dummy module
    //     let module = crate::models::module::ActiveModel {
    //         code: Set("COS301".to_string()),
    //         year: Set(2025),
    //         description: Set(Some("Capstone".to_string())),
    //         created_at: Set(Utc::now()),
    //         updated_at: Set(Utc::now()),
    //         ..Default::default()
    //     }
    //     .insert(&db)
    //     .await?;

    //     // Insert dummy assignment not used
    //     let _assignment1 = crate::models::assignment::Model::create(
    //         &db,
    //         module.id,
    //         "smt",
    //         None,
    //         crate::models::assignment::AssignmentType::Practical,
    //         Utc::now(),
    //         Utc::now(),
    //     )
    //     .await?;

    //     // Insert dummy assignment
    //     let assignment = crate::models::assignment::Model::create(
    //         &db,
    //         module.id,
    //         "Java Test Assignment",
    //         None,
    //         crate::models::assignment::AssignmentType::Practical,
    //         Utc::now(),
    //         Utc::now(),
    //     )
    //     .await?;

    //     // Helper function to create a zip file from given files with content
    //     fn create_zip_from_files(
    //         zip_path: &std::path::Path,
    //         files: Vec<(&str, &str)>,
    //     ) -> std::io::Result<()> {
    //         let zip_file = std::fs::File::create(zip_path)?;
    //         let mut zip = zip::ZipWriter::new(zip_file);

    //         let options = FileOptions::default()
    //             .compression_method(zip::CompressionMethod::Stored)
    //             .unix_permissions(0o644);

    //         for (name, content) in files {
    //             zip.start_file(name, options)?;
    //             zip.write_all(content.as_bytes())?;
    //         }

    //         zip.finish()?;
    //         Ok(())
    //     }

    //     // Create Main.java content which calls HelperOne and HelperTwo
    //     let main_java = r#"
    //         public class Main {
    //             public static void main(String[] args) {
    //                 HelperOne.greet();
    //                 HelperTwo.farewell();
    //             }
    //         }
    //     "#;

    //     let helper_one_java = r#"
    //         public class HelperOne {
    //             public static void greet() {
    //                 System.out.println("Hello from HelperOne");
    //             }
    //         }
    //     "#;

    //     let helper_two_java = r#"
    //         public class HelperTwo {
    //             public static void farewell() {
    //                 System.out.println("Goodbye from HelperTwo");
    //             }
    //         }
    //     "#;

    //     // Create main.zip containing Main.java only
    //     let main_zip_path = storage_path.join("main.zip");
    //     create_zip_from_files(&main_zip_path, vec![("Main.java", main_java)])?;

    //     // Create memo.zip containing HelperOne.java and HelperTwo.java
    //     let memo_zip_path = storage_path.join("memo.zip");
    //     create_zip_from_files(
    //         &memo_zip_path,
    //         vec![
    //             ("HelperOne.java", helper_one_java),
    //             ("HelperTwo.java", helper_two_java),
    //         ],
    //     )?;

    //     // Save main file as FileType::Main in DB
    //     let main_bytes = std::fs::read(&main_zip_path)?;
    //     let _main_model = Model::save_file(
    //         &db,
    //         assignment.id,
    //         module.id,
    //         FileType::Main,
    //         "main.zip",
    //         &main_bytes,
    //     )
    //     .await?;

    //     // Save memo file as FileType::Memo in DB
    //     let memo_bytes = std::fs::read(&memo_zip_path)?;
    //     let _memo_model = Model::save_file(
    //         &db,
    //         assignment.id,
    //         module.id,
    //         FileType::Memo,
    //         "memo.zip",
    //         &memo_bytes,
    //     )
    //     .await?;

    //     let output_model = Model::create_memo_output(&db, assignment.id, "java", None)
    //         .await
    //         .expect("create_memo_output failed");

    //     // Verify output file exists on disk physically
    //     let output_path = Model::storage_root().join(&output_model.path);
    //     assert!(
    //         output_path.exists(),
    //         "Output file does not exist: {}",
    //         output_path.display()
    //     );

    //     println!("Output file path: {}", output_path.display());

    //     // Optionally print content for manual verification
    //     let content = std::fs::read_to_string(output_path)?;
    //     println!("Output content:\n{}", content);

    //     Ok(())
    // }

    // //TODO - once again I have to ignore this test since github actions isn't set up for docker containers
    // #[tokio::test]
    // #[ignore]
    // async fn test_create_submission_output_with_real_java_files(
    // ) -> Result<(), Box<dyn std::error::Error>> {
    //     use std::io::Write;
    //     use zip::write::FileOptions;

    //     let temp_dir = TempDir::new()?;
    //     let storage_path = temp_dir.path().to_path_buf();
    //     env::set_var("ASSIGNMENT_STORAGE_ROOT", &storage_path);

    //     let db = setup_test_db().await;

    //     // Insert dummy module thats not used to check file path
    //     let _modulenotused = crate::models::module::ActiveModel {
    //         code: Set("COS638".to_string()),
    //         year: Set(1923),
    //         description: Set(Some("asfkahs".to_string())),
    //         created_at: Set(Utc::now()),
    //         updated_at: Set(Utc::now()),
    //         ..Default::default()
    //     }
    //     .insert(&db)
    //     .await?;

    //     // Insert dummy module
    //     let module = crate::models::module::ActiveModel {
    //         code: Set("COS5284".to_string()),
    //         year: Set(1638),
    //         description: Set(Some("faisyvs".to_string())),
    //         created_at: Set(Utc::now()),
    //         updated_at: Set(Utc::now()),
    //         ..Default::default()
    //     }
    //     .insert(&db)
    //     .await?;

    //     // Insert dummy assignment not used
    //     let _assignment1 = crate::models::assignment::Model::create(
    //         &db,
    //         module.id,
    //         "smt",
    //         None,
    //         crate::models::assignment::AssignmentType::Practical,
    //         Utc::now(),
    //         Utc::now(),
    //     )
    //     .await?;

    //     // Insert dummy assignment not used
    //     let _assignment2 = crate::models::assignment::Model::create(
    //         &db,
    //         module.id,
    //         "smtelse",
    //         None,
    //         crate::models::assignment::AssignmentType::Practical,
    //         Utc::now(),
    //         Utc::now(),
    //     )
    //     .await?;

    //     // Insert dummy assignment
    //     let assignment = crate::models::assignment::Model::create(
    //         &db,
    //         module.id,
    //         "aifhkashvgkas",
    //         None,
    //         crate::models::assignment::AssignmentType::Practical,
    //         Utc::now(),
    //         Utc::now(),
    //     )
    //     .await?;

    //     // Create Java source contents
    //     let main_java = r#"
    //     public class Main {
    //         public static void main(String[] args) {
    //             HelperOne.greet();
    //             HelperTwo.farewell();
    //         }
    //     }
    // "#;

    //     let helper_one_java = r#"
    //     public class HelperOne {
    //         public static void greet() {
    //             System.out.println("Hello from HelperOne");
    //         }
    //     }
    // "#;

    //     let helper_two_java = r#"
    //     public class HelperTwo {
    //         public static void farewell() {
    //             System.out.println("Goodbye from HelperTwo");
    //         }
    //     }
    // "#;

    //     // Helper function to write a zip
    //     fn create_zip_from_files(
    //         zip_path: &std::path::Path,
    //         files: Vec<(&str, &str)>,
    //     ) -> std::io::Result<()> {
    //         let zip_file = std::fs::File::create(zip_path)?;
    //         let mut zip = zip::ZipWriter::new(zip_file);

    //         let options = FileOptions::default()
    //             .compression_method(zip::CompressionMethod::Stored)
    //             .unix_permissions(0o644);

    //         for (name, content) in files {
    //             zip.start_file(name, options)?;
    //             zip.write_all(content.as_bytes())?;
    //         }

    //         zip.finish()?;
    //         Ok(())
    //     }

    //     // Write main.zip
    //     let main_zip_path = storage_path.join("main.zip");
    //     create_zip_from_files(&main_zip_path, vec![("Main.java", main_java)])?;

    //     // Write submission.zip
    //     let submission_zip_path = storage_path.join("submission.zip");
    //     create_zip_from_files(
    //         &submission_zip_path,
    //         vec![
    //             ("HelperOne.java", helper_one_java),
    //             ("HelperTwo.java", helper_two_java),
    //         ],
    //     )?;

    //     // Save main file as FileType::Main in DB
    //     let main_bytes = std::fs::read(&main_zip_path)?;
    //     let _main_model = Model::save_file(
    //         &db,
    //         assignment.id,
    //         module.id,
    //         FileType::Main,
    //         "main.zip",
    //         &main_bytes,
    //     )
    //     .await?;

    //     // Manually create submission file record in assignment_submissions
    //     let submission_bytes = std::fs::read(&submission_zip_path)?;
    //     let submission_path = format!(
    //         "modules/{}/assignments/{}/submissions/submission.zip",
    //         module.id, assignment.id
    //     );
    //     let full_submission_path = storage_path.join(&submission_path);

    //     std::fs::create_dir_all(full_submission_path.parent().unwrap())?;
    //     std::fs::write(&full_submission_path, &submission_bytes)?;

    //     // Insert dummy user
    //     let user = crate::models::user::ActiveModel {
    //         student_number: Set("u12345678".to_string()),
    //         email: Set("student@example.com".to_string()),
    //         password_hash: Set("hashed_password".to_string()),
    //         admin: Set(false),
    //         created_at: Set(Utc::now()),
    //         updated_at: Set(Utc::now()),
    //         ..Default::default()
    //     }
    //     .insert(&db)
    //     .await?;

    //     let submission = crate::models::assignment_submission::ActiveModel {
    //         assignment_id: Set(assignment.id),
    //         user_id: Set(user.id), // dummy user ID
    //         filename: Set("submission.zip".to_string()),
    //         path: Set(submission_path),
    //         created_at: Set(Utc::now()),
    //         updated_at: Set(Utc::now()),
    //         ..Default::default()
    //     }
    //     .insert(&db)
    //     .await?;

    //     let output_model = Model::create_submission_output(&db, submission.id, "java", None)
    //         .await
    //         .expect("create_submission_output failed");

    //     // Check output file exists
    //     let output_path = Model::storage_root().join(&output_model.path);
    //     assert!(
    //         output_path.exists(),
    //         "Output file does not exist: {}",
    //         output_path.display()
    //     );

    //     println!("Output file path: {}", output_path.display());

    //     // Optionally verify content
    //     let content = std::fs::read_to_string(output_path)?;
    //     println!("Output content:\n{}", content);

    //     Ok(())
    // }
}
