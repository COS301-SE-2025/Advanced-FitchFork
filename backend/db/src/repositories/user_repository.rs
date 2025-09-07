use crate::models::user::{Entity, Column};
use crate::repositories::repository::Repository;
use crate::filters::UserFilter;
use crate::comparisons::ApplyComparison;
use sea_orm::{QueryFilter, QueryOrder, Select, Condition};

pub struct UserRepository;

impl UserRepository {}

impl Repository<Entity, UserFilter> for UserRepository {
    fn apply_filter(query: Select<Entity>, filter: &UserFilter) -> Select<Entity> {
        let mut condition = Condition::all();
        if let Some(id) = &filter.id {
            condition = i64::apply_comparison(condition, Column::Id, &id);
        }
        if let Some(username) = &filter.username {
            condition = String::apply_comparison(condition, Column::Username, &username);
        }
        if let Some(email) = &filter.email {
            condition = String::apply_comparison(condition, Column::Email, &email);
        }
        if let Some(admin) = &filter.admin {
            condition = bool::apply_comparison(condition, Column::Admin, &admin);
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
                "username" => {
                    if asc {
                        query.order_by_asc(Column::Username)
                    } else {
                        query.order_by_desc(Column::Username)
                    }
                }
                "email" => {
                    if asc {
                        query.order_by_asc(Column::Email)
                    } else {
                        query.order_by_desc(Column::Email)
                    }
                }
                "admin" => {
                    if asc {
                        query.order_by_asc(Column::Admin)
                    } else {
                        query.order_by_desc(Column::Admin)
                    }
                }
                _ => query,
            };
        }
        query
    }
}