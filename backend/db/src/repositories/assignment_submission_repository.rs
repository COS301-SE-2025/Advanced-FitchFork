use crate::models::assignment_submission::{Entity, Column};
use crate::repositories::repository::Repository;
use crate::filter_utils::{FilterUtils, SortUtils};
use util::filters::FilterParam;
use sea_orm::{QueryFilter, Select, DbErr};
use std::str::FromStr;

pub struct AssignmentSubmissionRepository;

impl Repository<Entity> for AssignmentSubmissionRepository {
    fn apply_filter(query: Select<Entity>, filter_params: &[FilterParam]) -> Result<Select<Entity>, DbErr> {
        let condition = FilterUtils::apply_all_filters(filter_params, |column_name| {
            Column::from_str(column_name)
                .map_err(|_| DbErr::Custom(format!("Invalid column name: {}", column_name)))
        })?;
        
        Ok(query.filter(condition))
    }

    fn apply_sorting(query: Select<Entity>, sort_by: Option<String>) -> Select<Entity> {
        SortUtils::apply_sorting(query.clone(), sort_by, |column_name| {
            Column::from_str(column_name)
                .map_err(|_| DbErr::Custom(format!("Invalid column name: {}", column_name)))
        }).unwrap_or(query)
    }
}