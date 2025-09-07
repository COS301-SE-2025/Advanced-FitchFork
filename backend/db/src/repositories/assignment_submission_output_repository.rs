use crate::models::assignment_submission_output::{Entity, Column};
use crate::repositories::repository::Repository;
use crate::comparisons::ApplyComparison;
use crate::filters::AssignmentSubmissionOutputFilter;
use sea_orm::{QueryFilter, QueryOrder, Select, Condition};

pub struct AssignmentSubmissionOutputRepository;

impl AssignmentSubmissionOutputRepository {}

impl Repository<Entity, AssignmentSubmissionOutputFilter> for AssignmentSubmissionOutputRepository {
    fn apply_filter(query: Select<Entity>, filter: &AssignmentSubmissionOutputFilter) -> Select<Entity> {
        let mut condition = Condition::all();
        if let Some(id) = &filter.id {
            condition = i64::apply_comparison(condition, Column::Id, &id);
        }
        if let Some(task_id) = &filter.task_id {
            condition = i64::apply_comparison(condition, Column::TaskId, &task_id);
        }
        if let Some(submission_id) = &filter.submission_id {
            condition = i64::apply_comparison(condition, Column::SubmissionId, &submission_id);
        }
        query.filter(condition)
    }

    fn apply_sorting(mut query: Select<Entity>, sort_by: Option<String>) -> Select<Entity> {
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
                            query.order_by_asc(Column::Id)
                        } else {
                            query.order_by_desc(Column::Id)
                        }
                    }
                    "task_id" => {
                        if asc {
                            query.order_by_asc(Column::TaskId)
                        } else {
                            query.order_by_desc(Column::TaskId)
                        }
                    }
                    "submission_id" => {
                        if asc {
                            query.order_by_asc(Column::SubmissionId)
                        } else {
                            query.order_by_desc(Column::SubmissionId)
                        }
                    }
                    "created_at" => {
                        if asc {
                            query.order_by_asc(Column::CreatedAt)
                        } else {
                            query.order_by_desc(Column::CreatedAt)
                        }
                    }
                    "updated_at" => {
                        if asc {
                            query.order_by_asc(Column::UpdatedAt)
                        } else {
                            query.order_by_desc(Column::UpdatedAt)
                        }
                    }
                    _ => query,
                };
            }
        }
        query
    }
}