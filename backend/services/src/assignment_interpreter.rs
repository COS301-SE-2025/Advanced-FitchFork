use crate::service::{AppError, Service, ToActiveModel};
use chrono::Utc;
use db::{
    models::assignment_interpreter::{ActiveModel, Column, Entity, Model},
    repository::Repository,
};
use sea_orm::{DbErr, Set};
use std::path::PathBuf;
use std::{env, fs};
use util::filters::FilterParam;

pub use db::models::assignment_interpreter::Model as AssignmentInterpreter;

#[derive(Debug, Clone)]
pub struct CreateAssignmentInterpreter {
    pub assignment_id: i64,
    pub module_id: i64,
    pub filename: String,
    pub command: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct UpdateAssignmentInterpreter {
    pub id: i64,
}

impl ToActiveModel<Entity> for CreateAssignmentInterpreter {
    async fn into_active_model(self) -> Result<ActiveModel, AppError> {
        let now = Utc::now();
        Ok(ActiveModel {
            assignment_id: Set(self.assignment_id),
            filename: Set(self.filename.to_string()),
            path: Set("".to_string()), // updated after file write
            command: Set(self.command.to_string()),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        })
    }
}

impl ToActiveModel<Entity> for UpdateAssignmentInterpreter {
    async fn into_active_model(self) -> Result<ActiveModel, AppError> {
        let announcement = match Repository::<Entity, Column>::find_by_id(self.id).await {
            Ok(Some(announcement)) => announcement,
            Ok(None) => {
                return Err(AppError::from(DbErr::RecordNotFound(format!(
                    "AssignmentInterpreter not found for ID {}",
                    self.id
                ))));
            }
            Err(err) => return Err(AppError::from(err)),
        };
        let mut active: ActiveModel = announcement.into();

        active.updated_at = Set(Utc::now());

        Ok(active)
    }
}

pub struct AssignmentInterpreterService;

impl<'a> Service<'a, Entity, Column, CreateAssignmentInterpreter, UpdateAssignmentInterpreter>
    for AssignmentInterpreterService
{
    // ↓↓↓ OVERRIDE DEFAULT BEHAVIOR IF NEEDED HERE ↓↓↓

    fn create(
        params: CreateAssignmentInterpreter,
    ) -> std::pin::Pin<
        Box<
            dyn std::prelude::rust_2024::Future<
                    Output = Result<<Entity as sea_orm::EntityTrait>::Model, AppError>,
                > + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            // Remove any existing interpreter for this assignment
            let existing = AssignmentInterpreter::find()
                .filter(Column::AssignmentId.eq(assignment_id))
                .all(db)
                .await?;

            for record in existing {
                let existing_path = storage_root().join(&record.path);
                let _ = fs::remove_file(existing_path);
                record.delete(db).await?;
            }

            let now = Utc::now();

            let partial = ActiveModel {
                assignment_id: Set(assignment_id),
                filename: Set(filename.to_string()),
                path: Set(String::new()),
                command: Set(command.to_string()),
                created_at: Set(now),
                updated_at: Set(now),
                ..Default::default()
            };

            let inserted: Model = partial.insert(db).await?;

            let ext = PathBuf::from(filename)
                .extension()
                .map(|e| e.to_string_lossy().to_string());

            let stored_filename = match ext {
                Some(ext) => format!("{}.{}", inserted.id, ext),
                None => inserted.id.to_string(),
            };

            let dir_path = interpreter_dir(module_id, assignment_id);
            fs::create_dir_all(&dir_path)
                .map_err(|e| sea_orm::DbErr::Custom(format!("Failed to create directory: {e}")))?;

            let file_path = dir_path.join(&stored_filename);
            fs::write(&file_path, bytes)
                .map_err(|e| sea_orm::DbErr::Custom(format!("Failed to write file: {e}")))?;

            let relative_path = file_path
                .strip_prefix(storage_root())
                .unwrap()
                .to_string_lossy()
                .to_string();

            let mut model: ActiveModel = inserted.into();
            model.path = Set(relative_path);
            model.updated_at = Set(Utc::now());

            model.update(db).await
        })
    }
}

