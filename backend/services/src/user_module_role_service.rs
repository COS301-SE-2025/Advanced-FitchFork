// use crate::service::Service;
// use db::{
//     models::{module, user::UserModuleRole, user_module_role::{self, Role}},
//     repositories::{repository::Repository, user_module_role_repository::UserModuleRoleRepository, module_repository::ModuleRepository},
//     filters::{ModuleFilter, UserModuleRoleFilter},
// };
// use sea_orm::{DbErr, Set, Value};
// use std::collections::HashMap;

// pub struct UserModuleRoleService {
//     repo: UserModuleRoleRepository,
// }

// impl<'a> Service<'a, user_module_role::Entity, UserModuleRoleFilter, UserModuleRoleRepository> for UserModuleRoleService {
//     fn repository(&self) -> &UserModuleRoleRepository {
//         &self.repo
//     }

//     // ↓↓↓ OVERRIDE DEFAULT BEHAVIOR IF NEEDED HERE ↓↓↓
// }

// impl UserModuleRoleService {
//     pub fn new(repo: UserModuleRoleRepository) -> Self {
//         Self { repo }
//     }

//     // ↓↓↓ CUSTOM METHODS CAN BE DEFINED HERE ↓↓↓

//     pub async fn assign_user_to_module(
//         &self,
//         user_id: i64,
//         module_id: i64,
//         role: Role,
//     ) -> Result<user_module_role::Model, DbErr> {
//         let active = user_module_role::ActiveModel {
//             user_id: Set(user_id),
//             module_id: Set(module_id),
//             role: Set(role),
//             ..Default::default()
//         };
//         self.repo.create(active).await
//     }

//     pub async fn remove_user_from_module(
//         &self,
//         user_id: i64,
//         module_id: i64,
//     ) -> Result<(), DbErr> {
//         self.repo.delete((user_id, module_id)).await?;
//         Ok(())
//     }

//     pub async fn get_all(&self) -> Result<Vec<user_module_role::Model>, DbErr> {
//         self.repo
//             .find_all(UserModuleRoleFilter::default())
//             .await
//     }

//     pub async fn get_users_by_module_role(
//         &self,
//         module_id: i64,
//         role: Role,
//     ) -> Result<Vec<user_module_role::Model>, DbErr> {
//         self.repo
//             .find_all(UserModuleRoleFilter {
//                 module_id: Some(module_id),
//                 role: Some(role),
//                 ..Default::default()
//             })
//             .await
//     }

//     pub async fn get_modules_by_user_role(
//         &self,
//         user_id: i64,
//         role: Role,
//     ) -> Result<Vec<user_module_role::Model>, DbErr> {
//         self.repo
//             .find_all(UserModuleRoleFilter {
//                 user_id: Some(user_id),
//                 role: Some(role),
//                 ..Default::default()
//             })
//             .await
//     }

//     pub async fn is_in_role(
//         &self,
//         user_id: i64,
//         module_id: i64,
//         role: Role,
//     ) -> Result<bool, DbErr> {
//         Ok(
//             self.repo.exists(UserModuleRoleFilter {
//                 user_id: Some(user_id),
//                 module_id: Some(module_id),
//                 role: Some(role),
//                 ..Default::default()
//             })
//             .await?
//         )
//     }

//     // TODO: A join would be more efficient here. Figure that out @Reece
//     pub async fn get_module_roles<'a>(
//         &self,
//         user_id: i64,
//         module_service: &'a impl crate::service::Service<'a, module::Entity, ModuleFilter, ModuleRepository>,
//     ) -> Result<Vec<UserModuleRole>, DbErr> {
//         let user_roles = self.repo
//             .find_all(UserModuleRoleFilter {
//                 user_id: Some(user_id),
//                 ..Default::default()
//             })
//             .await?;

//         let module_ids: Vec<i64> = user_roles.iter()
//             .map(|ur| ur.module_id)
//             .collect::<std::collections::HashSet<_>>()
//             .into_iter()
//             .collect();

//         let module_id_values: Vec<Value> = module_ids.iter().map(|id| id.clone().into()).collect();

//         let modules = module_service
//             .find_in(module::Column::Id, module_id_values)
//             .await?;

//         let module_map: HashMap<i64, module::Model> = modules.into_iter().map(|m| (m.id, m)).collect();

//         Ok(
//             user_roles.into_iter().filter_map(|role| {
//                 module_map.get(&role.module_id).map(|module| UserModuleRole {
//                     module_id: module.id,
//                     module_code: module.code.clone(),
//                     module_year: module.year,
//                     module_description: module.description.clone(),
//                     module_credits: module.credits,
//                     module_created_at: module.created_at.to_string(),
//                     module_updated_at: module.updated_at.to_string(),
//                     role: role.role.to_string(),
//                 })
//             }).collect()
//         )
//     }
// }