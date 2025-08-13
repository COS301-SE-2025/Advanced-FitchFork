use crate::models::plagiarism_case;
use crate::repositories::repository::Repository;
use crate::filters::PlagiarismCaseFilter;
use sea_orm::{prelude::Expr, QueryFilter, QueryOrder, ColumnTrait, DatabaseConnection, Select, Condition};

#[derive(Clone)]
pub struct PlagiarismCaseRepository {
    db: DatabaseConnection,
}

impl PlagiarismCaseRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl Repository<plagiarism_case::Entity, PlagiarismCaseFilter> for PlagiarismCaseRepository {
    fn db(&self) -> &DatabaseConnection {
        &self.db
    }

    fn apply_filter(query: Select<plagiarism_case::Entity>, filter: &PlagiarismCaseFilter) -> Select<plagiarism_case::Entity> {
        let mut condition = Condition::all();

        if let Some(id) = filter.id {
            condition = condition.add(plagiarism_case::Column::Id.eq(id));
        }

        if let Some(assignment_id) = filter.assignment_id {
            condition = condition.add(plagiarism_case::Column::AssignmentId.eq(assignment_id));
        }

        if let Some(submission_id_1) = filter.submission_id_1 {
            condition = condition.add(plagiarism_case::Column::SubmissionId1.eq(submission_id_1));
        }

        if let Some(submission_id_2) = filter.submission_id_2 {
            condition = condition.add(plagiarism_case::Column::SubmissionId2.eq(submission_id_2));
        }

        if let Some(ref description) = filter.description {
            let pattern = format!("%{}%", description.to_lowercase());
            condition = condition.add(Expr::cust("LOWER(description)").like(&pattern));
        }

        if let Some(ref status) = filter.status {
            condition = condition.add(plagiarism_case::Column::Status.eq(status.clone()));
        }

        if let Some(ref query_text) = filter.query {
            let pattern = format!("%{}%", query_text.to_lowercase());
            let search_condition = Condition::any()
                .add(Expr::cust("LOWER(description)").like(&pattern));
            condition = condition.add(search_condition);
        }

        query.filter(condition)
    }

    fn apply_sorting(mut query: Select<plagiarism_case::Entity>, sort_by: Option<String>) -> Select<plagiarism_case::Entity> {
        if let Some(sort_param) = sort_by {
            for sort in sort_param.split(',') {
                let (field, asc) = if sort.starts_with('-') {
                    (&sort[1..], false)
                } else {
                    (sort, true)
                };

                query = match field {
                    "id" => {
                        if asc {
                            query.order_by_asc(plagiarism_case::Column::Id)
                        } else {
                            query.order_by_desc(plagiarism_case::Column::Id)
                        }
                    }
                    "assignment_id" => {
                        if asc {
                            query.order_by_asc(plagiarism_case::Column::AssignmentId)
                        } else {
                            query.order_by_desc(plagiarism_case::Column::AssignmentId)
                        }
                    }
                    "submission_id_1" => {
                        if asc {
                            query.order_by_asc(plagiarism_case::Column::SubmissionId1)
                        } else {
                            query.order_by_desc(plagiarism_case::Column::SubmissionId1)
                        }
                    }
                    "submission_id_2" => {
                        if asc {
                            query.order_by_asc(plagiarism_case::Column::SubmissionId2)
                        } else {
                            query.order_by_desc(plagiarism_case::Column::SubmissionId2)
                        }
                    }
                    "description" => {
                        if asc {
                            query.order_by_asc(plagiarism_case::Column::Description)
                        } else {
                            query.order_by_desc(plagiarism_case::Column::Description)
                        }
                    }
                    "status" => {
                        if asc {
                            query.order_by_asc(plagiarism_case::Column::Status)
                        } else {
                            query.order_by_desc(plagiarism_case::Column::Status)
                        }
                    }
                    "created_at" => {
                        if asc {
                            query.order_by_asc(plagiarism_case::Column::CreatedAt)
                        } else {
                            query.order_by_desc(plagiarism_case::Column::CreatedAt)
                        }
                    }
                    "updated_at" => {
                        if asc {
                            query.order_by_asc(plagiarism_case::Column::UpdatedAt)
                        } else {
                            query.order_by_desc(plagiarism_case::Column::UpdatedAt)
                        }
                    }
                    _ => query,
                };
            }
        }
        query
    }
}