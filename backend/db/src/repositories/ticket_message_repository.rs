use crate::models::ticket_messages;
use crate::repositories::repository::Repository;
use crate::filters::TicketMessageFilter;
use sea_orm::{prelude::Expr, QueryFilter, QueryOrder, ColumnTrait, Select, Condition};

pub struct TicketMessageRepository;

impl TicketMessageRepository {}

impl Repository<ticket_messages::Entity, TicketMessageFilter> for TicketMessageRepository {
    fn apply_filter(query: Select<ticket_messages::Entity>, filter: &TicketMessageFilter) -> Select<ticket_messages::Entity> {
        let mut condition = Condition::all();

        if let Some(id) = filter.id {
            condition = condition.add(ticket_messages::Column::Id.eq(id));
        }

        if let Some(ticket_id) = filter.ticket_id {
            condition = condition.add(ticket_messages::Column::TicketId.eq(ticket_id));
        }

        if let Some(user_id) = filter.user_id {
            condition = condition.add(ticket_messages::Column::UserId.eq(user_id));
        }

        if let Some(ref content) = filter.content {
            let pattern = format!("%{}%", content.to_lowercase());
            condition = condition.add(Expr::cust("LOWER(content)").like(&pattern));
        }

        if let Some(ref query_text) = filter.query {
            let pattern = format!("%{}%", query_text.to_lowercase());
            let search_condition = Condition::any()
                .add(Expr::cust("LOWER(content)").like(&pattern));
            condition = condition.add(search_condition);
        }

        query.filter(condition)
    }

    fn apply_sorting(mut query: Select<ticket_messages::Entity>, sort_by: Option<String>) -> Select<ticket_messages::Entity> {
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
                            query.order_by_asc(ticket_messages::Column::Id)
                        } else {
                            query.order_by_desc(ticket_messages::Column::Id)
                        }
                    }
                    "ticket_id" => {
                        if asc {
                            query.order_by_asc(ticket_messages::Column::TicketId)
                        } else {
                            query.order_by_desc(ticket_messages::Column::TicketId)
                        }
                    }
                    "user_id" => {
                        if asc {
                            query.order_by_asc(ticket_messages::Column::UserId)
                        } else {
                            query.order_by_desc(ticket_messages::Column::UserId)
                        }
                    }
                    "content" => {
                        if asc {
                            query.order_by_asc(ticket_messages::Column::Content)
                        } else {
                            query.order_by_desc(ticket_messages::Column::Content)
                        }
                    }
                    "created_at" => {
                        if asc {
                            query.order_by_asc(ticket_messages::Column::CreatedAt)
                        } else {
                            query.order_by_desc(ticket_messages::Column::CreatedAt)
                        }
                    }
                    "updated_at" => {
                        if asc {
                            query.order_by_asc(ticket_messages::Column::UpdatedAt)
                        } else {
                            query.order_by_desc(ticket_messages::Column::UpdatedAt)
                        }
                    }
                    _ => query,
                };
            }
        }
        query
    }
}