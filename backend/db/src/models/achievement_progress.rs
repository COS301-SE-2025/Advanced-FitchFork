use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveModelTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set, QueryOrder, QuerySelect};
use serde::{Deserialize, Serialize};

/// Represents a user's progress toward a specific achievement in the `achievement_progress` table.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "achievement_progress")]
pub struct Model {
    /// Primary key ID (auto-incremented).
    #[sea_orm(primary_key)]
    pub id: i64,
    /// Foreign key to the user.
    pub user_id: i64,
    /// Foreign key to the achievement.
    pub achievement_id: i64,
    /// Current level achieved (0-based, where 0 means no levels completed yet).
    pub current_level: i32,
    /// Numeric progress towards the next level.
    pub progress_value: i32,
    /// Timestamp when this progress was last updated.
    pub last_updated_at: DateTime<Utc>,
    /// Timestamp when this progress record was created.
    pub created_at: DateTime<Utc>,
}

/// Defines relationships between `achievement_progress` and other tables.
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    /// Link to the related user.
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    User,

    /// Link to the related achievement.
    #[sea_orm(
        belongs_to = "super::achievements::Entity",
        from = "Column::AchievementId",
        to = "super::achievements::Column::Id"
    )]
    Achievement,
}

/// SeaORM hook point for customizing model behavior.
impl ActiveModelBehavior for ActiveModel {}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::achievements::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Achievement.def()
    }
}

impl Model {
    /// Creates a new achievement progress record and returns the inserted model.
    ///
    /// # Arguments
    /// * `db` - Database connection reference.
    /// * `user_id` - ID of the user.
    /// * `achievement_id` - ID of the achievement.
    /// * `current_level` - Starting level (typically 0).
    /// * `progress_value` - Initial progress value (typically 0).
    pub async fn create(
        db: &DatabaseConnection,
        user_id: i64,
        achievement_id: i64,
        current_level: i32,
        progress_value: i32,
    ) -> Result<Model, DbErr> {
        let active = ActiveModel {
            user_id: Set(user_id),
            achievement_id: Set(achievement_id),
            current_level: Set(current_level),
            progress_value: Set(progress_value),
            last_updated_at: Set(Utc::now()),
            ..Default::default()
        };
        active.insert(db).await
    }

    /// Fetches achievement progress for a specific user and achievement.
    ///
    /// # Arguments
    /// * `db` - Database connection.
    /// * `user_id` - The user ID.
    /// * `achievement_id` - The achievement ID.
    ///
    /// # Returns
    /// An optional achievement progress model if found.
    pub async fn get_by_user_and_achievement(
        db: &DatabaseConnection,
        user_id: i64,
        achievement_id: i64,
    ) -> Result<Option<Model>, DbErr> {
        Entity::find()
            .filter(Column::UserId.eq(user_id))
            .filter(Column::AchievementId.eq(achievement_id))
            .one(db)
            .await
    }

    /// Fetches all achievement progress records for a specific user.
    ///
    /// # Arguments
    /// * `db` - Database connection.
    /// * `user_id` - The user ID.
    ///
    /// # Returns
    /// A vector of achievement progress models.
    pub async fn get_by_user(
        db: &DatabaseConnection,
        user_id: i64,
    ) -> Result<Vec<Model>, DbErr> {
        Entity::find()
            .filter(Column::UserId.eq(user_id))
            .all(db)
            .await
    }

    /// Fetches all users who have progress on a specific achievement.
    ///
    /// # Arguments
    /// * `db` - Database connection.
    /// * `achievement_id` - The achievement ID.
    ///
    /// # Returns
    /// A vector of achievement progress models.
    pub async fn get_by_achievement(
        db: &DatabaseConnection,
        achievement_id: i64,
    ) -> Result<Vec<Model>, DbErr> {
        Entity::find()
            .filter(Column::AchievementId.eq(achievement_id))
            .all(db)
            .await
    }

