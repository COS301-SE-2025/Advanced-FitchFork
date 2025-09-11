use crate::seed::Seeder;
use services::service::{Service, AppError};
use services::user::UserService;
use services::module::{ModuleService, Module};
use services::user_module_role::{UserModuleRoleService, CreateUserModuleRole, UpdateUserModuleRole};
use rand::{seq::SliceRandom, Rng, SeedableRng};
use rand::rngs::{StdRng, OsRng};
use std::pin::Pin;

pub struct UserRoleSeeder;

impl Seeder for UserRoleSeeder {
    fn seed<'a>(&'a self) -> Pin<Box<dyn Future<Output = Result<(), AppError>> + Send + 'a>> {
        Box::pin(async move {
            let users = UserService::find_all(&[], None).await?;
            let modules = ModuleService::find_all(&[], None).await?;

            let mut rng = StdRng::from_rng(OsRng).expect("Failed to seed RNG");

            // helper: assign ALL modules with a single role
            async fn assign_all_as(
                modules: &Vec<Module>,
                user_id: i64,
                role: &str,
            ) {
                for module in modules {
                    let _ = UserModuleRoleService::create(
                        CreateUserModuleRole{
                            user_id: user_id,
                            module_id: module.id,
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
                    "lecturer" => { assign_all_as(&modules, u.id, "lecturer").await; }
                    "assistant_lecturer" => { assign_all_as(&modules, u.id, "assistant_lecturer").await; }
                    "tutor" => { assign_all_as(&modules, u.id, "tutor").await; }
                    "student" => { assign_all_as(&modules, u.id, "student").await; }

                    // student_tutor => split half Student, half Tutor
                    "student_tutor" => {
                        if modules.is_empty() { continue; }
                        let mut shuffled = modules.clone();
                        shuffled.shuffle(&mut rng);
                        let mid = shuffled.len() / 2;

                        for m in &shuffled[..mid] {
                            let _ = UserModuleRoleService::update(
                                UpdateUserModuleRole{
                                    user_id: u.id,
                                    module_id: m.id,
                                    role: Some("student".to_string()),
                                },
                            ).await;
                        }
                        for m in &shuffled[mid..] {
                            let _ = UserModuleRoleService::update(
                                UpdateUserModuleRole{
                                    user_id: u.id,
                                    module_id: m.id,
                                    role: Some("tutor".to_string()),
                                },
                            ).await;
                        }
                    }

                    // all_staff => round-robin Lecturer -> AssistantLecturer -> Tutor
                    "all_staff" => {
                        if modules.is_empty() { continue; }
                        let mut rr = ["lecturer", "assistant_lecturer", "tutor"].iter().cycle();
                        for m in &modules {
                            let r = rr.next().unwrap();
                            let _ = UserModuleRoleService::update(
                                UpdateUserModuleRole{
                                    user_id: u.id,
                                    module_id: m.id,
                                    role: Some(r.to_string()),
                                },
                            ).await;
                        }
                    }

                    // lecturer_assistant => round-robin Lecturer -> AssistantLecturer
                    "lecturer_assistant" => {
                        if modules.is_empty() { continue; }
                        let mut rr = ["lecturer", "assistant_lecturer"].iter().cycle();
                        for m in &modules {
                            let r = rr.next().unwrap();
                            let _ = UserModuleRoleService::update(
                                UpdateUserModuleRole{
                                    user_id: u.id,
                                    module_id: m.id,
                                    role: Some(r.to_string()),
                                },
                            ).await;
                        }
                    }

                    // NEW: all => round-robin Student -> Tutor -> AssistantLecturer -> Lecturer
                    "all" => {
                        if modules.is_empty() { continue; }
                        let mut rr = ["lecturer", "assistant_lecturer", "tutor", "student"].iter().cycle();
                        for m in &modules {
                            let r = rr.next().unwrap();
                            let _ = UserModuleRoleService::update(
                                UpdateUserModuleRole{
                                    user_id: u.id,
                                    module_id: m.id,
                                    role: Some(r.to_string()),
                                },
                            ).await;
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
                            let _ = UserModuleRoleService::update(
                                UpdateUserModuleRole{
                                    user_id: u.id,
                                    module_id: m.id,
                                    role: Some("student".to_string()),
                                },
                            ).await;
                        }
                    }
                }
            }

            Ok(())
        })
    }
}
