use crate::service::{Service, AppError, ToActiveModel};
use db::{
    models::{
        module::{Model, Entity as ModuleEntity, Column as ModuleColumn},
        user_module_role::{ActiveModel, Entity as UserModuleRoleEntity, Column as UserModuleRoleColumn, Role},
    },
    repository::Repository,
};
use util::filters::FilterParam;
use sea_orm::{DbErr, Set};
use serde::Serialize;

pub use db::models::user_module_role::Model as UserModuleRole;

#[derive(Debug, Clone, Serialize)]
pub struct UserModuleRoleInfo {
    pub module_id: i64,
    pub module_code: String,
    pub module_year: i64,
    pub module_description: Option<String>,
    pub module_credits: i64,
    pub module_created_at: String,
    pub module_updated_at: String,
    pub role: String,
}

#[derive(Debug, Clone)]
pub struct CreateUserModuleRole {
    pub user_id: i64,
    pub module_id: i64,
    pub role: String,
}

#[derive(Debug, Clone)]
pub struct UpdateUserModuleRole {
    pub user_id: i64,
    pub module_id: i64,
    pub role: Option<String>,
}

impl ToActiveModel<UserModuleRoleEntity> for CreateUserModuleRole {
    async fn into_active_model(self) -> Result<ActiveModel, AppError> {
        Ok(ActiveModel {
            user_id: Set(self.user_id),
            module_id: Set(self.module_id),
            role: Set(self.role.trim().parse::<Role>().map_err(|e| DbErr::Custom(format!("Invalid role string '{}': {}", self.role, e)))?),
            ..Default::default()
        })
    }
}

impl ToActiveModel<UserModuleRoleEntity> for UpdateUserModuleRole {
    async fn into_active_model(self) -> Result<ActiveModel, AppError> {
        let task = match Repository::<UserModuleRoleEntity, UserModuleRoleColumn>::find_by_id((self.user_id, self.module_id)).await {
            Ok(Some(task)) => task,
            Ok(None) => {
                return Err(AppError::from(DbErr::RecordNotFound(format!("Role not found for user ID {} and module ID {}", self.user_id, self.module_id))));
            }
            Err(err) => return Err(AppError::from(err)),
        };
        let mut active: ActiveModel = task.into();

        if let Some(role) = self.role {
            active.role = Set(role.trim().parse::<Role>().map_err(|e| DbErr::Custom(format!("Invalid role string '{}': {}", role, e)))?);
        }

        Ok(active)
    }
}

pub struct UserModuleRoleService;

impl<'a> Service<'a, UserModuleRoleEntity, UserModuleRoleColumn, CreateUserModuleRole, UpdateUserModuleRole> for UserModuleRoleService {
    // ↓↓↓ OVERRIDE DEFAULT BEHAVIOR IF NEEDED HERE ↓↓↓
}

impl UserModuleRoleService {
    // ↓↓↓ CUSTOM METHODS CAN BE DEFINED HERE ↓↓↓

    // pub async fn get_users_by_module_role(
    //     module_id: i64,
    //     role: String,
    // ) -> Result<Vec<Model>, DbErr> {
    //     Repository::<UserModuleRoleEntity, UserModuleRoleColumn>::find_all(
    //         UserModuleRoleFilter {
    //             module_id: Some(module_id),
    //             role: Some(Role::from_str(&role).map_err(|_| DbErr::Custom(format!("Invalid role string: '{}'", role)))?),
    //             ..Default::default()
    //         }
    //     ).await
    // }

    // pub async fn get_modules_by_user_role(
    //     user_id: i64,
    //     role: String,
    // ) -> Result<Vec<Model>, DbErr> {
    //     Repository::<UserModuleRoleEntity, UserModuleRoleColumn>::find_all(
    //         UserModuleRoleFilter {
    //             user_id: Some(user_id),
    //             role: Some(Role::from_str(&role).map_err(|_| DbErr::Custom(format!("Invalid role string: '{}'", role)))?),
    //             ..Default::default()
    //         }
    //     ).await
    // }

    pub async fn get_module_roles(
        user_id: i64
    ) -> Result<Vec<UserModuleRoleInfo>, DbErr> {
        let roles = Repository::<UserModuleRoleEntity, UserModuleRoleColumn>::find_all(
            &vec![
                FilterParam::eq("user_id", user_id),
            ],
            None
        ).await?;
        let modules = Repository::<ModuleEntity, ModuleColumn>::find_all(
            &vec![
                FilterParam::eq("id", roles.iter().map(|role| role.module_id).collect::<Vec<i64>>()),
            ],
            None
        ).await?;

        let modules_by_id: std::collections::HashMap<i64, Model> =
            modules.into_iter().map(|m| (m.id, m)).collect();

        let result = roles
            .into_iter()
            .filter_map(|role| {
                modules_by_id.get(&role.module_id).map(|module| {
                    UserModuleRoleInfo {
                        module_id: module.id,
                        module_code: module.code.clone(),
                        module_year: module.year,
                        module_description: module.description.clone(),
                        module_credits: module.credits,
                        module_created_at: module.created_at.to_string(),
                        module_updated_at: module.updated_at.to_string(),
                        role: role.role.to_string(),
                    }
                })
            })
            .collect();

        Ok(result)
    }

    pub async fn is_in_role(
        user_id: i64,
        module_id: i64,
        role: String,
    ) -> Result<bool, DbErr> {
        Repository::<UserModuleRoleEntity, UserModuleRoleColumn>::exists(
            &vec![
                FilterParam::eq("user_id", user_id),
                FilterParam::eq("module_id", module_id),
                FilterParam::eq("role", role),
            ]
        ).await
    }
}