    /// Updates the progress value and level for a specific achievement progress record.
    ///
    /// # Arguments
    /// * `db` - Database connection.
    /// * `id` - Achievement progress record ID.
    /// * `new_level` - New current level.
    /// * `new_progress` - New progress value.
    ///
    /// # Returns
    /// The updated achievement progress model.
    pub async fn update_progress(
        db: &DatabaseConnection,
        id: i64,
        new_level: i32,
        new_progress: i32,
    ) -> Result<Model, DbErr> {
        let mut active: ActiveModel = Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("Achievement progress not found".to_string()))?
            .into();

        active.current_level = Set(new_level);
        active.progress_value = Set(new_progress);
        active.last_updated_at = Set(Utc::now());
        active.update(db).await
    }

    /// Updates or creates achievement progress for a user and achievement.
    /// This is useful for incrementing progress or initializing it if it doesn't exist.
    ///
    /// # Arguments
    /// * `db` - Database connection.
    /// * `user_id` - The user ID.
    /// * `achievement_id` - The achievement ID.
    /// * `level_increment` - How many levels to add.
    /// * `progress_increment` - How much progress to add.
    ///
    /// # Returns
    /// The updated or created achievement progress model.
    pub async fn update_or_create_progress(
        db: &DatabaseConnection,
        user_id: i64,
        achievement_id: i64,
        level_increment: i32,
        progress_increment: i32,
    ) -> Result<Model, DbErr> {
        if let Some(existing) = Self::get_by_user_and_achievement(db, user_id, achievement_id).await? {
            // Update existing progress
            Self::update_progress(
                db,
                existing.id,
                existing.current_level + level_increment,
                existing.progress_value + progress_increment,
            ).await
        } else {
            // Create new progress record
            Self::create(db, user_id, achievement_id, level_increment, progress_increment).await
        }
    }

    /// Gets the leaderboard for a specific achievement (users with highest levels/progress).
    ///
    /// # Arguments
    /// * `db` - Database connection.
    /// * `achievement_id` - The achievement ID.
    /// * `limit` - Maximum number of results to return.
    ///
    /// # Returns
    /// A vector of achievement progress models ordered by level (desc) then progress (desc).
    pub async fn get_leaderboard(
        db: &DatabaseConnection,
        achievement_id: i64,
        limit: u64,
    ) -> Result<Vec<Model>, DbErr> {
        Entity::find()
            .filter(Column::AchievementId.eq(achievement_id))
            .order_by_desc(Column::CurrentLevel)
            .order_by_desc(Column::ProgressValue)
            .limit(limit)
            .all(db)
            .await
    }

    /// Checks if a user has completed a specific achievement (reached max level).
    ///
    /// # Arguments
    /// * `db` - Database connection.
    /// * `user_id` - The user ID.
    /// * `achievement_id` - The achievement ID.
    /// * `max_level` - Maximum level for this achievement.
    ///
    /// # Returns
    /// `true` if the user has completed the achievement, `false` otherwise.
    pub async fn is_achievement_completed(
        db: &DatabaseConnection,
        user_id: i64,
        achievement_id: i64,
        max_level: i32,
    ) -> Result<bool, DbErr> {
        if let Some(progress) = Self::get_by_user_and_achievement(db, user_id, achievement_id).await? {
            Ok(progress.current_level >= max_level)
        } else {
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{achievements, user};
    use crate::test_utils::setup_test_db;

    #[tokio::test]
    async fn test_create_and_get_progress() {
        let db = setup_test_db().await;

        // Create a user
        let user = user::Model::create(&db, "u12345678", "test@example.com", "password", false)
            .await
            .expect("Failed to create user");

        // Create an achievement
        let achievement = achievements::Model::create(
            &db,
            "Test Achievement",
            "A test achievement",
            true,
            3,
            "test_condition",
        )
        .await
        .expect("Failed to create achievement");

        // Create progress
        let progress = Model::create(&db, user.id, achievement.id, 0, 5)
            .await
            .expect("Failed to create progress");

        assert_eq!(progress.user_id, user.id);
        assert_eq!(progress.achievement_id, achievement.id);
        assert_eq!(progress.current_level, 0);
        assert_eq!(progress.progress_value, 5);

        // Test get by user and achievement
        let found = Model::get_by_user_and_achievement(&db, user.id, achievement.id)
            .await
            .expect("Failed to query progress");

        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.id, progress.id);
    }

    #[tokio::test]
    async fn test_update_progress() {
        let db = setup_test_db().await;

        // Create user and achievement
        let user = user::Model::create(&db, "u87654321", "progress@example.com", "password", false)
            .await
            .expect("Failed to create user");

        let achievement = achievements::Model::create(
            &db,
            "Progress Test",
            "Testing progress updates",
            true,
            5,
            "progress_test",
        )
        .await
        .expect("Failed to create achievement");

        // Create initial progress
        let progress = Model::create(&db, user.id, achievement.id, 1, 10)
            .await
            .expect("Failed to create progress");

        // Update progress
        let updated = Model::update_progress(&db, progress.id, 2, 25)
            .await
            .expect("Failed to update progress");

        assert_eq!(updated.current_level, 2);
        assert_eq!(updated.progress_value, 25);
        assert!(updated.last_updated_at > progress.last_updated_at);
    }

    #[tokio::test]
    async fn test_update_or_create_progress() {
        let db = setup_test_db().await;

        // Create user and achievement
        let user = user::Model::create(&db, "u11111111", "upsert@example.com", "password", false)
            .await
            .expect("Failed to create user");

        let achievement = achievements::Model::create(
            &db,
            "Upsert Test",
            "Testing upsert functionality",
            true,
            3,
            "upsert_test",
        )
        .await
        .expect("Failed to create achievement");

        // First call should create new progress
        let created = Model::update_or_create_progress(&db, user.id, achievement.id, 1, 5)
            .await
            .expect("Failed to upsert progress (create)");

        assert_eq!(created.current_level, 1);
        assert_eq!(created.progress_value, 5);

        // Second call should update existing progress
        let updated = Model::update_or_create_progress(&db, user.id, achievement.id, 1, 10)
            .await
            .expect("Failed to upsert progress (update)");

        assert_eq!(updated.current_level, 2);
        assert_eq!(updated.progress_value, 15);
        assert_eq!(updated.id, created.id); // Same record
    }

    #[tokio::test]
    async fn test_is_achievement_completed() {
        let db = setup_test_db().await;

        // Create user and achievement
        let user = user::Model::create(&db, "u99999999", "complete@example.com", "password", false)
            .await
            .expect("Failed to create user");

        let achievement = achievements::Model::create(
            &db,
            "Completion Test",
            "Testing completion check",
            true,
            3,
            "completion_test",
        )
        .await
        .expect("Failed to create achievement");

        // No progress yet - should not be completed
        let completed = Model::is_achievement_completed(&db, user.id, achievement.id, 3)
            .await
            .expect("Failed to check completion");
        assert!(!completed);

        // Create progress but not at max level
        Model::create(&db, user.id, achievement.id, 2, 0)
            .await
            .expect("Failed to create progress");

        let completed = Model::is_achievement_completed(&db, user.id, achievement.id, 3)
            .await
            .expect("Failed to check completion");
        assert!(!completed);

        // Update to max level
        let progress = Model::get_by_user_and_achievement(&db, user.id, achievement.id)
            .await
            .expect("Failed to get progress")
            .unwrap();

        Model::update_progress(&db, progress.id, 3, 0)
            .await
            .expect("Failed to update progress");

        let completed = Model::is_achievement_completed(&db, user.id, achievement.id, 3)
            .await
            .expect("Failed to check completion");
        assert!(completed);
    }
}