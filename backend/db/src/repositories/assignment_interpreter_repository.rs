use crate::models::assignment_interpreter::{Column, Entity};
use crate::repositories::repository::Repository;
use crate::comparisons::ApplyComparison;
use crate::filters::AssignmentInterpreterFilter;
use sea_orm::{QueryFilter, QueryOrder, Select, Condition};

pub struct AssignmentInterpreterRepository;

impl AssignmentInterpreterRepository {}

impl Repository<Entity, AssignmentInterpreterFilter> for AssignmentInterpreterRepository {
    fn apply_filter(query: Select<Entity>, filter: &AssignmentInterpreterFilter) -> Select<Entity> {
        let mut condition = Condition::all();
        if let Some(id) = &filter.id {
            condition = i64::apply_comparison(condition, Column::Id, &id);
        }
        if let Some(assignment_id) = &filter.assignment_id {
            condition = i64::apply_comparison(condition, Column::AssignmentId, &assignment_id);
        }
        if let Some(filename) = &filter.filename {
            condition = String::apply_comparison(condition, Column::Filename, &filename);
        }
        if let Some(path) = &filter.path {
            condition = String::apply_comparison(condition, Column::Path, &path);
        }
        if let Some(command) = &filter.command {
            condition = String::apply_comparison(condition, Column::Command, &command);
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
                    "module_id" => {
                        if asc {
                            query.order_by_asc(Column::AssignmentId)
                        } else {
                            query.order_by_desc(Column::AssignmentId)
                        }
                    }
                    "filename" => {
                        if asc {
                            query.order_by_asc(Column::Filename)
                        } else {
                            query.order_by_desc(Column::Filename)
                        }
                    }
                    "path" => {
                        if asc {
                            query.order_by_asc(Column::Path)
                        } else {
                            query.order_by_desc(Column::Path)
                        }
                    }
                    "command" => {
                        if asc {
                            query.order_by_asc(Column::Command)
                        } else {
                            query.order_by_desc(Column::Command)
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