use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveModelTrait, DatabaseConnection, DbErr, EntityTrait, Set};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Comparison operators for achievement condition checks
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ComparisonOperator {
    /// Equal to
    Eq,
    /// Greater than or equal to
    Gte,
    /// Less than or equal to
    Lte,
    /// Greater than
    Gt,
    /// Less than
    Lt,
    /// Not equal to
    Ne,
    /// Field exists (ignores value)
    Exists,
    /// String/array contains value
    Contains,
    /// Value is in array
    In,
}

/// A single field check within an achievement condition
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConditionCheck {
    /// Field name to check in the event data
    pub field: String,
    /// Comparison operator to use
    pub op: ComparisonOperator,
    /// Target value for comparison (optional for "exists" operator)
    pub value: Option<serde_json::Value>,
}

/// Achievement condition definition
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AchievementCondition {
    /// Event type to listen for (e.g., "assignment_submitted")
    pub event: String,
    /// List of field checks that must all pass
    pub checks: Vec<ConditionCheck>,
}

/// Achievement definition from JSON
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AchievementDefinition {
    /// Unique identifier (matches condition_id in database)
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Achievement description
    pub description: String,
    /// Whether this is a positive achievement
    pub is_positive: bool,
    /// Event condition to trigger this achievement
    pub condition: AchievementCondition,
    /// Required counts for each of the 5 levels
    pub level_thresholds: [u32; 5],
}

/// Root structure of achievements.json
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AchievementsConfig {
    /// Map of achievement ID to achievement definition
    pub achievements: HashMap<String, AchievementDefinition>,
}

impl ConditionCheck {
    /// Creates a new condition check
    pub fn new(field: String, op: ComparisonOperator, value: Option<serde_json::Value>) -> Self {
        Self { field, op, value }
    }

    /// Creates an equality check
    pub fn eq<T: Into<serde_json::Value>>(field: String, value: T) -> Self {
        Self::new(field, ComparisonOperator::Eq, Some(value.into()))
    }

    /// Creates a greater-than-or-equal check
    pub fn gte<T: Into<serde_json::Value>>(field: String, value: T) -> Self {
        Self::new(field, ComparisonOperator::Gte, Some(value.into()))
    }

    /// Creates an existence check
    pub fn exists(field: String) -> Self {
        Self::new(field, ComparisonOperator::Exists, None)
    }

    /// Evaluates this check against event data
    pub fn evaluate(&self, event_data: &serde_json::Value) -> bool {
        let field_value = match event_data.get(&self.field) {
            Some(value) => value,
            None => return self.op == ComparisonOperator::Exists && self.value.is_none(),
        };

        match &self.op {
            ComparisonOperator::Exists => true,
            ComparisonOperator::Eq => {
                self.value.as_ref().map_or(false, |v| field_value == v)
            }
            ComparisonOperator::Ne => {
                self.value.as_ref().map_or(true, |v| field_value != v)
            }
            ComparisonOperator::Gt => {
                self.compare_numeric(field_value, |a, b| a > b)
            }
            ComparisonOperator::Gte => {
                self.compare_numeric(field_value, |a, b| a >= b)
            }
            ComparisonOperator::Lt => {
                self.compare_numeric(field_value, |a, b| a < b)
            }
            ComparisonOperator::Lte => {
                self.compare_numeric(field_value, |a, b| a <= b)
            }
            ComparisonOperator::Contains => {
                self.check_contains(field_value)
            }
            ComparisonOperator::In => {
                self.check_in(field_value)
            }
        }
    }

    /// Helper for numeric comparisons
    fn compare_numeric<F>(&self, field_value: &serde_json::Value, comparator: F) -> bool
    where
        F: Fn(f64, f64) -> bool,
    {
        let field_num = match field_value.as_f64() {
            Some(n) => n,
            None => return false,
        };

        let target_num = match self.value.as_ref().and_then(|v| v.as_f64()) {
            Some(n) => n,
            None => return false,
        };

        comparator(field_num, target_num)
    }

    /// Helper for contains check
    fn check_contains(&self, field_value: &serde_json::Value) -> bool {
        let target = match &self.value {
            Some(v) => v,
            None => return false,
        };

        match field_value {
            serde_json::Value::String(s) => {
                target.as_str().map_or(false, |t| s.contains(t))
            }
            serde_json::Value::Array(arr) => arr.contains(target),
            _ => false,
        }
    }

    /// Helper for "in" check
    fn check_in(&self, field_value: &serde_json::Value) -> bool {
        match &self.value {
            Some(serde_json::Value::Array(arr)) => arr.contains(field_value),
            _ => false,
        }
    }
}

impl AchievementCondition {
    /// Creates a new achievement condition
    pub fn new(event: String, checks: Vec<ConditionCheck>) -> Self {
        Self { event, checks }
    }

    /// Evaluates all checks against event data
    pub fn evaluate(&self, event_type: &str, event_data: &serde_json::Value) -> bool {
        if self.event != event_type {
            return false;
        }

        self.checks.iter().all(|check| check.evaluate(event_data))
    }
}

impl AchievementDefinition {
    /// Loads achievement definitions from JSON string
    pub fn load_from_json(json_content: &str) -> Result<HashMap<String, Self>, serde_json::Error> {
        let config: AchievementsConfig = serde_json::from_str(json_content)?;
        Ok(config.achievements)
    }

