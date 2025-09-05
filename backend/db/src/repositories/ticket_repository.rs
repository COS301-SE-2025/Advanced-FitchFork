use crate::models::tickets;
use crate::repositories::repository::Repository;
use crate::filters::TicketFilter;
use sea_orm::{prelude::Expr, QueryFilter, QueryOrder, ColumnTrait, Select, Condition};

pub struct TicketRepository;

impl TicketRepository {}

impl Repository<tickets::Entity, TicketFilter> for TicketRepository {
    fn apply_filter(query: Select<tickets::Entity>, filter: &TicketFilter) -> Select<tickets::Entity> {
        let mut condition = Condition::all();

        if let Some(id) = filter.id {
            condition = condition.add(tickets::Column::Id.eq(id));
        }

        if let Some(assignment_id) = filter.assignment_id {
            condition = condition.add(tickets::Column::AssignmentId.eq(assignment_id));
        }

        if let Some(user_id) = filter.user_id {
            condition = condition.add(tickets::Column::UserId.eq(user_id));
        }

        if let Some(ref title) = filter.title {
            let pattern = format!("%{}%", title.to_lowercase());
            condition = condition.add(Expr::cust("LOWER(title)").like(&pattern));
        }

        if let Some(ref description) = filter.description {
            let pattern = format!("%{}%", description.to_lowercase());
            condition = condition.add(Expr::cust("LOWER(description)").like(&pattern));
        }

        if let Some(ref status) = filter.status {
            condition = condition.add(tickets::Column::Status.eq(status.clone()));
        }

        if let Some(ref query_text) = filter.query {
            let pattern = format!("%{}%", query_text.to_lowercase());
            let search_condition = Condition::any()
                .add(Expr::cust("LOWER(title)").like(&pattern))
                .add(Expr::cust("LOWER(description)").like(&pattern));
            condition = condition.add(search_condition);
        }

        query.filter(condition)
    }

    fn apply_sorting(mut query: Select<tickets::Entity>, sort_by: Option<String>) -> Select<tickets::Entity> {
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
                            query.order_by_asc(tickets::Column::Id)
                        } else {
                            query.order_by_desc(tickets::Column::Id)
                        }
                    }
                    "assignment_id" => {
                        if asc {
                            query.order_by_asc(tickets::Column::AssignmentId)
                        } else {
                            query.order_by_desc(tickets::Column::AssignmentId)
                        }
                    }
                    "user_id" => {
                        if asc {
                            query.order_by_asc(tickets::Column::UserId)
                        } else {
                            query.order_by_desc(tickets::Column::UserId)
                        }
                    }
                    "title" => {
                        if asc {
                            query.order_by_asc(tickets::Column::Title)
                        } else {
                            query.order_by_desc(tickets::Column::Title)
                        }
                    }
                    "description" => {
                        if asc {
                            query.order_by_asc(tickets::Column::Description)
                        } else {
                            query.order_by_desc(tickets::Column::Description)
                        }
                    }
                    "status" => {
                        if asc {
                            query.order_by_asc(tickets::Column::Status)
                        } else {
                            query.order_by_desc(tickets::Column::Status)
                        }
                    }
                    "created_at" => {
                        if asc {
                            query.order_by_asc(tickets::Column::CreatedAt)
                        } else {
                            query.order_by_desc(tickets::Column::CreatedAt)
                        }
                    }
                    "updated_at" => {
                        if asc {
                            query.order_by_asc(tickets::Column::UpdatedAt)
                        } else {
                            query.order_by_desc(tickets::Column::UpdatedAt)
                        }
                    }
                    _ => query,
                };
            }
        }
        query
    }
}