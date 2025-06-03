use crate::seed::Seeder;
use db::models::assignment;
use db::models::assignment_file::{FileType, Model};
use sea_orm::{DatabaseConnection, EntityTrait};

pub struct AssignmentFileSeeder;

#[async_trait::async_trait]
impl Seeder for AssignmentFileSeeder {
    async fn seed(&self, db: &DatabaseConnection) {
        let assignments = assignment::Entity::find()
            .all(db)
            .await
            .expect("Failed to fetch assignments");

        // Define the file types and corresponding filename templates
        let file_types: Vec<(FileType, fn(i32) -> String)> = vec![
            (FileType::Spec, |id| format!("spec_{}.txt", id)),
            (FileType::Memo, |id| format!("memo_{}.txt", id)),
            (FileType::Main, |id| format!("main_{}.txt", id)),
            (FileType::Additional, |id| format!("additional_{}.txt", id)),
            (FileType::Makefile, |id| format!("makefile_{}.txt", id)),
            (FileType::MarkAllocator, |id| {
                format!("mark_allocator_{}.txt", id)
            }),
            (FileType::Config, |id| format!("config_{}.txt", id)),
        ];

        for a in &assignments {
            for &(ref file_type, filename_fn) in &file_types {
                let filename = filename_fn(a.id.try_into().unwrap());
                let content = format!("This is the content of assignment file {}", a.id);

                let _ = Model::save_file(
                    db,
                    a.id,
                    a.module_id,
                    file_type.clone(),
                    &filename,
                    content.as_bytes(),
                )
                .await;
            }
        }
    }
}
