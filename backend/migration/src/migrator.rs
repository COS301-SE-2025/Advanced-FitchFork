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
            Box::new(migrations::m202505290006_create_password_reset_tokens::Migration),
            Box::new(migrations::m202505290006_create_assignment_submissions::Migration),
        ]
    }
}
