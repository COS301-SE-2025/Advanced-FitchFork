use crate::models::assignment_file::{Entity, Column, FileType};
use crate::repositories::repository::Repository;
use crate::filters::AssignmentFileFilter;
use crate::comparisons::ApplyComparison;
use sea_orm::{QueryFilter, QueryOrder, Select, Condition};
pub struct AssignmentFileRepository;

impl AssignmentFileRepository {}

impl Repository<Entity, AssignmentFileFilter> for AssignmentFileRepository {
    fn apply_filter(query: Select<Entity>, filter: &AssignmentFileFilter) -> Select<Entity> {
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
        if let Some(file_type) = &filter.file_type {
            condition = FileType::apply_comparison(condition, Column::FileType, &file_type);
        }
        query.filter(condition)
    }

    fn apply_sorting(mut query: Select<Entity>, sort_by: Option<String>) -> Select<Entity> {
        if let Some(sort) = sort_by {
            let (column, asc) = if sort.starts_with('-') {
                (&sort[1..], false)
            } else {
                (sort.as_str(), true)
            };

            query = match column {
                "filename" => {
                    if asc {
                        query.order_by_asc(Column::Filename)
                    } else {
                        query.order_by_desc(Column::Filename)
                    }
                }
                "file_type" => {
                    if asc {
                        query.order_by_asc(Column::FileType)
                    } else {
                        query.order_by_desc(Column::FileType)
                    }
                }
                _ => query,
            };
        }
        query
    }
}