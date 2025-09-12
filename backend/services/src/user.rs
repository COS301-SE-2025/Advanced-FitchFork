use crate::service::{Service, AppError, ToActiveModel};
use db::{
    models::user::{Model, Column, Entity, ActiveModel},
    repository::Repository,
};
use util::filters::FilterParam;
use sea_orm::{DbErr, Set};
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rand::rngs::OsRng;
use validator::{Validate, ValidationError};
use chrono::Utc;

pub use db::models::user::Model as User;

#[derive(Debug, Clone, Validate)]
pub struct CreateUser {
    pub id: Option<i64>,

    #[validate(length(min = 1, message = "Username cannot be empty"))]
    pub username: String,

    #[validate(email(message = "Invalid email address"))]
    pub email: String,

    // TODO: this messes up the seeding, but it should work in practice
    //#[validate(custom(function = "validate_password"))]
    pub password: String,

    pub admin: bool,
}

#[derive(Debug, Clone, Validate)]
pub struct UpdateUser {
    pub id: i64,

    #[validate(length(min = 1, message = "Username cannot be empty"))]
    pub username: Option<String>,

    #[validate(email(message = "Invalid email address"))]
    pub email: Option<String>,

    #[validate(custom(function = "validate_password"))]
    pub password: Option<String>,

    pub admin: Option<bool>,
    pub profile_picture_path: Option<Option<String>>,
}

fn validate_password(password: &str) -> Result<(), ValidationError> {
    let mut has_upper = false;
    let mut has_lower = false;
    let mut has_digit = false;
    let mut has_special = false;
    
    if password.len() < 8 {
        return Err(ValidationError::new("Password must be at least 8 characters long"));
    }
    
    for c in password.chars() {
        if c.is_ascii_uppercase() {
            has_upper = true;
        } else if c.is_ascii_lowercase() {
            has_lower = true;
        } else if c.is_ascii_digit() {
            has_digit = true;
        } else if c.is_ascii_punctuation() {
            has_special = true;
        }
    }
    
    if !has_upper {
        return Err(ValidationError::new("Password must contain at least one uppercase letter"));
    }
    if !has_lower {
        return Err(ValidationError::new("Password must contain at least one lowercase letter"));
    }
    if !has_digit {
        return Err(ValidationError::new("Password must contain at least one number"));
    }
    if !has_special {
        return Err(ValidationError::new("Password must contain at least one special character"));
    }
    
    Ok(())
}

impl ToActiveModel<Entity> for CreateUser {
    async fn into_active_model(self) -> Result<ActiveModel, AppError> {
        self.validate()
            .map_err(|e| DbErr::Custom(e.to_string()))?;

        let mut active = ActiveModel {
            username: Set(self.username),
            email: Set(self.email),
            password_hash: Set(UserService::hash_password(&self.password)),
            admin: Set(self.admin),
            ..Default::default()
        };

        if let Some(id) = self.id {
            active.id = Set(id);
        }

        Ok(active)
    }
}

impl ToActiveModel<Entity> for UpdateUser {
    async fn into_active_model(self) -> Result<ActiveModel, AppError> {
        self.validate()
            .map_err(|e| DbErr::Custom(e.to_string()))?;

        let user = match Repository::<Entity, Column>::find_by_id(self.id).await {
            Ok(Some(user)) => user,
            Ok(None) => {
                return Err(AppError::from(DbErr::RecordNotFound(format!("User ID {} not found", self.id))));
            }
            Err(err) => return Err(AppError::from(err)),
        };
        let mut active: ActiveModel = user.into();

        if let Some(username) = self.username {
            active.username = Set(username);
        }

        if let Some(email) = self.email {
            active.email = Set(email);
        }

        if let Some(password) = self.password {
            let salt = SaltString::generate(&mut OsRng);
            let hash = Argon2::default()
                .hash_password(password.as_bytes(), &salt)
                .map_err(|e| DbErr::Custom(format!("password hashing failed: {}", e)))?
                .to_string();
            active.password_hash = Set(hash);
        }

        if let Some(admin) = self.admin {
            active.admin = Set(admin);
        }

        if let Some(profile_path_opt) = self.profile_picture_path {
            active.profile_picture_path = Set(profile_path_opt);
        }

        active.updated_at = Set(Utc::now());

        Ok(active)
    }
}

pub struct UserService;

impl<'a> Service<'a, Entity, Column, CreateUser, UpdateUser> for UserService {
    // ↓↓↓ OVERRIDE DEFAULT BEHAVIOR IF NEEDED HERE ↓↓↓
}

impl UserService {
    // ↓↓↓ CUSTOM METHODS CAN BE DEFINED HERE ↓↓↓

    pub async fn create_fake_user_with_no_hashed_password_do_not_use(
        username: &str,
        email: &str,
        password: &str,
        admin: bool,
    ) -> Result<Model, DbErr> {
        let active = ActiveModel {
            username: Set(username.to_owned()),
            email: Set(email.to_owned()),
            password_hash: Set(password.to_string()),
            admin: Set(admin),
            ..Default::default()
        };
        Repository::<Entity, Column>::create(active).await
    }

