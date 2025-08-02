use sea_orm_migration::prelude::*;

use crate::migrations;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(migrations::m202505290001_create_users::Migration),
            Box::new(migrations::m202505290002_create_modules::Migration),
            Box::new(migrations::m202505290003_create_user_module_roles::Migration),
            Box::new(migrations::m202505290004_create_assignments::Migration),
            Box::new(migrations::m202505290005_create_assignment_files::Migration),
            Box::new(migrations::m202505290006_create_assignment_submissions::Migration),
            Box::new(migrations::m202505290007_create_password_reset_tokens::Migration),
            Box::new(migrations::m202505290008_create_tasks::Migration),
            Box::new(migrations::m202505290009_create_memo_outputs::Migration),
            Box::new(migrations::m202505290010_create_submission_outputs::Migration),
            Box::new(migrations::m202505290011_create_overwrite_files::Migration),
            Box::new(migrations::m202506080012_plagiarism_case::Migration),
            Box::new(migrations::m202508020001_create_tickets::Migration),
        ]
    }
}
