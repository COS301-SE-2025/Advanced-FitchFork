use crate::seed::Seeder;
use db::models::user_module_role::{self, Role};
use db::models::{module, user};
use rand::rngs::{OsRng, StdRng};
use rand::{Rng, SeedableRng, seq::SliceRandom};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};

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

        // helper: assign ALL modules with a single role
        async fn assign_all_as(
            db: &DatabaseConnection,
            mods: &[module::Model],
            uid: i64,
            role: Role,
        ) {
            for m in mods {
                let _ = user_module_role::ActiveModel {
                    user_id: Set(uid),
                    module_id: Set(m.id),
                    role: Set(role.clone()),
                    ..Default::default()
                }
                .insert(db)
                .await;
            }
        }

        for u in users {
            // Skip only `admin` (admin)
            if u.username == "admin" {
                continue;
            }

            match u.username.as_str() {
                // Existing single-role fixtures
                "lecturer" => {
                    assign_all_as(db, &modules, u.id, Role::Lecturer).await;
                }
                "assistant_lecturer" => {
                    assign_all_as(db, &modules, u.id, Role::AssistantLecturer).await;
                }
                "tutor" => {
                    assign_all_as(db, &modules, u.id, Role::Tutor).await;
                }
                "student" => {
                    assign_all_as(db, &modules, u.id, Role::Student).await;
                }

                // student_tutor => split half Student, half Tutor
                "student_tutor" => {
                    if modules.is_empty() {
                        continue;
                    }
                    let mut shuffled = modules.clone();
                    shuffled.shuffle(&mut rng);
                    let mid = shuffled.len() / 2;

                    for m in &shuffled[..mid] {
                        let _ = user_module_role::ActiveModel {
                            user_id: Set(u.id),
                            module_id: Set(m.id),
                            role: Set(Role::Student),
                            ..Default::default()
                        }
                        .insert(db)
                        .await;
                    }
                    for m in &shuffled[mid..] {
                        let _ = user_module_role::ActiveModel {
                            user_id: Set(u.id),
                            module_id: Set(m.id),
                            role: Set(Role::Tutor),
                            ..Default::default()
                        }
                        .insert(db)
                        .await;
                    }
                }

                // all_staff => round-robin Lecturer -> AssistantLecturer -> Tutor
                "all_staff" => {
                    if modules.is_empty() {
                        continue;
                    }
                    let mut rr = [Role::Lecturer, Role::AssistantLecturer, Role::Tutor]
                        .iter()
                        .cycle();
                    for m in &modules {
                        let r = rr.next().unwrap().clone();
                        let _ = user_module_role::ActiveModel {
                            user_id: Set(u.id),
                            module_id: Set(m.id),
                            role: Set(r),
                            ..Default::default()
                        }
                        .insert(db)
                        .await;
                    }
                }

                // lecturer_assistant => round-robin Lecturer -> AssistantLecturer
                "lecturer_assistant" => {
                    if modules.is_empty() {
                        continue;
                    }
                    let mut rr = [Role::Lecturer, Role::AssistantLecturer].iter().cycle();
                    for m in &modules {
                        let r = rr.next().unwrap().clone();
                        let _ = user_module_role::ActiveModel {
                            user_id: Set(u.id),
                            module_id: Set(m.id),
                            role: Set(r),
                            ..Default::default()
                        }
                        .insert(db)
                        .await;
                    }
                }

                // NEW: all => round-robin Student -> Tutor -> AssistantLecturer -> Lecturer
                "all" => {
                    if modules.is_empty() {
                        continue;
                    }
                    let mut rr = [
                        Role::Student,
                        Role::Tutor,
                        Role::AssistantLecturer,
                        Role::Lecturer,
                    ]
                    .iter()
                    .cycle();
                    for m in &modules {
                        let r = rr.next().unwrap().clone();
                        let _ = user_module_role::ActiveModel {
                            user_id: Set(u.id),
                            module_id: Set(m.id),
                            role: Set(r),
                            ..Default::default()
                        }
                        .insert(db)
                        .await;
                    }
                }

                // Everyone else: random 3–6 modules as Student
                _ => {
                    if modules.is_empty() {
                        continue;
                    }
                    let count = rng.gen_range(3..=6).min(modules.len());
                    let assigned = modules
                        .choose_multiple(&mut rng, count)
                        .cloned()
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
}
