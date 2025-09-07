use crate::models::user_module_role::{Entity, Column, Role};
use crate::repositories::repository::Repository;
use crate::comparisons::ApplyComparison;
use crate::filters::UserModuleRoleFilter;
use sea_orm::{QueryFilter, QueryOrder, Select};

pub struct UserModuleRoleRepository;

impl UserModuleRoleRepository {}

impl Repository<Entity, UserModuleRoleFilter> for UserModuleRoleRepository {
    fn apply_filter(query: Select<Entity>, filter: &UserModuleRoleFilter) -> Select<Entity> {
        let mut condition = sea_orm::Condition::all();
        if let Some(user_id) = &filter.user_id {
            condition = i64::apply_comparison(condition, Column::UserId, &user_id);
        }
        if let Some(module_id) = &filter.module_id {
            condition = i64::apply_comparison(condition, Column::ModuleId, &module_id);
        }
        if let Some(role) = &filter.role {
            condition = Role::apply_comparison(condition, Column::Role, &role);
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
                "user_id" => {
                    if asc {
                        query.order_by_asc(Column::UserId)
                    } else {
                        query.order_by_desc(Column::UserId)
                    }
                }
                "module_id" => {
                    if asc {
                        query.order_by_asc(Column::ModuleId)
                    } else {
                        query.order_by_desc(Column::ModuleId)
                    }
                }
                "role" => {
                    if asc {
                        query.order_by_asc(Column::Role)
                    } else {
                        query.order_by_desc(Column::Role)
                    }
                }
                _ => query,
            };
        }
        query
    }
}