use chrono::Utc;
use sea_orm::{EntityTrait, ActiveModelTrait, Set, DatabaseConnection};
use db::models::assignment_file::{self, FileType};
use db::models::assignment;
use crate::seed::Seeder;
use std::{env, fs, path::PathBuf};

pub struct AssignmentFileSeeder;

#[async_trait::async_trait]
impl Seeder for AssignmentFileSeeder {
    async fn seed(&self, db: &DatabaseConnection) {
        let storage_root = env::var("ASSIGNMENT_STORAGE_ROOT")
            .expect("ASSIGNMENT_STORAGE_ROOT must be set");

        let assignments = assignment::Entity::find()
            .all(db)
            .await
            .expect("Failed to fetch assignments");

        for a in &assignments {
            let assignment_id = a.id;

            let relative_path = format!(
                "module_{}/assignment_{}/spec/spec_{}.txt",
                a.module_id, a.id, a.id
            );
            let full_path = PathBuf::from(&storage_root).join(&relative_path);

            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent).expect("Failed to create directories for assignment file");
            }

            fs::write(&full_path, format!("This is the content of assignment file {assignment_id}"))
                .expect("Failed to write assignment file");

            let f = assignment_file::ActiveModel {
                assignment_id: Set(assignment_id),
                filename: Set(format!("spec_{assignment_id}.txt")),
                path: Set(relative_path.clone()),
                file_type: Set(FileType::Spec),
                created_at: Set(Utc::now()),
                updated_at: Set(Utc::now()),
                ..Default::default()
            };

            let _ = f.insert(db).await;
        }
    }
}
