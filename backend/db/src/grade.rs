use std::{cmp::Ordering, collections::HashMap};

use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, JoinType, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect, QueryTrait, RelationTrait,
};

use crate::models::{
    assignment,
    assignment_submission::{
        Column as SubCol, Entity as SubmissionEntity, Model as SubmissionModel, Relation as SubRel,
    },
    user::{Column as UserCol, Entity as UserEntity, Model as UserModel},
    user_module_role::{Column as UmrCol, Entity as UmrEntity, Role as ModuleRole},
};

use util::execution_config::{ExecutionConfig, GradingPolicy};

#[derive(Debug, Clone)]
pub struct GradeSelection {
    pub submission: SubmissionModel,
    pub user: UserModel,
    pub score_pct: f64,
}

#[derive(Debug, Clone)]
pub struct GradeComputationResult {
    pub execution_config: ExecutionConfig,
    pub grades: Vec<GradeSelection>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct GradeComputationOptions<'a> {
    pub username_filter: Option<&'a str>,
    pub user_id: Option<i64>,
}

#[derive(Debug)]
pub enum GradeComputationError {
    AssignmentNotFound,
    ExecutionConfig(String),
    Database(sea_orm::DbErr),
}

impl From<sea_orm::DbErr> for GradeComputationError {
    fn from(value: sea_orm::DbErr) -> Self {
        GradeComputationError::Database(value)
    }
}

impl std::fmt::Display for GradeComputationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GradeComputationError::AssignmentNotFound => write!(f, "Assignment not found"),
            GradeComputationError::ExecutionConfig(e) => write!(f, "Execution config error: {e}"),
            GradeComputationError::Database(e) => write!(f, "Database error: {e}"),
        }
    }
}

impl std::error::Error for GradeComputationError {}

/// Helper to compute percentage safely.
pub fn percentage(earned: f64, total: f64) -> f64 {
    if total <= 0.0 {
        0.0
    } else {
        (earned * 100.0) / total
    }
}

fn apply_policy(
    policy: GradingPolicy,
    attempts: Vec<(SubmissionModel, UserModel)>,
) -> Option<(SubmissionModel, UserModel)> {
    match policy {
        GradingPolicy::Last => attempts.into_iter().next(),
        GradingPolicy::Best => {
            attempts
                .into_iter()
                .max_by(|(a_submission, _), (b_submission, _)| {
                    let a_ratio = a_submission.earned as f64 / a_submission.total.max(1.0) as f64;
                    let b_ratio = b_submission.earned as f64 / b_submission.total.max(1.0) as f64;

                    match a_ratio.partial_cmp(&b_ratio).unwrap_or(Ordering::Equal) {
                        Ordering::Equal => {
                            match a_submission.created_at.cmp(&b_submission.created_at) {
                                Ordering::Equal => a_submission.attempt.cmp(&b_submission.attempt),
                                ord => ord,
                            }
                        }
                        ord => ord,
                    }
                })
        }
    }
}

async fn load_execution_config(
    module_id: i64,
    assignment_id: i64,
) -> Result<ExecutionConfig, GradeComputationError> {
    ExecutionConfig::get_execution_config(module_id, assignment_id)
        .map_err(GradeComputationError::ExecutionConfig)
}

/// Compute grades for an assignment based on the execution config policy.
///
/// Returns the execution config along with the chosen submission per student (respecting policy).
pub async fn compute_assignment_grades(
    db: &DatabaseConnection,
    module_id: i64,
    assignment_id: i64,
    options: GradeComputationOptions<'_>,
) -> Result<GradeComputationResult, GradeComputationError> {
    let assignment_exists = assignment::Entity::find()
        .filter(assignment::Column::Id.eq(assignment_id))
        .filter(assignment::Column::ModuleId.eq(module_id))
        .count(db)
        .await?;

    if assignment_exists == 0 {
        return Err(GradeComputationError::AssignmentNotFound);
    }

    let exec_cfg = load_execution_config(module_id, assignment_id).await?;

    let student_ids_subq = UmrEntity::find()
        .select_only()
        .column(UmrCol::UserId)
        .filter(UmrCol::ModuleId.eq(module_id))
        .filter(UmrCol::Role.eq(ModuleRole::Student));

    let mut query = SubmissionEntity::find()
        .filter(SubCol::AssignmentId.eq(assignment_id))
        .filter(SubCol::IsPractice.eq(false))
        .filter(SubCol::Ignored.eq(false))
        .filter(SubCol::UserId.in_subquery(student_ids_subq.as_query().to_owned()))
        .join(JoinType::InnerJoin, SubRel::User.def())
        .select_also(UserEntity)
        .order_by_asc(SubCol::UserId)
        .order_by_desc(SubCol::CreatedAt)
        .order_by_desc(SubCol::Attempt);

    if let Some(user_id) = options.user_id {
        query = query.filter(SubCol::UserId.eq(user_id));
    }

    if let Some(username_filter) = options.username_filter {
        let filter = username_filter.trim();
        if !filter.is_empty() {
            query = query.filter(UserCol::Username.contains(filter));
        }
    }

    let rows: Vec<(SubmissionModel, Option<UserModel>)> = query.all(db).await?;

    let mut per_user: HashMap<i64, Vec<(SubmissionModel, UserModel)>> = HashMap::new();
    for (submission, user_opt) in rows {
        if let Some(user) = user_opt {
            per_user
                .entry(submission.user_id)
                .or_default()
                .push((submission, user));
        }
    }

    let mut grades = Vec::with_capacity(per_user.len());

    for (_user_id, attempts) in per_user.into_iter() {
        if let Some((submission, user)) = apply_policy(exec_cfg.marking.grading_policy, attempts) {
            grades.push(GradeSelection {
                score_pct: percentage(submission.earned, submission.total),
                submission,
                user,
            });
        }
    }

    grades.sort_by(|a, b| a.user.id.cmp(&b.user.id));

    Ok(GradeComputationResult {
        execution_config: exec_cfg,
        grades,
    })
}

/// Convenience helper to compute the grade for a single student within an assignment.
pub async fn compute_assignment_grade_for_student(
    db: &DatabaseConnection,
    module_id: i64,
    assignment_id: i64,
    student_id: i64,
) -> Result<Option<GradeSelection>, GradeComputationError> {
    let result = compute_assignment_grades(
        db,
        module_id,
        assignment_id,
        GradeComputationOptions {
            username_filter: None,
            user_id: Some(student_id),
        },
    )
    .await?;

    Ok(result.grades.into_iter().next())
}
