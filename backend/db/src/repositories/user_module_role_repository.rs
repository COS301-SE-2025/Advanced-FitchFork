use crate::models::user_module_role;
use crate::repositories::repository::Repository;
use crate::filters::UserModuleRoleFilter;
use sea_orm::{QueryFilter, QueryOrder, ColumnTrait, DatabaseConnection, Select};

#[derive(Clone)]
pub struct UserModuleRoleRepository {
    db: DatabaseConnection,
}

impl UserModuleRoleRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl Repository<user_module_role::Entity, UserModuleRoleFilter> for UserModuleRoleRepository {
    fn db(&self) -> &DatabaseConnection {
        &self.db
    }

    fn apply_filter(query: Select<user_module_role::Entity>, filter: &UserModuleRoleFilter) -> Select<user_module_role::Entity> {
        let mut condition = sea_orm::Condition::all();
        if let Some(user_id) = filter.user_id {
            condition = condition.add(user_module_role::Column::UserId.eq(user_id));
        }
        if let Some(module_id) = filter.module_id {
            condition = condition.add(user_module_role::Column::ModuleId.eq(module_id));
        }
        if let Some(role) = &filter.role {
            condition = condition.add(user_module_role::Column::Role.eq(role.clone()));
        }
        query.filter(condition)
    }

    fn apply_sorting(mut query: Select<user_module_role::Entity>, sort_by: Option<String>) -> Select<user_module_role::Entity> {
        if let Some(sort) = sort_by {
            let (column, asc) = if sort.starts_with('-') {
                (&sort[1..], false)
            } else {
                (sort.as_str(), true)
            };

            query = match column {
                "user_id" => {
                    if asc {
                        query.order_by_asc(user_module_role::Column::UserId)
                    } else {
                        query.order_by_desc(user_module_role::Column::UserId)
                    }
                }
                "module_id" => {
                    if asc {
                        query.order_by_asc(user_module_role::Column::ModuleId)
                    } else {
                        query.order_by_desc(user_module_role::Column::ModuleId)
                    }
                }
                "role" => {
                    if asc {
                        query.order_by_asc(user_module_role::Column::Role)
                    } else {
                        query.order_by_desc(user_module_role::Column::Role)
                    }
                }
                _ => query,
            };
        }
        query
    }
}