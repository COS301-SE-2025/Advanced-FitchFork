use sea_orm::entity::prelude::*;
use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set,
};
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rand::rngs::OsRng;

use crate::models::{
    module::Entity as ModuleEntity,
    user::{self, ActiveModel as UserActiveModel, Entity as UserEntity},
    user_module_role::{Column as RoleColumn, Entity as RoleEntity},
};
use std::str::FromStr;
use crate::models::user_module_role::Role;

/// Represents a user in the `users` table.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, serde::Serialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    /// Primary key ID (auto-incremented).
    #[sea_orm(primary_key)]
    pub id: i64,
    /// Unique student number.
    pub student_number: String,
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
}

/// This enum would define relations if any exist. Currently unused.
#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        panic!("No RelationDef implemented")
    }
}

/// SeaORM hook point for customizing model behavior.
impl ActiveModelBehavior for ActiveModel {}

/// Struct returned by `get_module_roles`, summarizing a user’s role in a module.
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

impl Model {
    /// Creates a new user with hashed password and returns the inserted model.
    ///
    /// # Arguments
    /// * `db` - Database connection reference.
    /// * `student_number` - Unique student number.
    /// * `email` - Email address.
    /// * `password` - Plaintext password to hash.
    /// * `admin` - Whether the user is an admin.
    pub async fn create(
        db: &DatabaseConnection,
        student_number: &str,
        email: &str,
        password: &str,
        admin: bool,
    ) -> Result<Model, DbErr> {
        let hash = Self::hash_password(password);
        let active = UserActiveModel {
            student_number: Set(student_number.to_owned()),
            email: Set(email.to_owned()),
            password_hash: Set(hash),
            admin: Set(admin),
            ..Default::default()
        };
        active.insert(db).await
    }

    /// Fetches a user by student number.
    ///
    /// # Arguments
    /// * `db` - Database connection.
    /// * `student_number` - The student number to look up.
    ///
    /// # Returns
    /// An optional user model if found.
    pub async fn get_by_student_number(
        db: &DatabaseConnection,
        student_number: &str,
    ) -> Result<Option<Model>, DbErr> {
        UserEntity::find()
            .filter(user::Column::StudentNumber.eq(student_number))
            .one(db)
            .await
    }

    /// Verifies user credentials by checking password against stored hash.
    ///
    /// # Arguments
    /// * `db` - Database connection.
    /// * `student_number` - The student number of the user.
    /// * `password` - The plaintext password to verify.
    ///
    /// # Returns
    /// The user model if credentials are valid.
    pub async fn verify_credentials(
        db: &DatabaseConnection,
        student_number: &str,
        password: &str,
    ) -> Result<Option<Model>, DbErr> {
        if let Some(user) = Self::get_by_student_number(db, student_number).await? {
            let parsed = PasswordHash::new(&user.password_hash)
                .map_err(|e| DbErr::Custom(format!("Invalid hash: {}", e)))?;
            if Argon2::default()
                .verify_password(password.as_bytes(), &parsed)
                .is_ok()
            {
                return Ok(Some(user));
            }
        }
        Ok(None)
    }

    /// Retrieves all module roles associated with the user.
    ///
    /// # Arguments
    /// * `db` - Database connection.
    /// * `user_id` - The user ID to query.
    ///
    /// # Returns
    /// A list of module roles with related metadata.
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

    /// Checks whether a user is assigned a specific role in a given module.
    ///
    /// # Arguments
    /// * `db` - Database connection.
    /// * `user_id` - User ID to check.
    /// * `module_id` - Module ID to check against.
    /// * `role` - Role name (e.g. "lecturer", "tutor", "student").
    ///
    /// # Returns
    /// `true` if the role exists, otherwise `false`.
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

    /// Hashes a plaintext password using Argon2 with a generated salt.
    ///
    /// # Arguments
    /// * `password` - The plaintext password.
    ///
    /// # Returns
    /// A hashed password string.
    fn hash_password(password: &str) -> String {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .expect("Failed to hash password")
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::setup_test_db;
    use crate::models::user_module_role::Role as UserRole;

    #[tokio::test]
    async fn test_create_and_get_user() {
        let db = setup_test_db().await;
        let student_number = "u12345678";
        let email = "test@example.com";
        let password = "secret123";

        let _user = Model::create(&db, student_number, email, password, false)
            .await
            .expect("Failed to create user");

        let found = Model::get_by_student_number(&db, student_number)
            .await
            .expect("Failed to query user");

        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.email, email);
        assert_eq!(found.student_number, student_number);
        assert_eq!(found.admin, false);
    }

    #[tokio::test]
    async fn test_verify_credentials_success_and_failure() {
        let db = setup_test_db().await;
        let student_number = "u87654321";
        let email = "auth@example.com";
        let password = "correct_pw";

        Model::create(&db, student_number, email, password, false)
            .await
            .expect("Failed to create user");

        let ok = Model::verify_credentials(&db, student_number, password)
            .await
            .expect("Failed to verify credentials");
        assert!(ok.is_some());

        let bad = Model::verify_credentials(&db, student_number, "wrong_pw")
            .await
            .expect("Failed to verify wrong credentials");
        assert!(bad.is_none());
    }

    #[tokio::test]
    async fn test_is_in_role_and_get_module_roles() {
        use crate::models::{
            module::{ActiveModel as ModuleActiveModel},
            user_module_role::ActiveModel as RoleActiveModel,
        };

        let db = setup_test_db().await;

        // Create user
        let user = Model::create(&db, "u00001111", "modrole@example.com", "pw", false)
            .await
            .expect("create user");

        // Create module
        let module = ModuleActiveModel {
            code: Set("COS999".into()),
            year: Set(2025),
            description: Set(Some("Test Module".into())),
            credits: Set(15),
            ..Default::default()
        }
        .insert(&db)
        .await
        .expect("create module");

        // Assign user to module with role
        RoleActiveModel {
            user_id: Set(user.id),
            module_id: Set(module.id),
            role: Set(UserRole::Lecturer),
            ..Default::default()
        }
        .insert(&db)
        .await
        .expect("assign role");

        // Check `is_in_role`
        let is_lecturer = Model::is_in_role(&db, user.id, module.id, "lecturer")
            .await
            .expect("check is_in_role");
        assert!(is_lecturer);

        let is_tutor = Model::is_in_role(&db, user.id, module.id, "tutor")
            .await
            .expect("check is_in_role");
        assert!(!is_tutor);

        // Check `get_module_roles`
        let roles = Model::get_module_roles(&db, user.id)
            .await
            .expect("get_module_roles");

        assert_eq!(roles.len(), 1);
        let r = &roles[0];
        assert_eq!(r.module_id, module.id);
        assert_eq!(r.module_code, "COS999");
        assert_eq!(r.role, "lecturer");
    }
}