    /// Gets the threshold for a specific level (1-5)
    pub fn get_level_threshold(&self, level: u8) -> Option<u32> {
        if level == 0 || level > 5 {
            return None;
        }
        Some(self.level_thresholds[(level - 1) as usize])
    }

    /// Checks if a progress value reaches the threshold for a specific level
    pub fn is_level_reached(&self, level: u8, progress: u32) -> bool {
        self.get_level_threshold(level)
            .map_or(false, |threshold| progress >= threshold)
    }
}

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
    use serde_json::json;

    #[tokio::test]
    async fn test_create_and_get_achievement() {
        let db = setup_test_db().await;
        let name = "First Submission";
        let description = "Submit your first assignment";
        let is_positive = true;
        let levels = 5;
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

        Model::create(&db, name, description, true, 5, condition_id)
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
        Model::create(&db, "Early Bird", "Submit before deadline", true, 5, "early_bird")
            .await
            .expect("Failed to create positive achievement");

        // Create negative achievement
        Model::create(&db, "Late Submission", "Submit after deadline", false, 5, "late_submission")
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

        let achievement = Model::create(&db, name, original_description, true, 5, "test_achievement")
            .await
            .expect("Failed to create achievement");

        let updated = Model::update_description(&db, achievement.id, new_description)
            .await
            .expect("Failed to update description");

        assert_eq!(updated.description, new_description);
        assert_eq!(updated.id, achievement.id);
    }

    #[test]
    fn test_condition_check_evaluation() {
        // Test equality check
        let check = ConditionCheck::eq("score".to_string(), 100);
        let event_data = json!({"score": 100, "attempt": 1});
        assert!(check.evaluate(&event_data));

        let event_data = json!({"score": 90, "attempt": 1});
        assert!(!check.evaluate(&event_data));

        // Test greater than or equal check
        let check = ConditionCheck::gte("attempt".to_string(), 1);
        let event_data = json!({"score": 100, "attempt": 3});
        assert!(check.evaluate(&event_data));

        let event_data = json!({"score": 100, "attempt": 0});
        assert!(!check.evaluate(&event_data));

        // Test exists check
        let check = ConditionCheck::exists("submission_id".to_string());
        let event_data = json!({"submission_id": 123, "score": 100});
        assert!(check.evaluate(&event_data));

        let event_data = json!({"score": 100});
        assert!(!check.evaluate(&event_data));
    }

    #[test]
    fn test_achievement_condition_evaluation() {
        // Create condition with multiple checks
        let condition = AchievementCondition::new(
            "assignment_graded".to_string(),
            vec![
                ConditionCheck::eq("score".to_string(), 100),
                ConditionCheck::eq("attempt".to_string(), 1),
            ]
        );

        // Test matching event and data
        let event_data = json!({"score": 100, "attempt": 1});
        assert!(condition.evaluate("assignment_graded", &event_data));

        // Test wrong event type
        assert!(!condition.evaluate("assignment_submitted", &event_data));

        // Test partial match
        let event_data = json!({"score": 100, "attempt": 2});
        assert!(!condition.evaluate("assignment_graded", &event_data));
    }

    #[test]
    fn test_achievement_definition_loading() {
        let json_content = r#"
        {
          "achievements": {
            "test_achievement": {
              "id": "test_achievement",
              "name": "Test Achievement",
              "description": "A test achievement",
              "is_positive": true,
              "condition": {
                "event": "test_event",
                "checks": [
                  { "field": "score", "op": "gte", "value": 90 }
                ]
              },
              "level_thresholds": [1, 5, 10, 20, 50]
            }
          }
        }
        "#;

        let definitions = AchievementDefinition::load_from_json(json_content)
            .expect("Failed to load achievement definitions");

        assert_eq!(definitions.len(), 1);
        let achievement = definitions.get("test_achievement").unwrap();
        assert_eq!(achievement.name, "Test Achievement");
        assert_eq!(achievement.level_thresholds, [1, 5, 10, 20, 50]);
    }

    #[test]
    fn test_level_thresholds() {
        let definition = AchievementDefinition {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: "Test".to_string(),
            is_positive: true,
            condition: AchievementCondition::new("test".to_string(), vec![]),
            level_thresholds: [1, 5, 10, 20, 50],
        };

        // Test valid levels
        assert_eq!(definition.get_level_threshold(1), Some(1));
        assert_eq!(definition.get_level_threshold(3), Some(10));
        assert_eq!(definition.get_level_threshold(5), Some(50));

        // Test invalid levels
        assert_eq!(definition.get_level_threshold(0), None);
        assert_eq!(definition.get_level_threshold(6), None);

        // Test level checking
        assert!(definition.is_level_reached(1, 1));
        assert!(definition.is_level_reached(2, 10));
        assert!(!definition.is_level_reached(3, 5));
    }

    #[test]
    fn test_complex_operators() {
        // Test contains operator
        let check = ConditionCheck::new(
            "tags".to_string(),
            ComparisonOperator::Contains,
            Some(json!("important"))
        );
        let event_data = json!({"tags": ["urgent", "important", "homework"]});
        assert!(check.evaluate(&event_data));

        let event_data = json!({"tags": ["urgent", "homework"]});
        assert!(!check.evaluate(&event_data));

        // Test in operator
        let check = ConditionCheck::new(
            "grade".to_string(),
            ComparisonOperator::In,
            Some(json!(["A", "B", "C"]))
        );
        let event_data = json!({"grade": "B"});
        assert!(check.evaluate(&event_data));

        let event_data = json!({"grade": "F"});
        assert!(!check.evaluate(&event_data));
    }
}