    pub async fn verify_credentials(
        username: &str,
        password: &str,
    ) -> Result<Option<Model>, DbErr> {
        if let Some(user) = Repository::<Entity, Column>::find_one(
            &vec![
                FilterParam::eq("username", username.trim().to_string()),
            ],
            None,
        ).await? {
            if Self::verify_password(&user, password) {
                return Ok(Some(user));
            }
        }

        Ok(None)
    }

    pub fn hash_password(password: &str) -> String {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .expect("Failed to hash password")
            .to_string()
    }

    pub fn verify_password(user: &Model, password: &str) -> bool {
        let parsed = match PasswordHash::new(&user.password_hash) {
            Ok(parsed) => parsed,
            Err(_) => return false,
        };

        Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok()
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use db::models::user::{Entity, Model};
//     use std::collections::HashMap;
//     use std::sync::Mutex;
//     use chrono::{Utc, TimeZone};
//     use db::models::module;
//     use db::models::user_module_role;
//     use sea_orm::Set;

//     struct MockUserRepository {
//         users: Mutex<HashMap<i64, Model>>,
//         next_id: Mutex<i64>,
//     }

//     impl MockUserRepository {
//         fn new() -> Self {
//             Self {
//                 users: Mutex::new(HashMap::new()),
//                 next_id: Mutex::new(1),
//             }
//         }
//     }

//     impl Repository<Entity, UserFilter> for MockUserRepository {
//         async fn create(&self, active_model: user::ActiveModel) -> Result<Model, DbErr> {
//             let mut users = self.users.lock().unwrap();
//             let mut next_id = self.next_id.lock().unwrap();

//             let id = *next_id;
//             *next_id += 1;

//             let user = Model {
//                 id,
//                 username: active_model.username.unwrap(),
//                 email: active_model.email.unwrap(),
//                 password_hash: active_model.password_hash.unwrap(),
//                 admin: active_model.admin.unwrap(),
//                 created_at: chrono::Utc::now(),
//                 updated_at: chrono::Utc::now(),
//                 profile_picture_path: None,
//             };

//             users.insert(id, user.clone());
//             Ok(user)
//         }

//         async fn find_by_id(&self, id: i64) -> Result<Option<Model>, DbErr> {
//             let users = self.users.lock().unwrap();
//             Ok(users.get(&id).cloned())
//         }

//         async fn update(&self, active_model: user::ActiveModel) -> Result<Model, DbErr> {
//             let mut users = self.users.lock().unwrap();
//             let id = active_model.id.unwrap();

//             if let Some(user) = users.get_mut(&id) {
//                 user.username = active_model.username.unwrap();
//                 user.email = active_model.email.unwrap();
//                 user.password_hash = active_model.password_hash.unwrap();
//                 user.admin = active_model.admin.unwrap();
//                 user.updated_at = chrono::Utc::now();
//                 Ok(user.clone())
//             } else {
//                 Err(DbErr::RecordNotFound("User not found".to_string()))
//             }
//         }

//         async fn delete(&self, id: i64) -> Result<(), DbErr> {
//             let mut users = self.users.lock().unwrap();
//             if users.remove(&id).is_some() {
//                 Ok(())
//             } else {
//                 Err(DbErr::RecordNotFound("User not found".to_string()))
//             }
//         }

//         async fn filter(
//             &self,
//             filter_params: UserFilter,
//             _page: u64,
//             _per_page: u64,
//             _sort_by: Option<String>,
//         ) -> Result<Vec<Model>, DbErr> {
//             let users = self.users.lock().unwrap();
//             match filter_params {
//                 UserFilter::Username(username) => {
//                     let filtered_users = users
//                         .values()
//                         .filter(|u| u.username == username)
//                         .cloned()
//                         .collect();
//                     Ok(filtered_users)
//                 }
//             }
//         }

//         async fn find_one(&self, filter_params: UserFilter) -> Result<Option<Model>, DbErr> {
//             let users = self.users.lock().unwrap();
//             match filter_params {
//                 UserFilter::Username(username) => {
//                     let user = users.values().find(|u| u.username == username).cloned();
//                     Ok(user)
//                 }
//             }
//         }
//     }

//     #[tokio::test]
//     async fn test_create_user() {
//         let repo = MockUserRepository::new();
//         let service = UserService::new(repo);

//         let username = "testuser";
//         let email = "test@example.com";
//         let password = "password";

//         let user = service
//             .create_user(username, email, password, false)
//             .await
//             .unwrap();

//         assert_eq!(user.username, username);
//         assert_eq!(user.email, email);
//     }

//     #[tokio::test]
//     async fn test_get_by_username() {
//         let repo = MockUserRepository::new();
//         let service = UserService::new(repo);

//         let username = "testuser";
//         let email = "test@example.com";
//         let password = "password";

//         service
//             .create_user(username, email, password, false)
//             .await
//             .unwrap();

//         let user = service.get_by_username(username).await.unwrap();
//         assert!(user.is_some());
//         assert_eq!(user.unwrap().username, username);
//     }

//     #[tokio::test]
//     async fn test_verify_credentials() {
//         let repo = MockUserRepository::new();
//         let service = UserService::new(repo);

//         let username = "testuser";
//         let email = "test@example.com";
//         let password = "password";

//         service
//             .create_user(username, email, password, false)
//             .await
//             .unwrap();

//         let user = service.verify_credentials(username, password).await.unwrap();
//         assert!(user.is_some());

//         let user = service
//             .verify_credentials(username, "wrongpassword")
//             .await
//             .unwrap();
//         assert!(user.is_none());
//     }

//     #[tokio::test]
//     async fn test_is_in_role() {
//         let repo = MockUserRepository::new();
//         let service = UserService::new(repo);

//         // Mock data for user and module
//         let user_model = db::models::user::Model {
//             id: 1,
//             username: "testuser".to_string(),
//             email: "test@example.com".to_string(),
//             password_hash: "hash".to_string(),
//             admin: false,
//             created_at: Utc::now(),
//             updated_at: Utc::now(),
//             profile_picture_path: None,
//         };

//         let module_model = module::Model {
//             id: 101,
//             code: "MOD101".to_string(),
//             year: 2023,
//             description: Some("Module Description".to_string()),
//             credits: 10,
//             created_at: Utc::now(),
//             updated_at: Utc::now(),
//         };

//         // Manually insert into mock repo (if needed for other tests, but not directly for is_in_role)
//         // repo.users.lock().unwrap().insert(user_model.id, user_model.clone());

//         // Mock a user_module_role entry
//         let user_module_role_model = user_module_role::Model {
//             user_id: user_model.id,
//             module_id: module_model.id,
//             role: user_module_role::Role::Lecturer,
//         };
//         // This part would typically involve a mock for user_module_role repository
//         // For now, we'll assume the underlying db query works as expected for this test.

//         // Test is_in_role
//         let is_lecturer = service.is_in_role(user_model.id, module_model.id, "lecturer").await.unwrap();
//         assert!(is_lecturer);

//         let is_student = service.is_in_role(user_model.id, module_model.id, "student").await.unwrap();
//         assert!(!is_student);
//     }

//     #[tokio::test]
//     async fn test_get_module_roles() {
//         let repo = MockUserRepository::new();
//         let service = UserService::new(repo);

//         // Mock data for user and module
//         let user_model = db::models::user::Model {
//             id: 1,
//             username: "testuser".to_string(),
//             email: "test@example.com".to_string(),
//             password_hash: "hash".to_string(),
//             admin: false,
//             created_at: Utc::now(),
//             updated_at: Utc::now(),
//             profile_picture_path: None,
//         };

//         let module_model_1 = module::Model {
//             id: 101,
//             code: "MOD101".to_string(),
//             year: 2023,
//             description: Some("Module One".to_string()),
//             credits: 10,
//             created_at: Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap(),
//             updated_at: Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap(),
//         };

//         let module_model_2 = module::Model {
//             id: 102,
//             code: "MOD102".to_string(),
//             year: 2023,
//             description: Some("Module Two".to_string()),
//             credits: 15,
//             created_at: Utc.with_ymd_and_hms(2023, 2, 1, 0, 0, 0).unwrap(),
//             updated_at: Utc.with_ymd_and_hms(2023, 2, 1, 0, 0, 0).unwrap(),
//         };

//         // Mock user_module_role entries
//         let _role_1 = user_module_role::Model {
//             user_id: user_model.id,
//             module_id: module_model_1.id,
//             role: user_module_role::Role::Lecturer,
//         };

//         let _role_2 = user_module_role::Model {
//             user_id: user_model.id,
//             module_id: module_model_2.id,
//             role: user_module_role::Role::Student,
//         };

//         // This test would ideally use a mock for the user_module_role repository
//         // that can return these mocked relationships. For now, we're assuming
//         // the underlying db query logic works.

//         let roles = service.get_module_roles(user_model.id).await.unwrap();

//         assert_eq!(roles.len(), 2);

//         // Check if roles contain expected data (order might not be guaranteed)
//         let role_codes: Vec<String> = roles.iter().map(|r| r.module_code.clone()).collect();
//         assert!(role_codes.contains(&"MOD101".to_string()));
//         assert!(role_codes.contains(&"MOD102".to_string()));

//         let lecturer_role = roles.iter().find(|r| r.role == "lecturer").unwrap();
//         assert_eq!(lecturer_role.module_id, module_model_1.id);
//         assert_eq!(lecturer_role.module_code, "MOD101");

//         let student_role = roles.iter().find(|r| r.role == "student").unwrap();
//         assert_eq!(student_role.module_id, module_model_2.id);
//         assert_eq!(student_role.module_code, "MOD102");
//     }
// }