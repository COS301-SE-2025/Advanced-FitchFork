use crate::models::assignment_overwrite_file;
use crate::repositories::repository::Repository;
use crate::filters::AssignmentOverwriteFilter;
use sea_orm::{prelude::Expr, QueryFilter, QueryOrder, ColumnTrait, DatabaseConnection, Select, Condition};

#[derive(Clone)]
pub struct AssignmentOverwriteRepository {
    db: DatabaseConnection,
}

impl AssignmentOverwriteRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl Repository<assignment_overwrite_file::Entity, AssignmentOverwriteFilter> for AssignmentOverwriteRepository {
    fn db(&self) -> &DatabaseConnection {
        &self.db
    }

    fn apply_filter(query: Select<assignment_overwrite_file::Entity>, filter: &AssignmentOverwriteFilter) -> Select<assignment_overwrite_file::Entity> {
        let mut condition = Condition::all();

        if let Some(id) = filter.id {
            condition = condition.add(assignment_overwrite_file::Column::Id.eq(id));
        }

        if let Some(assignment_id) = filter.assignment_id {
            condition = condition.add(assignment_overwrite_file::Column::AssignmentId.eq(assignment_id));
        }

        if let Some(task_id) = filter.task_id {
            condition = condition.add(assignment_overwrite_file::Column::TaskId.eq(task_id));
        }

        if let Some(ref filename) = filter.filename {
            let pattern = format!("%{}%", filename.to_lowercase());
            condition = condition.add(Expr::cust("LOWER(filename)").like(&pattern));
        }

        if let Some(ref query_text) = filter.query {
            let pattern = format!("%{}%", query_text.to_lowercase());
            let search_condition = Condition::any()
                .add(Expr::cust("LOWER(filename)").like(&pattern));
            condition = condition.add(search_condition);
        }

        query.filter(condition)
    }

    fn apply_sorting(mut query: Select<assignment_overwrite_file::Entity>, sort_by: Option<String>) -> Select<assignment_overwrite_file::Entity> {
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
                            query.order_by_asc(assignment_overwrite_file::Column::Id)
                        } else {
                            query.order_by_desc(assignment_overwrite_file::Column::Id)
                        }
                    }
                    "assignment_id" => {
                        if asc {
                            query.order_by_asc(assignment_overwrite_file::Column::AssignmentId)
                        } else {
                            query.order_by_desc(assignment_overwrite_file::Column::AssignmentId)
                        }
                    }
                    "task_id" => {
                        if asc {
                            query.order_by_asc(assignment_overwrite_file::Column::TaskId)
                        } else {
                            query.order_by_desc(assignment_overwrite_file::Column::TaskId)
                        }
                    }
                    "filename" => {
                        if asc {
                            query.order_by_asc(assignment_overwrite_file::Column::Filename)
                        } else {
                            query.order_by_desc(assignment_overwrite_file::Column::Filename)
                        }
                    }
                    "created_at" => {
                        if asc {
                            query.order_by_asc(assignment_overwrite_file::Column::CreatedAt)
                        } else {
                            query.order_by_desc(assignment_overwrite_file::Column::CreatedAt)
                        }
                    }
                    "updated_at" => {
                        if asc {
                            query.order_by_asc(assignment_overwrite_file::Column::UpdatedAt)
                        } else {
                            query.order_by_desc(assignment_overwrite_file::Column::UpdatedAt)
                        }
                    }
                    _ => query,
                };
            }
        }
        query
    }
}