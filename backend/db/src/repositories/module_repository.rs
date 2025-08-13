use crate::models::module;
use crate::repositories::repository::Repository;
use crate::filters::ModuleFilter;
use sea_orm::{QueryFilter, QueryOrder, ColumnTrait, DatabaseConnection, Select};

#[derive(Clone)]
pub struct ModuleRepository {
    db: DatabaseConnection,
}

impl ModuleRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl Repository<module::Entity, ModuleFilter> for ModuleRepository {
    fn db(&self) -> &DatabaseConnection {
        &self.db
    }

    fn apply_filter(query: Select<module::Entity>, filter: &ModuleFilter) -> Select<module::Entity> {
        let mut condition = sea_orm::Condition::all();
        if let Some(id) = filter.id {
            condition = condition.add(module::Column::Id.eq(id));
        }
        if let Some(code) = &filter.code {
            condition = condition.add(module::Column::Code.eq(code.clone()));
        }
        if let Some(year) = filter.year {
            condition = condition.add(module::Column::Year.eq(year));
        }
        if let Some(description) = &filter.description {
            condition = condition.add(module::Column::Description.like(format!("%{}%", description)));
        }
        if let Some(credits) = filter.credits {
            condition = condition.add(module::Column::Credits.eq(credits));
        }
        if let Some(query_str) = &filter.query {
            condition = condition.add(
                sea_orm::Condition::any()
                    .add(module::Column::Code.like(format!("%{}%", query_str)))
                    .add(module::Column::Description.like(format!("%{}%", query_str))),
            );
        }
        query.filter(condition)
    }

    fn apply_sorting(mut query: Select<module::Entity>, sort_by: Option<String>) -> Select<module::Entity> {
        if let Some(sort) = sort_by {
            let (column, asc) = if sort.starts_with('-') {
                (&sort[1..], false)
            } else {
                (sort.as_str(), true)
            };

            query = match column {
                "code" => {
                    if asc {
                        query.order_by_asc(module::Column::Code)
                    } else {
                        query.order_by_desc(module::Column::Code)
                    }
                }
                "year" => {
                    if asc {
                        query.order_by_asc(module::Column::Year)
                    } else {
                        query.order_by_desc(module::Column::Year)
                    }
                }
                "description" => {
                    if asc {
                        query.order_by_asc(module::Column::Description)
                    } else {
                        query.order_by_desc(module::Column::Description)
                    }
                }
                "credits" => {
                    if asc {
                        query.order_by_asc(module::Column::Credits)
                    } else {
                        query.order_by_desc(module::Column::Credits)
                    }
                }
                _ => query,
            };
        }
        query
    }
}