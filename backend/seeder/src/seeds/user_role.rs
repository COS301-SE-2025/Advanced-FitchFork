use rand::{seq::SliceRandom, Rng, SeedableRng};
use rand::rngs::{StdRng, OsRng};
use sea_orm::{ActiveModelTrait, EntityTrait, Set, DatabaseConnection};
use db::models::user_module_role::{self, Role};
use db::models::{user, module};
use crate::seed::Seeder;

pub struct UserRoleSeeder;

#[async_trait::async_trait]
impl Seeder for UserRoleSeeder {
    async fn seed(&self, db: &DatabaseConnection) {
        let users = user::Entity::find()
            .all(db)
            .await
            .expect("Failed to fetch users");
        let modules = module::Entity::find()
            .all(db)
            .await
            .expect("Failed to fetch modules");

        let mut rng = StdRng::from_rng(OsRng).expect("Failed to seed RNG");

        for u in users {
            // Skip only `admin` (admin)
            if u.username == "admin" {
                continue;
            }

            let role = match u.username.as_str() {
                "lecturer" => Some(Role::Lecturer),
                "assistant_lecturer" => Some(Role::AssistantLecturer),
                "tutor" => Some(Role::Tutor),
                "student" => Some(Role::Student),
                _ => None,
            };

            if let Some(role) = role {
                // Assign ALL modules with the specific role
                for m in &modules {
                    let _ = user_module_role::ActiveModel {
                        user_id: Set(u.id),
                        module_id: Set(m.id),
                        role: Set(role.clone()),
                        ..Default::default()
                    }
                    .insert(db)
                    .await;
                }
            } else {
                // Other users: random 3–6 modules as Student
                let count = rng.gen_range(3..=6);
                let assigned = modules
                    .choose_multiple(&mut rng, count)
                    .collect::<Vec<_>>();

                for m in assigned {
                    let _ = user_module_role::ActiveModel {
                        user_id: Set(u.id),
                        module_id: Set(m.id),
                        role: Set(Role::Student),
                        ..Default::default()
                    }
                    .insert(db)
                    .await;
                }
            }
        }
    }
}
