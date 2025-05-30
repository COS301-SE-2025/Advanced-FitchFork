use crate::seed::Seeder;
use chrono::Utc;
use db::models::{
    assignment, assignment_submission,
    submission_file::{self, Model},
};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use std::{env, fs};

pub struct SubmissionFileSeeder;

#[async_trait::async_trait]
impl Seeder for SubmissionFileSeeder {
    async fn seed(&self, db: &DatabaseConnection) {
        let storage_root =
            env::var("ASSIGNMENT_STORAGE_ROOT").expect("ASSIGNMENT_STORAGE_ROOT must be set");

        let submissions = assignment_submission::Entity::find()
            .all(db)
            .await
            .expect("Failed to fetch submissions");

        for s in &submissions {
            let submission_id = s.id;
            let assignment = assignment::Entity::find_by_id(s.assignment_id)
                .one(db)
                .await
                .expect("Failed to fetch assignment")
                .expect("Assignment not found for submission");

            let module_id = assignment.module_id;
            let assignment_id = assignment.id;

            let stored_filename = format!("file_{submission_id}.txt");
            let dir_path = Model::full_directory_path(module_id, assignment_id);
            fs::create_dir_all(&dir_path)
                .expect("Failed to create directories for submission file");

            let full_path = dir_path.join(&stored_filename);
            let relative_path = full_path
                .strip_prefix(&storage_root)
                .unwrap()
                .to_string_lossy()
                .to_string();

            fs::write(
                &full_path,
                format!("This is the content of submission file {submission_id}"),
            )
            .expect("Failed to write submission file");

            let f = submission_file::ActiveModel {
                submission_id: Set(submission_id),
                filename: Set(stored_filename),
                path: Set(relative_path),
                created_at: Set(Utc::now()),
                updated_at: Set(Utc::now()),
                ..Default::default()
            };

            let _ = f.insert(db).await;
        }
    }
}
