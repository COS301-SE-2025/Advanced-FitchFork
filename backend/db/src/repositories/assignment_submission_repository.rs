use crate::models::assignment_submission;
use crate::repositories::repository::Repository;
use crate::filters::AssignmentSubmissionFilter;
use sea_orm::{prelude::Expr, QueryFilter, QueryOrder, ColumnTrait, Select, Condition};

pub struct AssignmentSubmissionRepository;

impl AssignmentSubmissionRepository {}

impl Repository<assignment_submission::Entity, AssignmentSubmissionFilter> for AssignmentSubmissionRepository {
    fn apply_filter(query: Select<assignment_submission::Entity>, filter: &AssignmentSubmissionFilter) -> Select<assignment_submission::Entity> {
        let mut condition = Condition::all();

        if let Some(id) = filter.id {
            condition = condition.add(assignment_submission::Column::Id.eq(id));
        }

        if let Some(assignment_id) = filter.assignment_id {
            condition = condition.add(assignment_submission::Column::AssignmentId.eq(assignment_id));
        }

        if let Some(user_id) = filter.user_id {
            condition = condition.add(assignment_submission::Column::UserId.eq(user_id));
        }

        if let Some(attempt) = filter.attempt {
            condition = condition.add(assignment_submission::Column::Attempt.eq(attempt));
        }

        if let Some(ref filename) = filter.filename {
            let pattern = format!("%{}%", filename.to_lowercase());
            condition = condition.add(Expr::cust("LOWER(filename)").like(&pattern));
        }

        if let Some(ref file_hash) = filter.file_hash {
            let pattern = format!("%{}%", file_hash.to_lowercase());
            condition = condition.add(Expr::cust("LOWER(file_hash)").like(&pattern));
        }

        if let Some(is_practice) = filter.is_practice {
            condition = condition.add(assignment_submission::Column::IsPractice.eq(is_practice));
        }

        if let Some(ref query_text) = filter.query {
            let pattern = format!("%{}%", query_text.to_lowercase());
            let search_condition = Condition::any()
                .add(Expr::cust("LOWER(filename)").like(&pattern))
                .add(Expr::cust("LOWER(file_hash)").like(&pattern));
            condition = condition.add(search_condition);
        }

        query.filter(condition)
    }

    fn apply_sorting(mut query: Select<assignment_submission::Entity>, sort_by: Option<String>) -> Select<assignment_submission::Entity> {
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
                            query.order_by_asc(assignment_submission::Column::Id)
                        } else {
                            query.order_by_desc(assignment_submission::Column::Id)
                        }
                    }
                    "assignment_id" => {
                        if asc {
                            query.order_by_asc(assignment_submission::Column::AssignmentId)
                        } else {
                            query.order_by_desc(assignment_submission::Column::AssignmentId)
                        }
                    }
                    "user_id" => {
                        if asc {
                            query.order_by_asc(assignment_submission::Column::UserId)
                        } else {
                            query.order_by_desc(assignment_submission::Column::UserId)
                        }
                    }
                    "attempt" => {
                        if asc {
                            query.order_by_asc(assignment_submission::Column::Attempt)
                        } else {
                            query.order_by_desc(assignment_submission::Column::Attempt)
                        }
                    }
                    "filename" => {
                        if asc {
                            query.order_by_asc(assignment_submission::Column::Filename)
                        } else {
                            query.order_by_desc(assignment_submission::Column::Filename)
                        }
                    }
                    "file_hash" => {
                        if asc {
                            query.order_by_asc(assignment_submission::Column::FileHash)
                        } else {
                            query.order_by_desc(assignment_submission::Column::FileHash)
                        }
                    }
                    "is_practice" => {
                        if asc {
                            query.order_by_asc(assignment_submission::Column::IsPractice)
                        } else {
                            query.order_by_desc(assignment_submission::Column::IsPractice)
                        }
                    }
                    "created_at" => {
                        if asc {
                            query.order_by_asc(assignment_submission::Column::CreatedAt)
                        } else {
                            query.order_by_desc(assignment_submission::Column::CreatedAt)
                        }
                    }
                    "updated_at" => {
                        if asc {
                            query.order_by_asc(assignment_submission::Column::UpdatedAt)
                        } else {
                            query.order_by_desc(assignment_submission::Column::UpdatedAt)
                        }
                    }
                    _ => query,
                };
            }
        }
        query
    }
}