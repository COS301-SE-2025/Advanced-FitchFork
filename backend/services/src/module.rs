use crate::service::{Service, AppError, ToActiveModel};
use db::{
    models::module::{Entity, Column, ActiveModel},
    repository::Repository,
};
use sea_orm::{DbErr, Set};
use std::{env, fs, path::PathBuf};
use log::{info, warn};
use std::future::Future;
use std::pin::Pin;
use chrono::Utc;

pub use db::models::module::Model as Module;

#[derive(Debug, Clone)]
pub struct CreateModule {
    pub id: Option<i64>,
    pub code: String,
    pub year: i64,
    pub description: Option<String>,
    pub credits: i64,
}

#[derive(Debug, Clone)]
pub struct UpdateModule {
    pub id: i64,
    pub code: Option<String>,
    pub year:  Option<i64>,
    pub description: Option<String>,
    pub credits:  Option<i64>,
}

impl ToActiveModel<Entity> for CreateModule {
    async fn into_active_model(self) -> Result<ActiveModel, AppError> {
        let now = Utc::now();
        let mut active: ActiveModel = ActiveModel {
            code: Set(self.code),
            year: Set(self.year),
            description: Set(self.description),
            credits: Set(self.credits),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        if let Some(id) = self.id {
            active.id = Set(id);
        }

        Ok(active)
    }
}

impl ToActiveModel<Entity> for UpdateModule {
    async fn into_active_model(self) -> Result<ActiveModel, AppError> {
        let module = match Repository::<Entity, Column>::find_by_id(self.id).await {
            Ok(Some(module)) => module,
            Ok(None) => {
                return Err(AppError::from(DbErr::RecordNotFound(format!("Module ID {} not found", self.id))));
            }
            Err(err) => return Err(AppError::from(err)),
        };
        let mut active: ActiveModel = module.into();

        if let Some(code) = self.code {
            active.code = Set(code);
        }

        if let Some(year) = self.year {
            active.year = Set(year);
        }

        if let Some(description) = self.description {
            active.description = Set(Some(description));
        }

        if let Some(credits) = self.credits {
            active.credits = Set(credits);
        }

        active.updated_at = Set(Utc::now());

        Ok(active)
    }
}

pub struct ModuleService;

impl<'a> Service<'a, Entity, Column, CreateModule, UpdateModule> for ModuleService {
    // ↓↓↓ OVERRIDE DEFAULT BEHAVIOR IF NEEDED HERE ↓↓↓

