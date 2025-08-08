use sea_orm::entity::prelude::*;
use sea_orm::Iterable;
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "grades")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,

    pub assignment_id: i64,        // FK -> assignments.id
    pub student_id: i64,           // FK -> users.id
    pub submission_id: Option<i64>,// FK -> submissions.id (nullable)

    pub score: f32,             
    pub created_at: DateTime,      // DEFAULT CURRENT_TIMESTAMP (DB)
    pub updated_at: DateTime,      // DEFAULT CURRENT_TIMESTAMP (DB); todo: might have to go back and add this to the DB migration
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Assignment,
    Student,
    AssignmentSubmission,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            // Grade -> Assignment (belongs_to)
            Relation::Assignment => Entity::belongs_to(super::assignment::Entity)
                .from(Column::AssignmentId)
                .to(super::assignment::Column::Id)
                .on_delete(ForeignKeyAction::Cascade) // delete grades if assignment is deleted
                .into(),
            
            // Grade -> User (student) (belongs_to)    
            Relation::Student => Entity::belongs_to(super::user::Entity)
                .from(Column::StudentId)
                .to(super::user::Column::Id)
                .on_delete(ForeignKeyAction::Cascade) // delete grades if user is deleted
                .into(),
            
            // Grade -> Submission (belongs_to, nullable)    
            Relation::AssignmentSubmission => Entity::belongs_to(super::assignment_submission::Entity)
                .from(Column::SubmissionId)
                .to(super::assignment_submission::Column::Id)
                .on_delete(ForeignKeyAction::SetNull) // set to null if submission is deleted
                .into(),
        }
    }
}

impl Related<super::assignment::Entity> for Entity {
    fn to() -> RelationDef { Relation::Assignment.def() }
}
impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef { Relation::Student.def() }
}
impl Related<super::assignment_submission::Entity> for Entity {
    fn to() -> RelationDef { Relation::AssignmentSubmission.def() }
}



/// Load a single grade record for a specific assignment & student,
/// along with its related `Assignment`, `User` (student), and optional `AssignmentSubmission`.
///
/// # Arguments
/// * `db` - The active database connection.
/// * `assignment_id` - The ID of the assignment the grade belongs to.
/// * `student_id` - The ID of the student whose grade is being retrieved.
///
/// # Returns
/// * `Ok(Some((grade, submission, assignment, student)))`
///   - `grade` - The `Grade` model.
///   - `submission` - An `Option<AssignmentSubmission>` if the grade is linked to a submission; `None` if no submission was selected.
///   - `assignment` - The `Assignment` model linked to the grade.
///   - `student` - The `User` model linked to the grade.
/// * `Ok(None)` - No grade was found for the given assignment/student.
/// * `Err(DbErr)` - A database error occurred.
///
/// # Example
/// ```rust,ignore
/// let result = grade::find_one_with_related(&db, 42, 101).await?;
/// if let Some((grade, submission, assignment, student)) = result {
///     println!("Grade: {}% for student {}", grade.score, student.name);
/// }
/// ```
pub async fn find_one_with_related(
    db: &DatabaseConnection,
    assignment_id: i64,
    student_id: i64,
) -> Result<Option<(Model, Option<super::assignment_submission::Model>, super::assignment::Model, super::user::Model)>, sea_orm::DbErr> {
    if let Some(grade) = Entity::find()
        .filter(Column::AssignmentId.eq(assignment_id))
        .filter(Column::StudentId.eq(student_id))
        .one(db)
        .await?
    {
        let assignment = grade
            .find_related(super::assignment::Entity)
            .one(db)
            .await?
            .expect("FK to assignment is non-null, but record not found");

        let student = grade
            .find_related(super::user::Entity)
            .one(db)
            .await?
            .expect("FK to student is non-null, but record not found");

        let submission = grade
            .find_related(super::assignment_submission::Entity)
            .one(db)
            .await?;

        Ok(Some((grade, submission, assignment, student)))
    } else {
        Ok(None)
    }
}
