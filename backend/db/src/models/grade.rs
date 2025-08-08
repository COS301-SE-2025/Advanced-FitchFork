use sea_orm::entity::prelude::*;
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, ForeignKeyAction};
use migration::MigratorTrait;

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



#[cfg(test)]
mod mapping_tests {
    use super::*;
    use sea_orm::{Database, DatabaseConnection, Statement, DbBackend, EntityTrait};

    fn s(sql: &str) -> Statement {
        Statement::from_string(DbBackend::Sqlite, sql.to_owned())
    }

    async fn enable_fk(db: &DatabaseConnection) {
        
        let _ = db.execute(s("PRAGMA foreign_keys = ON;")).await;
    }

    /// Verifies:
    ///  - table exists
    ///  - required columns + nullability
    ///  - unique index on (assignment_id, student_id)
    ///  - the 3 foreign keys + ON DELETE actions
    ///  - a basic Entity::find() works
    #[tokio::test]
    async fn grade_entity_matches_schema() {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        enable_fk(&db).await;

        migration::Migrator::up(&db, None).await.unwrap();

        let row = db
            .query_one(s("SELECT name FROM sqlite_master WHERE type='table' AND name='grades'"))
            .await
            .unwrap();
        assert!(row.is_some(), "grades table should exist");

        let cols = db.query_all(s("PRAGMA table_info('grades')")).await.unwrap();

        let mut have = std::collections::HashMap::new();
        for c in cols {
            let name: String = c.try_get("", "name").unwrap();
            let notnull: i64 = c.try_get("", "notnull").unwrap(); // 1 or 0
            have.insert(name, notnull);
        }

        for (col, nn) in [
            ("id", 1),
            ("assignment_id", 1),
            ("student_id", 1),
            ("submission_id", 0),
            ("score", 1),
            ("created_at", 1),
            ("updated_at", 1),
        ] {
            assert!(have.contains_key(col), "missing column: {col}");
            assert_eq!(have[col], nn, "unexpected nullability for {col}");
        }

        let idx_list = db.query_all(s("PRAGMA index_list('grades')")).await.unwrap();

        let mut found_unique = false;
        for idx in idx_list {
            let unique: i64 = idx.try_get("", "unique").unwrap_or(0);
            if unique == 1 {
                let idx_name: String = idx.try_get("", "name").unwrap();

                let cols = db
                    .query_all(Statement::from_string(
                        DbBackend::Sqlite,
                        format!("PRAGMA index_info('{idx_name}')"),
                    ))
                    .await
                    .unwrap();

                let mut cols_vec: Vec<String> = cols
                    .iter()
                    .map(|r| r.try_get::<String>("", "name").unwrap())
                    .collect();
                cols_vec.sort();

                let expect: Vec<String> = vec![
                    "assignment_id".to_owned(),
                    "student_id".to_owned(),
                ];
                if cols_vec == expect {
                    found_unique = true;
                    break;
                }
            }
        }
        assert!(found_unique, "unique index on (assignment_id, student_id) not found");

        let fks = db
            .query_all(s("PRAGMA foreign_key_list('grades')"))
            .await
            .unwrap();

        let mut seen = Vec::new();
        for fk in fks {
            let from: String = fk.try_get("", "from").unwrap();
            let table: String = fk.try_get("", "table").unwrap();
            let to: String = fk.try_get("", "to").unwrap();
            let on_delete: String = fk.try_get("", "on_delete").unwrap(); // "CASCADE", "SET NULL", etc.
            seen.push((from, table, to, on_delete.to_uppercase()));
        }
        seen.sort();

        let mut expect = vec![
            ("assignment_id".into(), "assignments".into(), "id".into(), "CASCADE".into()),
            ("student_id".into(), "users".into(), "id".into(), "CASCADE".into()),
            ("submission_id".into(), "assignment_submissions".into(), "id".into(), "SET NULL".into()),
        ];
        expect.sort();

        assert_eq!(seen, expect, "FKs or delete actions do not match");
        let _ = Entity::find().one(&db).await.unwrap();
    }

    #[test]
    fn entity_table_name_is_grades() {
        use sea_orm::EntityName;
        assert_eq!(Entity.table_name(), "grades");
    }
}