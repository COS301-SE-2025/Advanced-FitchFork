use sea_orm::entity::prelude::*;
use sea_orm::Iterable;


#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "grades")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,

    pub assignment_id: i64,        // FK -> assignments.id
    pub student_id: i64,           // FK -> users.id
    pub submission_id: Option<i64>,// FK -> submissions.id (nullable)

    pub score: f32,                // 0.0..=100.0 (DB enforces default 0.0)
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
