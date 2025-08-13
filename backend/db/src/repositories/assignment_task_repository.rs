use crate::models::assignment_task;
use crate::repositories::repository::Repository;
use crate::filters::AssignmentTaskFilter;
use sea_orm::{prelude::Expr, QueryFilter, QueryOrder, ColumnTrait, DatabaseConnection, Select, Condition};

#[derive(Clone)]
pub struct AssignmentTaskRepository {
    db: DatabaseConnection,
}

impl AssignmentTaskRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl Repository<assignment_task::Entity, AssignmentTaskFilter> for AssignmentTaskRepository {
    fn db(&self) -> &DatabaseConnection {
        &self.db
    }

    fn apply_filter(query: Select<assignment_task::Entity>, filter: &AssignmentTaskFilter) -> Select<assignment_task::Entity> {
        let mut condition = Condition::all();

        if let Some(id) = filter.id {
            condition = condition.add(assignment_task::Column::Id.eq(id));
        }

        if let Some(assignment_id) = filter.assignment_id {
            condition = condition.add(assignment_task::Column::AssignmentId.eq(assignment_id));
        }

        if let Some(task_number) = filter.task_number {
            condition = condition.add(assignment_task::Column::TaskNumber.eq(task_number));
        }

        if let Some(ref name) = filter.name {
            let pattern = format!("%{}%", name.to_lowercase());
            condition = condition.add(Expr::cust("LOWER(name)").like(&pattern));
        }

        if let Some(ref command) = filter.command {
            let pattern = format!("%{}%", command.to_lowercase());
            condition = condition.add(Expr::cust("LOWER(command)").like(&pattern));
        }

        if let Some(ref query_text) = filter.query {
            let pattern = format!("%{}%", query_text.to_lowercase());
            let search_condition = Condition::any()
                .add(Expr::cust("LOWER(name)").like(&pattern))
                .add(Expr::cust("LOWER(command)").like(&pattern));
            condition = condition.add(search_condition);
        }

        query.filter(condition)
    }

    fn apply_sorting(mut query: Select<assignment_task::Entity>, sort_by: Option<String>) -> Select<assignment_task::Entity> {
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
                            query.order_by_asc(assignment_task::Column::Id)
                        } else {
                            query.order_by_desc(assignment_task::Column::Id)
                        }
                    }
                    "assignment_id" => {
                        if asc {
                            query.order_by_asc(assignment_task::Column::AssignmentId)
                        } else {
                            query.order_by_desc(assignment_task::Column::AssignmentId)
                        }
                    }
                    "task_number" => {
                        if asc {
                            query.order_by_asc(assignment_task::Column::TaskNumber)
                        } else {
                            query.order_by_desc(assignment_task::Column::TaskNumber)
                        }
                    }
                    "name" => {
                        if asc {
                            query.order_by_asc(assignment_task::Column::Name)
                        } else {
                            query.order_by_desc(assignment_task::Column::Name)
                        }
                    }
                    "command" => {
                        if asc {
                            query.order_by_asc(assignment_task::Column::Command)
                        } else {
                            query.order_by_desc(assignment_task::Column::Command)
                        }
                    }
                    "created_at" => {
                        if asc {
                            query.order_by_asc(assignment_task::Column::CreatedAt)
                        } else {
                            query.order_by_desc(assignment_task::Column::CreatedAt)
                        }
                    }
                    "updated_at" => {
                        if asc {
                            query.order_by_asc(assignment_task::Column::UpdatedAt)
                        } else {
                            query.order_by_desc(assignment_task::Column::UpdatedAt)
                        }
                    }
                    _ => query,
                };
            }
        }
        query
    }
}