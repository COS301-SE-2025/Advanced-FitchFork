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
        let users = user::Entity::find().all(db).await.expect("Failed to fetch users");
        let modules = module::Entity::find().all(db).await.expect("Failed to fetch modules");

        let mut rng = StdRng::from_rng(OsRng).expect("Failed to seed RNG");

        for u in users {
            if u.student_number == "u00000002" {
                if modules.len() < 10 {
                    panic!("Need at least 10 modules to assign 10 to the normal user");
                }

                // Select 10 unique modules
                let mut selected = modules.choose_multiple(&mut rng, 10).cloned().collect::<Vec<_>>();

                // Assign 1 as Lecturer
                let lecturer_mod = selected.pop().unwrap();
                let _ = user_module_role::ActiveModel {
                    user_id: Set(u.id),
                    module_id: Set(lecturer_mod.id),
                    role: Set(Role::Lecturer),
                    ..Default::default()
                }
                .insert(db)
                .await;

                // Assign 1 as Tutor
                let tutor_mod = selected.pop().unwrap();
                let _ = user_module_role::ActiveModel {
                    user_id: Set(u.id),
                    module_id: Set(tutor_mod.id),
                    role: Set(Role::Tutor),
                    ..Default::default()
                }
                .insert(db)
                .await;

                // Assign remaining 8 as Student
                for m in selected {
                    let _ = user_module_role::ActiveModel {
                        user_id: Set(u.id),
                        module_id: Set(m.id),
                        role: Set(Role::Student),
                        ..Default::default()
                    }
                    .insert(db)
                    .await;
                }
            } else {
                // Random users get 3–6 student modules
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
