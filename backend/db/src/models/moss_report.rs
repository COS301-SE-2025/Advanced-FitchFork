//! MOSS report metadata (many per assignment), including archive info,
//! URL liveness tracking, and generation filter metadata.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, DbErr, EntityTrait,
    IntoActiveModel, QueryFilter, QueryOrder,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "moss_reports")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,

    pub assignment_id: i64,
    pub report_url: String,
    pub generated_at: DateTime<Utc>,

    pub description: String,

    pub has_archive: bool,
    pub archive_generated_at: Option<DateTime<Utc>>,

    pub filter_mode: FilterMode,
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub filter_patterns: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Text")]
#[serde(rename_all = "lowercase")]
pub enum FilterMode {
    #[sea_orm(string_value = "all")]
    All,
    #[sea_orm(string_value = "whitelist")]
    Whitelist,
    #[sea_orm(string_value = "blacklist")]
    Blacklist,
}

impl fmt::Display for FilterMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            FilterMode::All => "all",
            FilterMode::Whitelist => "whitelist",
            FilterMode::Blacklist => "blacklist",
        };
        write!(f, "{s}")
    }
}

impl std::str::FromStr for FilterMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "all" => Ok(FilterMode::All),
            "whitelist" => Ok(FilterMode::Whitelist),
            "blacklist" => Ok(FilterMode::Blacklist),
            other => Err(format!("invalid FilterMode: {other}")),
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::assignment::Entity",
        from = "Column::AssignmentId",
        to = "super::assignment::Column::Id",
        on_delete = "Cascade"
    )]
    Assignment,
}

impl Related<super::assignment::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Assignment.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Entity {
    /// Create a new report. Enforces mutually-exclusive filter semantics.
    pub async fn create_report(
        db: &DatabaseConnection,
        assignment_id: i64,
        report_url: &str,
        filter_mode: FilterMode,
        description: String,
        filter_patterns: Option<Vec<String>>,
    ) -> Result<Model, DbErr> {
        // validations (unchanged)
        let patterns_len = filter_patterns.as_ref().map(|v| v.len()).unwrap_or(0);
        match filter_mode {
            FilterMode::All => {
                if patterns_len > 0 {
                    return Err(DbErr::Custom(
                        "filter_mode=all does not accept filter_patterns".into(),
                    ));
                }
            }
            FilterMode::Whitelist | FilterMode::Blacklist => {
                if patterns_len == 0 {
                    return Err(DbErr::Custom(
                        "filter_patterns required (non-empty) when filter_mode is whitelist or blacklist".into(),
                    ));
                }
            }
        }

        let now = Utc::now();
        let active = ActiveModel {
            assignment_id: Set(assignment_id),
            report_url: Set(report_url.to_string()),
            generated_at: Set(now),
            description: Set(description),
            has_archive: Set(false),
            archive_generated_at: Set(None),
            filter_mode: Set(filter_mode),
            filter_patterns: Set(filter_patterns.map(|arr| json!(arr))),
            ..Default::default()
        };
        active.insert(db).await
    }

    /// All reports for assignment (newest first).
    pub async fn list_by_assignment(
        db: &DatabaseConnection,
        assignment_id: i64,
    ) -> Result<Vec<Model>, DbErr> {
        Entity::find()
            .filter(Column::AssignmentId.eq(assignment_id))
            .order_by_desc(Column::GeneratedAt)
            .all(db)
            .await
    }

    /// Latest (most recent) report for assignment.
    pub async fn latest_for_assignment(
        db: &DatabaseConnection,
        assignment_id: i64,
    ) -> Result<Option<Model>, DbErr> {
        Entity::find()
            .filter(Column::AssignmentId.eq(assignment_id))
            .order_by_desc(Column::GeneratedAt)
            .one(db)
            .await
    }

    /// Mark archive state for this report.
    pub async fn set_archive_state(
        db: &DatabaseConnection,
        report_id: i64,
        has_archive: bool,
        archive_generated_at: Option<DateTime<Utc>>,
    ) -> Result<(), DbErr> {
        if let Some(m) = Entity::find_by_id(report_id).one(db).await? {
            let mut am = m.into_active_model();
            am.has_archive = Set(has_archive);
            am.archive_generated_at = Set(archive_generated_at);
            am.update(db).await.map(|_| ())
        } else {
            Err(DbErr::RecordNotFound("moss_report not found".into()))
        }
    }
}
