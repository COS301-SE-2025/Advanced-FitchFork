use crate::seed::Seeder;
use services::service::Service;
use services::user_service::{UserService, CreateUser};
use services::module_service::{ModuleService, CreateModule};
use services::user_module_role_service::{UserModuleRoleService, CreateUserModuleRole};
use rand::{seq::SliceRandom, Rng, SeedableRng};
use rand::rngs::{StdRng, OsRng};
use std::pin::Pin;

pub struct UserRoleSeeder;

impl Seeder for UserRoleSeeder {
    fn seed<'a>(&'a self) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let users = UserService::find_all(&[], None).await;
            let modules = ModuleService::find_all(&[], None).await;

            let mut rng = StdRng::from_rng(OsRng).expect("Failed to seed RNG");

            // helper: assign ALL modules with a single role
            async fn assign_all_as(
                module_ids: Vec<i64>,
                user_id: i64,
                role: &str,
            ) {
                for module_id in module_ids {
                    let _ = UserModuleRoleService::create(
                        CreateUserModuleRole{
                            user_id: user_id,
                            module_id: module_id,
                            role: role.to_string(),
                        }
                    ).await;
                }
            }

                for u in users {
                    // Skip only `admin` (admin)
                    if u.username == "admin" {
                        continue;
                    }

                match u.username.as_str() {
                    // Existing single-role fixtures
                    "lecturer" => { assign_all_as(db, &modules, u.id, Role::Lecturer).await; }
                    "assistant_lecturer" => { assign_all_as(db, &modules, u.id, Role::AssistantLecturer).await; }
                    "tutor" => { assign_all_as(db, &modules, u.id, Role::Tutor).await; }
                    "student" => { assign_all_as(db, &modules, u.id, Role::Student).await; }

                    // student_tutor => split half Student, half Tutor
                    "student_tutor" => {
                        if modules.is_empty() { continue; }
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
                        if modules.is_empty() { continue; }
                        let mut rr = [Role::Lecturer, Role::AssistantLecturer, Role::Tutor].iter().cycle();
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
                        if modules.is_empty() { continue; }
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
                        if modules.is_empty() { continue; }
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
                        if modules.is_empty() { continue; }
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
        })
    }
}
