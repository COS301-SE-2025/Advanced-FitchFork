use sea_orm::entity::prelude::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, DeleteResult};
use strum_macros::{Display, EnumString};

/// The central table for user-module-role relationships.
/// Replaces old `module_lecturers`, `module_tutors`, and `module_students`.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "user_module_roles")]
pub struct Model {
    /// User ID (foreign key to `users`)
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_id: i64,

    /// Module ID (foreign key to `modules`)
    #[sea_orm(primary_key, auto_increment = false)]
    pub module_id: i64,

    /// Role type: Lecturer, Tutor, or Student
    pub role: Role,
}

/// Enum representing user roles within a module.
/// Backed by a `user_module_role_type` enum in the database.
#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum, Display, EnumString)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "user_module_role_type")]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
pub enum Role {
    #[sea_orm(string_value = "lecturer")]
    Lecturer,

    #[sea_orm(string_value = "assistant_lecturer")]
    AssistantLecturer,

    #[sea_orm(string_value = "tutor")]
    Tutor,

    #[sea_orm(string_value = "student")]
    Student,
}

/// Defines relationships for foreign key joins.
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    /// Belongs to a user
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    User,

    /// Belongs to a module
    #[sea_orm(
        belongs_to = "super::module::Entity",
        from = "Column::ModuleId",
        to = "super::module::Column::Id"
    )]
    Module,
}

impl Related<super::module::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Module.def()
    }

    fn via() -> Option<RelationDef> {
        None
    }
}

/// Enables customization of insert/update behavior for the ActiveModel.
impl ActiveModelBehavior for ActiveModel {}

/// Additional CRUD logic and utilities for `user_module_roles`.
impl Model {
    /// Assign a user to a module with a specific role.
    pub async fn assign_user_to_module(
        db: &DatabaseConnection,
        user_id: i64,
        module_id: i64,
        role: Role,
    ) -> Result<Self, DbErr> {
        let active = ActiveModel {
            user_id: Set(user_id),
            module_id: Set(module_id),
            role: Set(role),
        };
        active.insert(db).await
    }

    /// Remove a user from a specific module.
    pub async fn remove_user_from_module(
        db: &DatabaseConnection,
        user_id: i64,
        module_id: i64,
    ) -> Result<DeleteResult, DbErr> {
        Entity::delete_many()
            .filter(Column::UserId.eq(user_id))
            .filter(Column::ModuleId.eq(module_id))
            .exec(db)
            .await
    }

    /// Get all role assignments.
    pub async fn get_all(db: &DatabaseConnection) -> Result<Vec<Self>, DbErr> {
        Entity::find().all(db).await
    }

    /// Get all users assigned to a module under a specific role.
    pub async fn get_users_by_module_role(
        db: &DatabaseConnection,
        module_id: i32,
        role: Role,
    ) -> Result<Vec<Self>, DbErr> {
        Entity::find()
            .filter(Column::ModuleId.eq(module_id))
            .filter(Column::Role.eq(role))
            .all(db)
            .await
    }

    /// Get all modules a user is assigned to under a specific role.
    pub async fn get_modules_by_user_role(
        db: &DatabaseConnection,
        user_id: i32,
        role: Role,
    ) -> Result<Vec<Self>, DbErr> {
        Entity::find()
            .filter(Column::UserId.eq(user_id))
            .filter(Column::Role.eq(role))
            .all(db)
            .await
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{module, user};
    use crate::test_utils::setup_test_db;

    #[tokio::test]
    async fn test_assign_and_get_user_module_role() {
        let db = setup_test_db().await;

        // Insert fake user and module records manually
        user::ActiveModel {
            id: Set(1),
            username: Set("s123456".to_string()),
            email: Set("a@example.com".to_string()),
            password_hash: Set("hash".to_string()),
            admin: Set(false),
            ..Default::default()
        }
        .insert(&db)
        .await
        .unwrap();

        module::ActiveModel {
            id: Set(1),
            code: Set("COS301".to_string()),
            year: Set(2025),
            description: Set(Some("Capstone".to_string())),
            credits: Set(30),
            ..Default::default()
        }.insert(&db).await.unwrap();

        let assigned = Model::assign_user_to_module(&db, 1, 1, Role::Lecturer).await.unwrap();
        assert_eq!(assigned.user_id, 1);
        assert_eq!(assigned.module_id, 1);
        assert_eq!(assigned.role, Role::Lecturer);

        let fetched = Model::get_users_by_module_role(&db, 1, Role::Lecturer).await.unwrap();
        assert_eq!(fetched.len(), 1);
        assert_eq!(fetched[0].user_id, 1);
    }

    #[tokio::test]
    async fn test_remove_user_module_role() {
        let db = setup_test_db().await;

        // Create foreign keys
        user::ActiveModel {
            id: Set(2),
            username: Set("s654321".to_string()),
            email: Set("b@example.com".to_string()),
            password_hash: Set("hash".to_string()),
            admin: Set(false),
            ..Default::default()
        }
        .insert(&db)
        .await
        .unwrap();

        module::ActiveModel {
            id: Set(2),
            code: Set("COS333".to_string()),
            year: Set(2025),
            description: Set(Some("Networks".to_string())),
            credits: Set(16),
            ..Default::default()
        }.insert(&db).await.unwrap();

        Model::assign_user_to_module(&db, 2, 2, Role::Tutor).await.unwrap();
        let result = Model::remove_user_from_module(&db, 2, 2).await.unwrap();
        assert_eq!(result.rows_affected, 1);
    }
}
