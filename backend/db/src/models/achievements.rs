use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveModelTrait, DatabaseConnection, DbErr, EntityTrait, Set};
use serde::{Deserialize, Serialize};

/// Represents an achievement in the `achievements` table.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "achievements")]
pub struct Model {
    /// Primary key ID (auto-incremented).
    #[sea_orm(primary_key)]
    pub id: i64,
    /// Unique name of the achievement.
    #[sea_orm(unique)]
    pub name: String,
    /// Description of the achievement.
    pub description: String,
    /// Whether this is a positive (true) or negative (false) achievement.
    pub is_positive: bool,
    /// Number of levels for this achievement.
    pub levels: i32,
    /// Key referencing condition in achievements.json.
    pub condition_id: String,
    /// Timestamp when the achievement was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp when the achievement was last updated.
    pub updated_at: DateTime<Utc>,
}

/// Defines relationships between `achievements` and other tables.
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    /// Link to achievement progress records.
    #[sea_orm(has_many = "super::achievement_progress::Entity")]
    AchievementProgress,
}

impl Related<super::achievement_progress::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AchievementProgress.def()
    }
}

/// SeaORM hook point for customizing model behavior.
impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// Creates a new achievement and returns the inserted model.
    ///
    /// # Arguments
    /// * `db` - Database connection reference.
    /// * `name` - Unique name of the achievement.
    /// * `description` - Description of the achievement.
    /// * `is_positive` - Whether the achievement is positive or negative.
    /// * `levels` - Number of levels for the achievement.
    /// * `condition_id` - Key referencing condition in achievements.json.
    pub async fn create(
        db: &DatabaseConnection,
        name: &str,
        description: &str,
        is_positive: bool,
        levels: i32,
        condition_id: &str,
    ) -> Result<Model, DbErr> {
        let active = ActiveModel {
            name: Set(name.to_owned()),
            description: Set(description.to_owned()),
            is_positive: Set(is_positive),
            levels: Set(levels),
            condition_id: Set(condition_id.to_owned()),
            ..Default::default()
        };
        active.insert(db).await
    }

    /// Fetches an achievement by name.
    ///
    /// # Arguments
    /// * `db` - Database connection.
    /// * `name` - The name of the achievement to look up.
    ///
    /// # Returns
    /// An optional achievement model if found.
    pub async fn get_by_name(
        db: &DatabaseConnection,
        name: &str,
    ) -> Result<Option<Model>, DbErr> {
        Entity::find()
            .filter(Column::Name.eq(name))
            .one(db)
            .await
    }

    /// Fetches an achievement by condition ID.
    ///
    /// # Arguments
    /// * `db` - Database connection.
    /// * `condition_id` - The condition ID to look up.
    ///
    /// # Returns
    /// An optional achievement model if found.
    pub async fn get_by_condition_id(
        db: &DatabaseConnection,
        condition_id: &str,
    ) -> Result<Option<Model>, DbErr> {
        Entity::find()
            .filter(Column::ConditionId.eq(condition_id))
            .one(db)
            .await
    }

    /// Fetches all positive achievements.
    ///
    /// # Arguments
    /// * `db` - Database connection.
    ///
    /// # Returns
    /// A vector of positive achievement models.
    pub async fn get_positive_achievements(
        db: &DatabaseConnection,
    ) -> Result<Vec<Model>, DbErr> {
        Entity::find()
            .filter(Column::IsPositive.eq(true))
            .all(db)
            .await
    }

    /// Fetches all negative achievements.
    ///
    /// # Arguments
    /// * `db` - Database connection.
    ///
    /// # Returns
    /// A vector of negative achievement models.
    pub async fn get_negative_achievements(
        db: &DatabaseConnection,
    ) -> Result<Vec<Model>, DbErr> {
        Entity::find()
            .filter(Column::IsPositive.eq(false))
            .all(db)
            .await
    }

    /// Updates an achievement's description.
    ///
    /// # Arguments
    /// * `db` - Database connection.
    /// * `id` - Achievement ID to update.
    /// * `description` - New description.
    ///
    /// # Returns
    /// The updated achievement model.
    pub async fn update_description(
        db: &DatabaseConnection,
        id: i64,
        description: &str,
    ) -> Result<Model, DbErr> {
        let mut active: ActiveModel = Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("Achievement not found".to_string()))?
            .into();

        active.description = Set(description.to_owned());
        active.update(db).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::setup_test_db;

    #[tokio::test]
    async fn test_create_and_get_achievement() {
        let db = setup_test_db().await;
        let name = "First Submission";
        let description = "Submit your first assignment";
        let is_positive = true;
        let levels = 1;
        let condition_id = "first_submission";

        let achievement = Model::create(&db, name, description, is_positive, levels, condition_id)
            .await
            .expect("Failed to create achievement");

        assert_eq!(achievement.name, name);
        assert_eq!(achievement.description, description);
        assert_eq!(achievement.is_positive, is_positive);
        assert_eq!(achievement.levels, levels);
        assert_eq!(achievement.condition_id, condition_id);

        let found = Model::get_by_name(&db, name)
            .await
            .expect("Failed to query achievement");

        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.name, name);
        assert_eq!(found.description, description);
    }

    #[tokio::test]
    async fn test_get_by_condition_id() {
        let db = setup_test_db().await;
        let name = "Perfect Score";
        let description = "Achieve 100% on an assignment";
        let condition_id = "perfect_score";

        Model::create(&db, name, description, true, 3, condition_id)
            .await
            .expect("Failed to create achievement");

        let found = Model::get_by_condition_id(&db, condition_id)
            .await
            .expect("Failed to query achievement by condition ID");

        assert!(found.is_some());
        assert_eq!(found.unwrap().name, name);
    }

    #[tokio::test]
    async fn test_get_positive_and_negative_achievements() {
        let db = setup_test_db().await;

        // Create positive achievement
        Model::create(&db, "Early Bird", "Submit before deadline", true, 2, "early_bird")
            .await
            .expect("Failed to create positive achievement");

        // Create negative achievement
        Model::create(&db, "Late Submission", "Submit after deadline", false, 3, "late_submission")
            .await
            .expect("Failed to create negative achievement");

        let positive = Model::get_positive_achievements(&db)
            .await
            .expect("Failed to get positive achievements");
        
        let negative = Model::get_negative_achievements(&db)
            .await
            .expect("Failed to get negative achievements");

        assert_eq!(positive.len(), 1);
        assert_eq!(negative.len(), 1);
        assert_eq!(positive[0].name, "Early Bird");
        assert_eq!(negative[0].name, "Late Submission");
    }

    #[tokio::test]
    async fn test_update_description() {
        let db = setup_test_db().await;
        let name = "Test Achievement";
        let original_description = "Original description";
        let new_description = "Updated description";

        let achievement = Model::create(&db, name, original_description, true, 1, "test_achievement")
            .await
            .expect("Failed to create achievement");

        let updated = Model::update_description(&db, achievement.id, new_description)
            .await
            .expect("Failed to update description");

        assert_eq!(updated.description, new_description);
        assert_eq!(updated.id, achievement.id);
    }
}