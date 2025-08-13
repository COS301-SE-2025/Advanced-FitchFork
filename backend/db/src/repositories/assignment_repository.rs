use crate::models::assignment;
use crate::repositories::repository::Repository;
use crate::filters::AssignmentFilter;
use sea_orm::{prelude::Expr, QueryFilter, QueryOrder, ColumnTrait, DatabaseConnection, Select, Condition};

#[derive(Clone)]
pub struct AssignmentRepository {
    db: DatabaseConnection,
}

impl AssignmentRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl Repository<assignment::Entity, AssignmentFilter> for AssignmentRepository {
    fn db(&self) -> &DatabaseConnection {
        &self.db
    }

    fn apply_filter(query: Select<assignment::Entity>, filter: &AssignmentFilter) -> Select<assignment::Entity> {
        let mut condition = Condition::all();

        if let Some(id) = filter.id {
            condition = condition.add(assignment::Column::Id.eq(id));
        }

        if let Some(module_id) = filter.module_id {
            condition = condition.add(assignment::Column::ModuleId.eq(module_id));
        }

        if let Some(ref name) = filter.name {
            let pattern = format!("%{}%", name.to_lowercase());
            condition = condition.add(Expr::cust("LOWER(name)").like(&pattern));
        }

        if let Some(ref description) = filter.description {
            let pattern = format!("%{}%", description.to_lowercase());
            condition = condition.add(Expr::cust("LOWER(description)").like(&pattern));
        }

        if let Some(ref assignment_type) = filter.assignment_type {
            condition = condition.add(assignment::Column::AssignmentType.eq(assignment_type.clone()));
        }

        if let Some(ref status) = filter.status {
            condition = condition.add(assignment::Column::Status.eq(status.clone()));
        }

        if let Some(available_before) = filter.available_before {
            condition = condition.add(assignment::Column::AvailableFrom.lt(available_before));
        }

        if let Some(available_after) = filter.available_after {
            condition = condition.add(assignment::Column::AvailableFrom.gt(available_after));
        }

        if let Some(due_before) = filter.due_before {
            condition = condition.add(assignment::Column::DueDate.lt(due_before));
        }

        if let Some(due_after) = filter.due_after {
            condition = condition.add(assignment::Column::DueDate.gt(due_after));
        }

        if let Some(ref query_text) = filter.query {
            let pattern = format!("%{}%", query_text.to_lowercase());
            let search_condition = Condition::any()
                .add(Expr::cust("LOWER(name)").like(&pattern))
                .add(Expr::cust("LOWER(description)").like(&pattern));
            condition = condition.add(search_condition);
        }

        query.filter(condition)
    }

    fn apply_sorting(mut query: Select<assignment::Entity>, sort_by: Option<String>) -> Select<assignment::Entity> {
        if let Some(sort_param) = sort_by {
            for sort in sort_param.split(',') {
                let (field, asc) = if sort.starts_with('-') {
                    (&sort[1..], false)
                } else {
                    (sort, true)
                };

                query = match field {
                    "name" => {
                        if asc {
                            query.order_by_asc(assignment::Column::Name)
                        } else {
                            query.order_by_desc(assignment::Column::Name)
                        }
                    }
                    "description" => {
                        if asc {
                            query.order_by_asc(assignment::Column::Description)
                        } else {
                            query.order_by_desc(assignment::Column::Description)
                        }
                    }
                    "assignment_type" => {
                        if asc {
                            query.order_by_asc(assignment::Column::AssignmentType)
                        } else {
                            query.order_by_desc(assignment::Column::AssignmentType)
                        }
                    }
                    "status" => {
                        if asc {
                            query.order_by_asc(assignment::Column::Status)
                        } else {
                            query.order_by_desc(assignment::Column::Status)
                        }
                    }
                    "available_from" => {
                        if asc {
                            query.order_by_asc(assignment::Column::AvailableFrom)
                        } else {
                            query.order_by_desc(assignment::Column::AvailableFrom)
                        }
                    }
                    "due_date" => {
                        if asc {
                            query.order_by_asc(assignment::Column::DueDate)
                        } else {
                            query.order_by_desc(assignment::Column::DueDate)
                        }
                    }
                    "created_at" => {
                        if asc {
                            query.order_by_asc(assignment::Column::CreatedAt)
                        } else {
                            query.order_by_desc(assignment::Column::CreatedAt)
                        }
                    }
                    "updated_at" => {
                        if asc {
                            query.order_by_asc(assignment::Column::UpdatedAt)
                        } else {
                            query.order_by_desc(assignment::Column::UpdatedAt)
                        }
                    }
                    _ => query,
                };
            }
        }
        query
    }
}