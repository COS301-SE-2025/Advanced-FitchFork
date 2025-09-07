use crate::models::assignment_submission::{Entity, Column};
use crate::repositories::repository::Repository;
use crate::comparisons::ApplyComparison;
use crate::filters::AssignmentSubmissionFilter;
use sea_orm::{QueryFilter, QueryOrder, Select, Condition};

pub struct AssignmentSubmissionRepository;

impl AssignmentSubmissionRepository {}

impl Repository<Entity, AssignmentSubmissionFilter> for AssignmentSubmissionRepository {
    fn apply_filter(query: Select<Entity>, filter: &AssignmentSubmissionFilter) -> Select<Entity> {
        let mut condition = Condition::all();
        if let Some(id) = &filter.id {
            condition = i64::apply_comparison(condition, Column::Id, &id);
        }
        if let Some(assignment_id) = &filter.assignment_id {
            condition = i64::apply_comparison(condition, Column::AssignmentId, &assignment_id);
        }
        if let Some(user_id) = &filter.user_id {
            condition = i64::apply_comparison(condition, Column::UserId, &user_id);
        }
        if let Some(attempt) = &filter.attempt {
            condition = i64::apply_comparison(condition, Column::Attempt, &attempt);
        }
        if let Some(filename) = &filter.filename {
            condition = String::apply_comparison(condition, Column::Filename, &filename);
        }
        if let Some(file_hash) = &filter.file_hash {
            condition = String::apply_comparison(condition, Column::FileHash, &file_hash);
        }
        if let Some(is_practice) = &filter.is_practice {
            condition = bool::apply_comparison(condition, Column::IsPractice, &is_practice);
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
                    "assignment_id" => {
                        if asc {
                            query.order_by_asc(Column::AssignmentId)
                        } else {
                            query.order_by_desc(Column::AssignmentId)
                        }
                    }
                    "user_id" => {
                        if asc {
                            query.order_by_asc(Column::UserId)
                        } else {
                            query.order_by_desc(Column::UserId)
                        }
                    }
                    "attempt" => {
                        if asc {
                            query.order_by_asc(Column::Attempt)
                        } else {
                            query.order_by_desc(Column::Attempt)
                        }
                    }
                    "filename" => {
                        if asc {
                            query.order_by_asc(Column::Filename)
                        } else {
                            query.order_by_desc(Column::Filename)
                        }
                    }
                    "file_hash" => {
                        if asc {
                            query.order_by_asc(Column::FileHash)
                        } else {
                            query.order_by_desc(Column::FileHash)
                        }
                    }
                    "is_practice" => {
                        if asc {
                            query.order_by_asc(Column::IsPractice)
                        } else {
                            query.order_by_desc(Column::IsPractice)
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