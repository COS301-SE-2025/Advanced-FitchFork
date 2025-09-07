use crate::models::assignment::{Entity, Column, AssignmentType, Status};
use crate::repositories::repository::Repository;
use crate::comparisons::ApplyComparison;
use crate::filters::AssignmentFilter;
use sea_orm::{QueryFilter, QueryOrder, Select, Condition};
use chrono::{DateTime, Utc};

pub struct AssignmentRepository;

impl AssignmentRepository {}

impl Repository<Entity, AssignmentFilter> for AssignmentRepository {
    fn apply_filter(query: Select<Entity>, filter: &AssignmentFilter) -> Select<Entity> {
        let mut condition = Condition::all();
        if let Some(id) = &filter.id {
            condition = i64::apply_comparison(condition, Column::Id, &id);
        }
        if let Some(module_id) = &filter.module_id {
            condition = i64::apply_comparison(condition, Column::ModuleId, &module_id);
        }
        if let Some(name) = &filter.name {
            condition = String::apply_comparison(condition, Column::Name, &name);
        }
        if let Some(description) = &filter.description {
            condition = String::apply_comparison(condition, Column::Description, &description);
        }
        if let Some(assignment_type) = &filter.assignment_type {
            condition = AssignmentType::apply_comparison(condition, Column::AssignmentType, &assignment_type);
        }
        if let Some(status) = &filter.status {
            condition = Status::apply_comparison(condition, Column::Status, &status);
        }
        if let Some(available_before) = &filter.available_before {
            condition = DateTime::<Utc>::apply_comparison(condition, Column::AvailableFrom, &available_before);
        }
        if let Some(available_after) = &filter.available_after {
            condition = DateTime::<Utc>::apply_comparison(condition, Column::AvailableFrom, &available_after);
        }
        if let Some(due_before) = &filter.due_before {
            condition = DateTime::<Utc>::apply_comparison(condition, Column::DueDate, &due_before);
        }
        if let Some(due_after) = &filter.due_after {
            condition = DateTime::<Utc>::apply_comparison(condition, Column::DueDate, &due_after);
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
                    "name" => {
                        if asc {
                            query.order_by_asc(Column::Name)
                        } else {
                            query.order_by_desc(Column::Name)
                        }
                    }
                    "description" => {
                        if asc {
                            query.order_by_asc(Column::Description)
                        } else {
                            query.order_by_desc(Column::Description)
                        }
                    }
                    "assignment_type" => {
                        if asc {
                            query.order_by_asc(Column::AssignmentType)
                        } else {
                            query.order_by_desc(Column::AssignmentType)
                        }
                    }
                    "status" => {
                        if asc {
                            query.order_by_asc(Column::Status)
                        } else {
                            query.order_by_desc(Column::Status)
                        }
                    }
                    "available_from" => {
                        if asc {
                            query.order_by_asc(Column::AvailableFrom)
                        } else {
                            query.order_by_desc(Column::AvailableFrom)
                        }
                    }
                    "due_date" => {
                        if asc {
                            query.order_by_asc(Column::DueDate)
                        } else {
                            query.order_by_desc(Column::DueDate)
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