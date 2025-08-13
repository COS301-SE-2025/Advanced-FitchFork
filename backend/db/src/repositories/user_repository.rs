use crate::models::user;
use crate::repositories::repository::Repository;
use crate::filters::UserFilter;
use sea_orm::{QueryFilter, QueryOrder, ColumnTrait, DatabaseConnection, Select};

#[derive(Clone)]
pub struct UserRepository {
    db: DatabaseConnection,
}

impl UserRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl Repository<user::Entity, UserFilter> for UserRepository {
    fn db(&self) -> &DatabaseConnection {
        &self.db
    }

    fn apply_filter(query: Select<user::Entity>, filter: &UserFilter) -> Select<user::Entity> {
        let mut condition = sea_orm::Condition::all();
        if let Some(id) = filter.id {
            condition = condition.add(user::Column::Id.eq(id));
        }
        if let Some(username) = &filter.username {
            condition = condition.add(user::Column::Username.eq(username.clone()));
        }
        if let Some(email) = &filter.email {
            condition = condition.add(user::Column::Email.eq(email.clone()));
        }
        if let Some(admin) = filter.admin {
            condition = condition.add(user::Column::Admin.eq(admin));
        }
        if let Some(query_str) = &filter.query {
            condition = condition.add(
                sea_orm::Condition::any()
                    .add(user::Column::Username.like(format!("%{}%", query_str)))
                    .add(user::Column::Email.like(format!("%{}%", query_str))),
            );
        }
        query.filter(condition)
    }

    fn apply_sorting(mut query: Select<user::Entity>, sort_by: Option<String>) -> Select<user::Entity> {
        if let Some(sort) = sort_by {
            let (column, asc) = if sort.starts_with('-') {
                (&sort[1..], false)
            } else {
                (sort.as_str(), true)
            };

            query = match column {
                "username" => {
                    if asc {
                        query.order_by_asc(user::Column::Username)
                    } else {
                        query.order_by_desc(user::Column::Username)
                    }
                }
                "email" => {
                    if asc {
                        query.order_by_asc(user::Column::Email)
                    } else {
                        query.order_by_desc(user::Column::Email)
                    }
                }
                "admin" => {
                    if asc {
                        query.order_by_asc(user::Column::Admin)
                    } else {
                        query.order_by_desc(user::Column::Admin)
                    }
                }
                _ => query,
            };
        }
        query
    }
}