use crate::models::assignment_submission_output;
use crate::repositories::repository::Repository;
use crate::filters::AssignmentSubmissionOutputFilter;
use sea_orm::{prelude::Expr, QueryFilter, QueryOrder, ColumnTrait, Select, Condition};

pub struct AssignmentSubmissionOutputRepository;

impl AssignmentSubmissionOutputRepository {}

impl Repository<assignment_submission_output::Entity, AssignmentSubmissionOutputFilter> for AssignmentSubmissionOutputRepository {
    fn apply_filter(query: Select<assignment_submission_output::Entity>, filter: &AssignmentSubmissionOutputFilter) -> Select<assignment_submission_output::Entity> {
        let mut condition = Condition::all();

        if let Some(id) = filter.id {
            condition = condition.add(assignment_submission_output::Column::Id.eq(id));
        }

        if let Some(task_id) = filter.task_id {
            condition = condition.add(assignment_submission_output::Column::TaskId.eq(task_id));
        }

        if let Some(submission_id) = filter.submission_id {
            condition = condition.add(assignment_submission_output::Column::SubmissionId.eq(submission_id));
        }

        if let Some(ref query_text) = filter.query {
            let pattern = format!("%{}%", query_text.to_lowercase());
            let search_condition = Condition::any()
                .add(Expr::cust("LOWER(path)").like(&pattern)); // Assuming 'path' is the only string field to search
            condition = condition.add(search_condition);
        }

        query.filter(condition)
    }

    fn apply_sorting(mut query: Select<assignment_submission_output::Entity>, sort_by: Option<String>) -> Select<assignment_submission_output::Entity> {
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
                            query.order_by_asc(assignment_submission_output::Column::Id)
                        } else {
                            query.order_by_desc(assignment_submission_output::Column::Id)
                        }
                    }
                    "task_id" => {
                        if asc {
                            query.order_by_asc(assignment_submission_output::Column::TaskId)
                        } else {
                            query.order_by_desc(assignment_submission_output::Column::TaskId)
                        }
                    }
                    "submission_id" => {
                        if asc {
                            query.order_by_asc(assignment_submission_output::Column::SubmissionId)
                        } else {
                            query.order_by_desc(assignment_submission_output::Column::SubmissionId)
                        }
                    }
                    "created_at" => {
                        if asc {
                            query.order_by_asc(assignment_submission_output::Column::CreatedAt)
                        } else {
                            query.order_by_desc(assignment_submission_output::Column::CreatedAt)
                        }
                    }
                    "updated_at" => {
                        if asc {
                            query.order_by_asc(assignment_submission_output::Column::UpdatedAt)
                        } else {
                            query.order_by_desc(assignment_submission_output::Column::UpdatedAt)
                        }
                    }
                    _ => query,
                };
            }
        }
        query
    }
}