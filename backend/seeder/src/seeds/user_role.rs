use sea_orm::{ActiveModelTrait, EntityTrait, Set, DatabaseConnection};
use db::models::user_module_role::{self, Role};
use db::models::{user, module};
use crate::seed::Seeder;

pub struct UserRoleSeeder;

#[async_trait::async_trait]
impl Seeder for UserRoleSeeder {
    async fn seed(&self, db: &DatabaseConnection) {
        let users = user::Entity::find().all(db).await.expect("Failed to fetch users");
        let modules = module::Entity::find().all(db).await.expect("Failed to fetch modules");

        for (i, u) in users.iter().enumerate() {
            if let Some(m) = modules.get(i % modules.len()) {
                let role = if u.id % 3 == 0 { Role::Lecturer } else { Role::Student };
                let umr = user_module_role::ActiveModel {
                    user_id: Set(u.id),
                    module_id: Set(m.id),
                    role: Set(role),
                    ..Default::default()
                };
                let _ = umr.insert(db).await;
            }
        }
    }
}
