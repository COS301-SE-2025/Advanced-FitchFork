use crate::models::announcements::{Column, Entity};
use crate::repositories::repository::Repository;
use crate::comparisons::ApplyComparison;
use crate::filters::AnnouncementFilter;
use sea_orm::{QueryFilter, QueryOrder, Select, Condition};

pub struct AnnouncementRepository;

impl AnnouncementRepository {}

impl Repository<Entity, AnnouncementFilter> for AnnouncementRepository {
    fn apply_filter(query: Select<Entity>, filter: &AnnouncementFilter) -> Select<Entity> {
        let mut condition = Condition::all();
        if let Some(id) = &filter.id {
            condition = i64::apply_comparison(condition, Column::Id, &id);
        }
        if let Some(module_id) = &filter.module_id {
            condition = i64::apply_comparison(condition, Column::ModuleId, &module_id);
        }
        if let Some(user_id) = &filter.user_id {
            condition = i64::apply_comparison(condition, Column::UserId, &user_id);
        }
        if let Some(title) = &filter.title {
            condition = String::apply_comparison(condition, Column::Title, &title);
        }
        if let Some(body) = &filter.body {
            condition = String::apply_comparison(condition, Column::Body, &body);
        }
        if let Some(pinned) = &filter.pinned {
            condition = bool::apply_comparison(condition, Column::Pinned, &pinned);
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
                            query.order_by_asc(Column::ModuleId)
                        } else {
                            query.order_by_desc(Column::ModuleId)
                        }
                    }
                    "user_id" => {
                        if asc {
                            query.order_by_asc(Column::UserId)
                        } else {
                            query.order_by_desc(Column::UserId)
                        }
                    }
                    "title" => {
                        if asc {
                            query.order_by_asc(Column::Title)
                        } else {
                            query.order_by_desc(Column::Title)
                        }
                    }
                    "body" => {
                        if asc {
                            query.order_by_asc(Column::Body)
                        } else {
                            query.order_by_desc(Column::Body)
                        }
                    }
                    "pinned" => {
                        if asc {
                            query.order_by_asc(Column::Pinned)
                        } else {
                            query.order_by_desc(Column::Pinned)
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