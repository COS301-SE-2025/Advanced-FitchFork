use sea_orm::entity::prelude::*;
use sea_orm::{DatabaseConnection, EntityTrait, ActiveModelTrait, Set};
use chrono::{DateTime, Utc};
use log::{info, warn};
use util::paths::module_dir;
use std::fs;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "modules")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub code: String,
    pub year: i32,
    pub description: Option<String>,
    pub credits: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        has_many = "super::user_module_role::Entity",
        from = "Column::Id",
        to = "super::user_module_role::Column::ModuleId"
    )]
    UserModuleRole,
}

impl Related<super::user_module_role::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserModuleRole.def()
    }

    fn via() -> Option<RelationDef> {
        None
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// Create a new module record in the database.
    ///
    /// # Arguments
    /// * `db` - Reference to the database connection.
    /// * `code` - The module code (e.g., "COS301").
    /// * `year` - The academic year.
    /// * `description` - Optional module description.
    /// * `credits` - Credit value for the module.
    ///
    /// # Returns
    /// A fully populated `Model` after insertion.
    pub async fn create<C>(
      db: &C,
        code: &str,
        year: i32,
        description: Option<&str>,
        credits: i32,
    ) -> Result<Self, DbErr>
        where
        C: ConnectionTrait, 
    {
        let active = ActiveModel {
            code: Set(code.to_owned()),
            year: Set(year),
            description: Set(description.map(|d| d.to_owned())),
            credits: Set(credits),
            ..Default::default()
        };

        active.insert(db).await
    }

    /// Deletes this module, its assignments, and associated files/folders.
    pub async fn delete(self, db: &DatabaseConnection) -> Result<(), DbErr> {
        // Step 1: Let DB cascade delete assignments
        info!("Deleting module {} and cascading assignments", self.id);

        // Step 2: Remove module-level folder
        let dir = module_dir(self.id);

        if dir.exists() {
            match fs::remove_dir_all(&dir) {
                Ok(_) => info!("Deleted module directory {}", dir.display()),
                Err(e) => warn!("Failed to delete module directory {}: {}", dir.display(), e),
            }
        } else {
            warn!("Expected module directory {} does not exist", dir.display());
        }


        // Step 3: Delete the module
        Entity::delete_by_id(self.id).exec(db).await?;
        info!("Deleted module {}", self.id);

        Ok(())
    }

    /// Edit a module by ID and return the updated model.
    ///
    /// All fields will be updated, and `updated_at` will be auto-set by the DB.
    pub async fn edit(
        db: &DatabaseConnection,
        id: i64,
        code: &str,
        year: i32,
        description: Option<&str>,
        credits: i32,
    ) -> Result<Self, DbErr> {
        let Some(module) = Entity::find_by_id(id).one(db).await? else {
            return Err(DbErr::RecordNotFound(format!("Module ID {} not found", id)));
        };

        let mut active: ActiveModel = module.into();
        active.code = Set(code.to_owned());
        active.year = Set(year);
        active.description = Set(description.map(|d| d.to_owned()));
        active.credits = Set(credits);

        active.update(db).await
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::setup_test_db;

    #[tokio::test]
    async fn test_create_module() {
        let db = setup_test_db().await;

        let code = "COS301";
        let year = 2025;
        let description = Some("Software Engineering");
        let credits = 16;

        let module = Model::create(&db, code, year, description, credits)
            .await
            .expect("Failed to create module");

        assert_eq!(module.code, code);
        assert_eq!(module.year, year);
        assert_eq!(module.description.as_deref(), description);
        assert_eq!(module.credits, credits);
    }

    #[tokio::test]
    async fn test_edit_module() {
        let db = setup_test_db().await;

        let initial = Model::create(&db, "COS132", 2024, Some("Old Desc"), 12)
            .await
            .unwrap();

        let updated = Model::edit(&db, initial.id, "COS133", 2025, Some("New Desc"), 14)
            .await
            .expect("Failed to edit module");

        assert_eq!(updated.id, initial.id);
        assert_eq!(updated.code, "COS133");
        assert_eq!(updated.year, 2025);
        assert_eq!(updated.description.as_deref(), Some("New Desc"));
        assert_eq!(updated.credits, 14);
    }
}
