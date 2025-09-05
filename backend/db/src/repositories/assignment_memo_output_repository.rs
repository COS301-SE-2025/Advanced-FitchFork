use crate::models::assignment_memo_output;
use crate::repositories::repository::Repository;
use crate::filters::AssignmentMemoOutputFilter;
use sea_orm::{prelude::Expr, QueryFilter, QueryOrder, ColumnTrait, Select, Condition};

pub struct AssignmentMemoOutputRepository;

impl AssignmentMemoOutputRepository {}

impl Repository<assignment_memo_output::Entity, AssignmentMemoOutputFilter> for AssignmentMemoOutputRepository {
    fn apply_filter(query: Select<assignment_memo_output::Entity>, filter: &AssignmentMemoOutputFilter) -> Select<assignment_memo_output::Entity> {
        let mut condition = Condition::all();

        if let Some(id) = filter.id {
            condition = condition.add(assignment_memo_output::Column::Id.eq(id));
        }

        if let Some(assignment_id) = filter.assignment_id {
            condition = condition.add(assignment_memo_output::Column::AssignmentId.eq(assignment_id));
        }

        if let Some(task_id) = filter.task_id {
            condition = condition.add(assignment_memo_output::Column::TaskId.eq(task_id));
        }

        if let Some(ref query_text) = filter.query {
            let pattern = format!("%{}%", query_text.to_lowercase());
            let search_condition = Condition::any()
                .add(Expr::cust("LOWER(path)").like(&pattern));
            condition = condition.add(search_condition);
        }

        query.filter(condition)
    }

    fn apply_sorting(mut query: Select<assignment_memo_output::Entity>, sort_by: Option<String>) -> Select<assignment_memo_output::Entity> {
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
                            query.order_by_asc(assignment_memo_output::Column::Id)
                        } else {
                            query.order_by_desc(assignment_memo_output::Column::Id)
                        }
                    }
                    "assignment_id" => {
                        if asc {
                            query.order_by_asc(assignment_memo_output::Column::AssignmentId)
                        } else {
                            query.order_by_desc(assignment_memo_output::Column::AssignmentId)
                        }
                    }
                    "task_id" => {
                        if asc {
                            query.order_by_asc(assignment_memo_output::Column::TaskId)
                        } else {
                            query.order_by_desc(assignment_memo_output::Column::TaskId)
                        }
                    }
                    "created_at" => {
                        if asc {
                            query.order_by_asc(assignment_memo_output::Column::CreatedAt)
                        } else {
                            query.order_by_desc(assignment_memo_output::Column::CreatedAt)
                        }
                    }
                    "updated_at" => {
                        if asc {
                            query.order_by_asc(assignment_memo_output::Column::UpdatedAt)
                        } else {
                            query.order_by_desc(assignment_memo_output::Column::UpdatedAt)
                        }
                    }
                    _ => query,
                };
            }
        }
        query
    }
}