    fn delete(
        id: i64,
    ) -> Pin<Box<dyn Future<Output = Result<(), AppError>> + Send>> {
        Box::pin(async move {
            let storage_root = env::var("ASSIGNMENT_STORAGE_ROOT")
                .unwrap_or_else(|_| "data/assignment_files".to_string());

            let module_dir = PathBuf::from(storage_root).join(format!("module_{}", id));

            if module_dir.exists() {
                match fs::remove_dir_all(&module_dir) {
                    Ok(_) => info!("Deleted module directory {}", module_dir.display()),
                    Err(e) => warn!("Failed to delete module directory {}: {}", module_dir.display(), e),
                }
            } else {
                warn!("Expected module directory {} does not exist", module_dir.display());
            }

            Repository::<Entity, Column>::delete(id).await.map_err(AppError::from)
        })
    }
}

impl ModuleService {
    // ↓↓↓ CUSTOM METHODS CAN BE DEFINED HERE ↓↓↓
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use async_trait::async_trait;
//     use db::models::module::{Entity, Model};
//     use std::collections::HashMap;
//     use std::sync::Mutex;

//     struct MockModuleRepository {
//         modules: Mutex<HashMap<i64, Model>>,
//         next_id: Mutex<i64>,
//     }

//     impl MockModuleRepository {
//         fn new() -> Self {
//             Self {
//                 modules: Mutex::new(HashMap::new()),
//                 next_id: Mutex::new(1),
//             }
//         }
//     }

//     #[async_trait]
//     impl Repository<Entity, ModuleFilter> for MockModuleRepository {
//         async fn create(&self, active_model: module::ActiveModel) -> Result<Model, DbErr> {
//             let mut modules = self.modules.lock().unwrap();
//             let mut next_id = self.next_id.lock().unwrap();

//             let id = *next_id;
//             *next_id += 1;

//             let module = Model {
//                 id,
//                 code: active_model.code.unwrap(),
//                 year: active_model.year.unwrap(),
//                 description: active_model.description.unwrap(),
//                 credits: active_model.credits.unwrap(),
//                 created_at: chrono::Utc::now(),
//                 updated_at: chrono::Utc::now(),
//             };

//             modules.insert(id, module.clone());
//             Ok(module)
//         }

//         async fn find_by_id(&self, id: i64) -> Result<Option<Model>, DbErr> {
//             let modules = self.modules.lock().unwrap();
//             Ok(modules.get(&id).cloned())
//         }

//         async fn update(&self, active_model: module::ActiveModel) -> Result<Model, DbErr> {
//             let mut modules = self.modules.lock().unwrap();
//             let id = active_model.id.unwrap();

//             if let Some(module) = modules.get_mut(&id) {
//                 module.code = active_model.code.unwrap();
//                 module.year = active_model.year.unwrap();
//                 module.description = active_model.description.unwrap();
//                 module.credits = active_model.credits.unwrap();
//                 module.updated_at = chrono::Utc::now();
//                 Ok(module.clone())
//             } else {
//                 Err(DbErr::RecordNotFound("Module not found".to_string()))
//             }
//         }

//         async fn delete(&self, id: i64) -> Result<(), DbErr> {
//             let mut modules = self.modules.lock().unwrap();
//             if modules.remove(&id).is_some() {
//                 Ok(())
//             } else {
//                 Err(DbErr::RecordNotFound("Module not found".to_string()))
//             }
//         }

//         async fn filter(
//             &self,
//             filter_params: ModuleFilter,
//             _page: u64,
//             _per_page: u64,
//             _sort_by: Option<String>,
//         ) -> Result<Vec<Model>, DbErr> {
//             let modules = self.modules.lock().unwrap();
//             match filter_params {
//                 ModuleFilter::Code(code) => {
//                     let filtered_modules = modules
//                         .values()
//                         .filter(|m| m.code == code)
//                         .cloned()
//                         .collect();
//                     Ok(filtered_modules)
//                 }
//             }
//         }

//         async fn find_one(&self, filter_params: ModuleFilter) -> Result<Option<Model>, DbErr> {
//             let modules = self.modules.lock().unwrap();
//             match filter_params {
//                 ModuleFilter::Code(code) => {
//                     let module = modules.values().find(|m| m.code == code).cloned();
//                     Ok(module)
//                 }
//             }
//         }
//     }

//     #[tokio::test]
//     async fn test_create_module() {
//         let repo = MockModuleRepository::new();
//         let service = ModuleService::new(repo);

//         let code = "COS301";
//         let year = 2025;
//         let description = Some("Software Engineering");
//         let credits = 16;

//         let module = service
//             .create_module(code, year, description, credits)
//             .await
//             .unwrap();

//         assert_eq!(module.code, code);
//         assert_eq!(module.year, year);
//         assert_eq!(module.description.as_deref(), description);
//         assert_eq!(module.credits, credits);
//     }

//     #[tokio::test]
//     async fn test_edit_module() {
//         let repo = MockModuleRepository::new();
//         let service = ModuleService::new(repo);

//         let initial = service
//             .create_module("COS132", 2024, Some("Old Desc"), 12)
//             .await
//             .unwrap();

//         let updated = service
//             .edit_module(initial.id, "COS133", 2025, Some("New Desc"), 14)
//             .await
//             .unwrap();

//         assert_eq!(updated.id, initial.id);
//         assert_eq!(updated.code, "COS133");
//         assert_eq!(updated.year, 2025);
//         assert_eq!(updated.description.as_deref(), Some("New Desc"));
//         assert_eq!(updated.credits, 14);
//     }
// }