impl AssignmentInterpreterService {
    // ↓↓↓ CUSTOM METHODS CAN BE DEFINED HERE ↓↓↓

    pub fn storage_root() -> PathBuf {
        env::var("ASSIGNMENT_STORAGE_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("data/interpreters"))
    }

    pub fn full_directory_path(module_id: i64, assignment_id: i64) -> PathBuf {
        Self::storage_root()
            .join(format!("module_{}", module_id))
            .join(format!("assignment_{}", assignment_id))
            .join("interpreter")
    }

    pub async fn load_file(id: i64) -> Result<Vec<u8>, std::io::Error> {
        let interpreter = Repository::<Entity, Column>::find_by_id(id)
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("DB error: {e}")))?
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Interpreter ID {} not found", id),
                )
            })?;
        let full_path = Self::storage_root().join(interpreter.path);
        fs::read(full_path)
    }

    pub async fn delete_file_only(id: i64) -> Result<(), std::io::Error> {
        let interpreter = Repository::<Entity, Column>::find_by_id(id)
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("DB error: {e}")))?
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Interpreter ID {} not found", id),
                )
            })?;
        let full_path = Self::storage_root().join(interpreter.path);
        fs::remove_file(full_path)
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::test_utils::setup_test_db;
//     use chrono::Utc;
//     use sea_orm::Set;
//     use tempfile::TempDir;
//     use util::test_helpers::setup_test_assignment_root; // <── use your new helper
//     use util::paths::{interpreter_dir, interpreter_path};
//     use serial_test::serial;
//
//     fn fake_bytes() -> Vec<u8> {
//         vec![0x50, 0x4B, 0x03, 0x04] // ZIP signature
//     }
//
//     #[tokio::test]
//     #[serial]
//     async fn test_save_and_load_file() {
//         let temp_dir = setup_test_assignment_root();
//         let db = setup_test_db().await;
//
//         // Insert dummy module so assignment FK passes
//         let _module = crate::models::module::ActiveModel {
//             code: Set("COS301".to_string()),
//             year: Set(2025),
//             description: Set(Some("Capstone".to_string())),
//             created_at: Set(Utc::now()),
//             updated_at: Set(Utc::now()),
//             ..Default::default()
//         }
//         .insert(&db)
//         .await
//         .expect("Insert module failed");
//
//         // Insert dummy assignment
//         let _assignment = crate::models::assignment::Model::create(
//             &db,
//             1,
//             "Test Assignment",
//             Some("Desc"),
//             crate::models::assignment::AssignmentType::Practical,
//             Utc::now(),
//             Utc::now(),
//         )
//         .await
//         .expect("Insert assignment failed");
//
//         // Save a fake file
//         let content = fake_bytes();
//         let filename = "interpreter.sh";
//         let command = "sh interpreter.sh";
//         let saved = Model::save_file(&db, 1, 1, filename, command, &content)
//             .await
//             .expect("Failed to save interpreter");
//
//         assert_eq!(saved.assignment_id, 1);
//         assert_eq!(saved.filename, filename);
//         assert_eq!(saved.command, command);
//
//         // Build expected path using path utils
//         let expected_dir = interpreter_dir(1, 1);
//         let expected_path = expected_dir.join(filename);
//         assert!(
//             expected_path.exists(),
//             "expected file missing at {:?}",
//             expected_path
//         );
//
//         // Load contents
//         let bytes = saved.load_file().unwrap();
//         assert_eq!(bytes, content);
//
//         // Delete file only
//         saved.delete_file_only().unwrap();
//         assert!(
//             !expected_path.exists(),
//             "file should be removed at {:?}",
//             expected_path
//         );
//     }
// }
