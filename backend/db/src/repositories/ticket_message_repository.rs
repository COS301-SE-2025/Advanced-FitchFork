use crate::models::ticket_messages::{Entity, Column};
use crate::repositories::repository::Repository;
use crate::comparisons::ApplyComparison;
use crate::filters::TicketMessageFilter;
use sea_orm::{QueryFilter, QueryOrder, Select, Condition};

pub struct TicketMessageRepository;

impl TicketMessageRepository {}

impl Repository<Entity, TicketMessageFilter> for TicketMessageRepository {
    fn apply_filter(query: Select<Entity>, filter: &TicketMessageFilter) -> Select<Entity> {
        let mut condition = Condition::all();
        if let Some(id) = &filter.id {
            condition = i64::apply_comparison(condition, Column::Id, &id);
        }
        if let Some(ticket_id) = &filter.ticket_id {
            condition = i64::apply_comparison(condition, Column::TicketId, &ticket_id);
        }
        if let Some(user_id) = &filter.user_id {
            condition = i64::apply_comparison(condition, Column::UserId, &user_id);
        }
        if let Some(content) = &filter.content {
            condition = String::apply_comparison(condition, Column::Content, &content);
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
                    "ticket_id" => {
                        if asc {
                            query.order_by_asc(Column::TicketId)
                        } else {
                            query.order_by_desc(Column::TicketId)
                        }
                    }
                    "user_id" => {
                        if asc {
                            query.order_by_asc(Column::UserId)
                        } else {
                            query.order_by_desc(Column::UserId)
                        }
                    }
                    "content" => {
                        if asc {
                            query.order_by_asc(Column::Content)
                        } else {
                            query.order_by_desc(Column::Content)
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