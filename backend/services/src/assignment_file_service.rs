// use crate::service::Service;
// use db::{
//     models::assignment_file::{self, ActiveModel, FileType, Model},
//     repositories::{repository::Repository, assignment_file_repository::AssignmentFileRepository},
// };
// use sea_orm::{DbErr, Set};
// use util::execution_config::ExecutionConfig;
// use db::filters::AssignmentFileFilter;
// use std::{env, fs, path::PathBuf};
// use chrono::Utc;

// pub struct AssignmentFileService {
//     repo: AssignmentFileRepository,
// }

// impl<'a> Service<'a, assignment_file::Entity, AssignmentFileFilter, AssignmentFileRepository> for AssignmentFileService {
//     fn repository(&self) -> &AssignmentFileRepository {
//         &self.repo
//     }

//     // ↓↓↓ OVERRIDE DEFAULT BEHAVIOR IF NEEDED HERE ↓↓↓
// }

// impl AssignmentFileService {
//     pub fn new(repo: AssignmentFileRepository) -> Self {
//         Self { repo }
//     }

//     // ↓↓↓ CUSTOM METHODS CAN BE DEFINED HERE ↓↓↓

//     pub fn storage_root() -> PathBuf {
//         env::var("ASSIGNMENT_STORAGE_ROOT")
//             .map(PathBuf::from)
//             .unwrap_or_else(|_| PathBuf::from("data/assignment_files"))
//     }

//     pub fn full_directory_path(module_id: i64, assignment_id: i64, file_type: &FileType) -> PathBuf {
//         Self::storage_root()
//             .join(format!("module_{module_id}"))
//             .join(format!("assignment_{assignment_id}"))
//             .join(file_type.to_string())
//     }

//     pub fn full_path(path: &str) -> PathBuf {
//         Self::storage_root().join(path)
//     }

//     pub fn load_execution_config(
//         &self,
//         module_id: i64,
//         assignment_id: i64,
//     ) -> Result<ExecutionConfig, String> {
//         ExecutionConfig::get_execution_config(module_id, assignment_id)
//     }

//     pub async fn save_file(
//         &self,
//         assignment_id: i64,
//         module_id: i64,
//         file_type: FileType,
//         filename: &str,
//         bytes: &[u8],
//     ) -> Result<Model, DbErr> {
//         let now = Utc::now();

//         if let Some(existing) = self
//             .repo
//             .find_one(AssignmentFileFilter {
//                 assignment_id: Some(assignment_id),
//                 ..Default::default()
//             })
//             .await?
//         {
//             let existing_path = Self::full_path(&existing.path);
//             let _ = fs::remove_file(existing_path);
//             self.repo.delete(existing.id).await?;
//         }

//         let partial = ActiveModel {
//             assignment_id: Set(assignment_id),
//             filename: Set(filename.to_string()),
//             path: Set("".to_string()),
//             file_type: Set(file_type.clone()),
//             created_at: Set(now),
//             updated_at: Set(now),
//             ..Default::default()
//         };
//         let inserted = self.repo.create(partial).await?;

//         let ext = PathBuf::from(filename)
//             .extension()
//             .map(|e| e.to_string_lossy().to_string());
//         let stored_filename = ext
//             .map(|ext| format!("{}.{}", inserted.id, ext))
//             .unwrap_or_else(|| inserted.id.to_string());

//         let dir_path = Self::full_directory_path(module_id, assignment_id, &file_type);
//         fs::create_dir_all(&dir_path)
//             .map_err(|e| DbErr::Custom(format!("Failed to create directory: {e}")))?;

//         let file_path = dir_path.join(&stored_filename);
//         let relative_path = file_path
//             .strip_prefix(Self::storage_root())
//             .unwrap()
//             .to_string_lossy()
//             .to_string();

//         fs::write(&file_path, bytes)
//             .map_err(|e| DbErr::Custom(format!("Failed to write file: {e}")))?;

//         let mut model: ActiveModel = inserted.into();
//         model.path = Set(relative_path);
//         model.updated_at = Set(Utc::now());

//         self.repo.update(model).await
//     }

//     pub fn load_file(&self, model: &Model) -> Result<Vec<u8>, std::io::Error> {
//         fs::read(Self::full_path(&model.path))
//     }

//     pub fn delete_file_only(&self, model: &Model) -> Result<(), std::io::Error> {
//         fs::remove_file(Self::full_path(&model.path))
//     }

//     pub async fn get_base_files(&self, assignment_id: i64) -> Result<Vec<Model>, DbErr> {
//         self.repo
//             .find_all(AssignmentFileFilter {
//                 assignment_id: Some(assignment_id),
//                 ..Default::default()
//             })
//             .await
//     }
// }