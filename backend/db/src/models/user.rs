use sea_orm::entity::prelude::*;
use serde::Serialize;
use chrono::{DateTime, Utc};

use crate::models::user_module_role::Role;
use std::str::FromStr;
use crate::models::{
    module::Entity as ModuleEntity,
    user_module_role::{Column as RoleColumn, Entity as RoleEntity},
};

/// Represents a user in the `users` table.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    /// Primary key ID (auto-incremented).
    #[sea_orm(primary_key)]
    pub id: i64,
    /// Unique student number.
    pub username: String,
    /// User's unique email address.
    pub email: String,
    /// Securely hashed password string.
    pub password_hash: String,
    /// Whether the user has admin privileges.
    pub admin: bool,
    /// Timestamp when the user was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp when the user was last updated.
    pub updated_at: DateTime<Utc>,
    //User profile picture
    pub profile_picture_path: Option<String>,
}

/// This enum would define relations if any exist. Currently unused.
#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        panic!("No RelationDef implemented")
    }
}

/// Struct returned by `get_module_roles`, summarizing a user's role in a module.
#[derive(Debug, Clone)]
pub struct UserModuleRole {
    pub module_id: i64,
    pub module_code: String,
    pub module_year: i32,
    pub module_description: Option<String>,
    pub module_credits: i32,
    pub module_created_at: String,
    pub module_updated_at: String,
    pub role: String,
}

impl ActiveModelBehavior for ActiveModel {}
impl Model {
    pub async fn get_module_roles(
        db: &DatabaseConnection,
        user_id: i64,
    ) -> Result<Vec<UserModuleRole>, DbErr> {
        let roles = RoleEntity::find()
            .filter(RoleColumn::UserId.eq(user_id))
            .find_also_related(ModuleEntity)
            .all(db)
            .await?;

        Ok(roles
            .into_iter()
            .filter_map(|(role, maybe_module)| {
                maybe_module.map(|module| UserModuleRole {
                    module_id: module.id,
                    module_code: module.code,
                    module_year: module.year,
                    module_description: module.description,
                    module_credits: module.credits,
                    module_created_at: module.created_at.to_string(),
                    module_updated_at: module.updated_at.to_string(),
                    role: role.role.to_string(),
                })
            })
            .collect())
    }
    
    pub async fn is_in_role(
        db: &DatabaseConnection,
        user_id: i64,
        module_id: i64,
        role: &str,
    ) -> Result<bool, DbErr> {
        let parsed_role = Role::from_str(role).map_err(|_| {
            DbErr::Custom(format!("Invalid role string: '{}'", role))
        })?;

        let exists = RoleEntity::find()
            .filter(RoleColumn::UserId.eq(user_id))
            .filter(RoleColumn::ModuleId.eq(module_id))
            .filter(RoleColumn::Role.eq(parsed_role))
            .one(db)
            .await?;

        Ok(exists.is_some())
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::test_utils::setup_test_db;
//     use crate::models::user_module_role::Role as UserRole;

//     #[tokio::test]
//     async fn test_create_and_get_user() {
//         let db = setup_test_db().await;
//         let username = "u12345678";
//         let email = "test@example.com";
//         let password = "secret123";

//         let _user = Model::create(&db, username, email, password, false)
//             .await
//             .expect("Failed to create user");

//         let found = Model::get_by_username(&db, username)
//             .await
//             .expect("Failed to query user");

//         assert!(found.is_some());
//         let found = found.unwrap();
//         assert_eq!(found.email, email);
//         assert_eq!(found.username, username);
//         assert_eq!(found.admin, false);
//     }

//     #[tokio::test]
//     async fn test_verify_credentials_success_and_failure() {
//         let db = setup_test_db().await;
//         let username = "u87654321";
//         let email = "auth@example.com";
//         let password = "correct_pw";

//         Model::create(&db, username, email, password, false)
//             .await
//             .expect("Failed to create user");

//         let ok = Model::verify_credentials(&db, username, password)
//             .await
//             .expect("Failed to verify credentials");
//         assert!(ok.is_some());

//         let bad = Model::verify_credentials(&db, username, "wrong_pw")
//             .await
//             .expect("Failed to verify wrong credentials");
//         assert!(bad.is_none());
//     }

//     #[tokio::test]
//     async fn test_is_in_role_and_get_module_roles() {
//         use crate::models::{
//             module::{ActiveModel as ModuleActiveModel},
//             user_module_role::ActiveModel as RoleActiveModel,
//         };

//         let db = setup_test_db().await;

//         // Create user
//         let user = Model::create(&db, "u00001111", "modrole@example.com", "pw", false)
//             .await
//             .expect("create user");

//         // Create module
//         let module = ModuleActiveModel {
//             code: Set("COS999".into()),
//             year: Set(2025),
//             description: Set(Some("Test Module".into())),
//             credits: Set(15),
//             ..Default::default()
//         }
//         .insert(&db)
//         .await
//         .expect("create module");

//         // Assign user to module with role
//         RoleActiveModel {
//             user_id: Set(user.id),
//             module_id: Set(module.id),
//             role: Set(UserRole::Lecturer),
//             ..Default::default()
//         }
//         .insert(&db)
//         .await
//         .expect("assign role");

//         // Check `is_in_role`
//         let is_lecturer = Model::is_in_role(&db, user.id, module.id, "lecturer")
//             .await
//             .expect("check is_in_role");
//         assert!(is_lecturer);

//         let is_tutor = Model::is_in_role(&db, user.id, module.id, "tutor")
//             .await
//             .expect("check is_in_role");
//         assert!(!is_tutor);

//         // Check `get_module_roles`
//         let roles = Model::get_module_roles(&db, user.id)
//             .await
//             .expect("get_module_roles");

//         assert_eq!(roles.len(), 1);
//         let r = &roles[0];
//         assert_eq!(r.module_id, module.id);
//         assert_eq!(r.module_code, "COS999");
//         assert_eq!(r.role, "lecturer");
//     }
// }