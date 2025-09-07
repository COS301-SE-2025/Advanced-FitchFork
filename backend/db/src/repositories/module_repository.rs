use crate::models::module::{Entity, Column};
use crate::repositories::repository::Repository;
use crate::comparisons::ApplyComparison;
use crate::filters::ModuleFilter;
use sea_orm::{QueryFilter, QueryOrder, Select};

pub struct ModuleRepository;

impl ModuleRepository {}

impl Repository<Entity, ModuleFilter> for ModuleRepository {
    fn apply_filter(query: Select<Entity>, filter: &ModuleFilter) -> Select<Entity> {
        let mut condition = sea_orm::Condition::all();
        if let Some(id) = &filter.id {
            condition = i64::apply_comparison(condition, Column::Id, &id);
        }
        if let Some(code) = &filter.code {
            condition = String::apply_comparison(condition, Column::Code, &code);
        }
        if let Some(year) = &filter.year {
            condition = i64::apply_comparison(condition, Column::Year, &year);
        }
        if let Some(description) = &filter.description {
            condition = String::apply_comparison(condition, Column::Description, &description);
        }
        if let Some(credits) = &filter.credits {
            condition = i64::apply_comparison(condition, Column::Credits, &credits);
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
                "code" => {
                    if asc {
                        query.order_by_asc(Column::Code)
                    } else {
                        query.order_by_desc(Column::Code)
                    }
                }
                "year" => {
                    if asc {
                        query.order_by_asc(Column::Year)
                    } else {
                        query.order_by_desc(Column::Year)
                    }
                }
                "description" => {
                    if asc {
                        query.order_by_asc(Column::Description)
                    } else {
                        query.order_by_desc(Column::Description)
                    }
                }
                "credits" => {
                    if asc {
                        query.order_by_asc(Column::Credits)
                    } else {
                        query.order_by_desc(Column::Credits)
                    }
                }
                _ => query,
            };
        }
        query
    }
}