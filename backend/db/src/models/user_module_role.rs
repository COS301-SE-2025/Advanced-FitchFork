use sea_orm::entity::prelude::*;
use sea_orm::EntityTrait;
use strum::{Display, EnumString};
use serde::{Deserialize, Serialize};

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
#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum, Display, EnumString, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
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

impl ActiveModelBehavior for ActiveModel {}
impl Model {}