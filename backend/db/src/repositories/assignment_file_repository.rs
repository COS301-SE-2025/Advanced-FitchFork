use crate::models::assignment_file;
use crate::repositories::repository::Repository;
use crate::filters::AssignmentFileFilter;
use sea_orm::{QueryFilter, QueryOrder, ColumnTrait, Select,};

pub struct AssignmentFileRepository;

impl AssignmentFileRepository {}

impl Repository<assignment_file::Entity, AssignmentFileFilter> for AssignmentFileRepository {
    fn apply_filter(query: Select<assignment_file::Entity>, filter: &AssignmentFileFilter) -> Select<assignment_file::Entity> {
        let mut condition = sea_orm::Condition::all();
        if let Some(id) = filter.id {
            condition = condition.add(assignment_file::Column::Id.eq(id));
        }
        if let Some(assignment_id) = filter.assignment_id {
            condition = condition.add(assignment_file::Column::AssignmentId.eq(assignment_id));
        }
        if let Some(filename) = &filter.filename {
            condition = condition.add(assignment_file::Column::Filename.like(format!("%{}%", filename)));
        }
        if let Some(file_type) = &filter.file_type {
            condition = condition.add(assignment_file::Column::FileType.eq(file_type.clone()));
        }
        if let Some(query_str) = &filter.query {
            condition = condition.add(
                sea_orm::Condition::any()
                    .add(assignment_file::Column::Filename.like(format!("%{}%", query_str))),
            );
        }
        query.filter(condition)
    }

    fn apply_sorting(mut query: Select<assignment_file::Entity>, sort_by: Option<String>) -> Select<assignment_file::Entity> {
        if let Some(sort) = sort_by {
            let (column, asc) = if sort.starts_with('-') {
                (&sort[1..], false)
            } else {
                (sort.as_str(), true)
            };

            query = match column {
                "filename" => {
                    if asc {
                        query.order_by_asc(assignment_file::Column::Filename)
                    } else {
                        query.order_by_desc(assignment_file::Column::Filename)
                    }
                }
                "file_type" => {
                    if asc {
                        query.order_by_asc(assignment_file::Column::FileType)
                    } else {
                        query.order_by_desc(assignment_file::Column::FileType)
                    }
                }
                _ => query,
            };
        }
        query
    }
}