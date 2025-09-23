use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use chrono::{DateTime, Utc};
use rand::rngs::OsRng;
use sea_orm::entity::prelude::*;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set,
};

use crate::models::user_module_role::Role;
use crate::models::{
    module::Entity as ModuleEntity,
    user::{self, ActiveModel as UserActiveModel, Entity as UserEntity},
    user_module_role::{Column as RoleColumn, Entity as RoleEntity},
};
use std::str::FromStr;

/// Represents a user in the `users` table.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, serde::Serialize)]
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

/// SeaORM hook point for customizing model behavior.
impl ActiveModelBehavior for ActiveModel {}

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

impl Model {
    /// Creates a new user with hashed password and returns the inserted model.
    ///
    /// # Arguments
    /// * `db` - Database connection reference.
    /// * `username` - Unique student number.
    /// * `email` - Email address.
    /// * `password` - Plaintext password to hash.
    /// * `admin` - Whether the user is an admin.
    pub async fn create(
        db: &DatabaseConnection,
        username: &str,
        email: &str,
        password: &str,
        admin: bool,
    ) -> Result<Model, DbErr> {
        let hash = Self::hash_password(password);
        let active = UserActiveModel {
            username: Set(username.to_owned()),
            email: Set(email.to_owned()),
            password_hash: Set(hash),
            admin: Set(admin),
            ..Default::default()
        };
        active.insert(db).await
    }

    /// Creates a new user with no hashed password and returns the inserted model.
    /// NOTE: FOR SEEDING PURPOSES ONLY
    /// DO NOT USE
    ///
    /// # Arguments
    /// * `db` - Database connection reference.
    /// * `username` - Unique student number.
    /// * `email` - Email address.
    /// * `password` - Plaintext password to hash.
    /// * `admin` - Whether the user is an admin.
    pub async fn create_fake_user_with_no_hashed_password_do_not_use(
        db: &DatabaseConnection,
        username: &str,
        email: &str,
        password: &str,
        admin: bool,
    ) -> Result<Model, DbErr> {
        let hash = password;
        let active = UserActiveModel {
            username: Set(username.to_owned()),
            email: Set(email.to_owned()),
            password_hash: Set(hash.to_string()),
            admin: Set(admin),
            ..Default::default()
        };
        active.insert(db).await
    }

    /// Fetches a user by student number.
    ///
    /// # Arguments
    /// * `db` - Database connection.
    /// * `username` - The student number to look up.
    ///
    /// # Returns
    /// An optional user model if found.
    pub async fn get_by_username(
        db: &DatabaseConnection,
        username: &str,
    ) -> Result<Option<Model>, DbErr> {
        UserEntity::find()
            .filter(user::Column::Username.eq(username))
            .one(db)
            .await
    }

    /// Verifies user credentials by checking password against stored hash.
    ///
    /// # Arguments
    /// * `db` - Database connection.
    /// * `username` - The student number of the user.
    /// * `password` - The plaintext password to verify.
    ///
    /// # Returns
    /// The user model if credentials are valid.
    pub async fn verify_credentials(
        db: &DatabaseConnection,
        username: &str,
        password: &str,
    ) -> Result<Option<Model>, DbErr> {
        let username = username.trim();

        if let Some(user) = Self::get_by_username(db, username).await? {
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

    /// Retrieves all email addresses of users assigned to a specific module.
    ///
    /// # Arguments
    /// * `db` - Database connection.
    /// * `module_id` - The module ID to query.
    ///
    /// # Returns
    /// A list of email addresses.
    pub async fn get_emails_by_module_id(db: &DatabaseConnection, module_id: i64) -> Vec<String> {
        let roles = RoleEntity::find()
            .filter(RoleColumn::ModuleId.eq(module_id))
            .all(db)
            .await
            .unwrap_or_default();

        let mut emails: Vec<String> = Vec::new();
        for role in roles {
            if let Some(user) = UserEntity::find_by_id(role.user_id)
                .one(db)
                .await
                .unwrap_or(None)
            {
                emails.push(user.email.clone());
            }
        }
        emails
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
        let parsed_role = Role::from_str(role)
            .map_err(|_| DbErr::Custom(format!("Invalid role string: '{}'", role)))?;

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
    pub fn hash_password(password: &str) -> String {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .expect("Failed to hash password")
            .to_string()
    }

    /// Verifies a plaintext password against the stored hash
    pub fn verify_password(&self, password: &str) -> bool {
        let parsed = match PasswordHash::new(&self.password_hash) {
            Ok(parsed) => parsed,
            Err(_) => return false,
        };

        Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::user_module_role::Role as UserRole;
    use crate::test_utils::setup_test_db;

    #[tokio::test]
    async fn test_create_and_get_user() {
        let db = setup_test_db().await;
        let username = "u12345678";
        let email = "test@example.com";
        let password = "secret123";

        let _user = Model::create(&db, username, email, password, false)
            .await
            .expect("Failed to create user");

        let found = Model::get_by_username(&db, username)
            .await
            .expect("Failed to query user");

        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.email, email);
        assert_eq!(found.username, username);
        assert_eq!(found.admin, false);
    }

    #[tokio::test]
    async fn test_verify_credentials_success_and_failure() {
        let db = setup_test_db().await;
        let username = "u87654321";
        let email = "auth@example.com";
        let password = "correct_pw";

        Model::create(&db, username, email, password, false)
            .await
            .expect("Failed to create user");

        let ok = Model::verify_credentials(&db, username, password)
            .await
            .expect("Failed to verify credentials");
        assert!(ok.is_some());

        let bad = Model::verify_credentials(&db, username, "wrong_pw")
            .await
            .expect("Failed to verify wrong credentials");
        assert!(bad.is_none());
    }

    #[tokio::test]
    async fn test_is_in_role_and_get_module_roles() {
        use crate::models::{
            module::ActiveModel as ModuleActiveModel,
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
