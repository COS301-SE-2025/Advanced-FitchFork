use crate::models::tickets::{Column, Entity, TicketStatus};
use crate::repositories::repository::Repository;
use crate::comparisons::ApplyComparison;
use crate::filters::TicketFilter;
use sea_orm::{QueryFilter, QueryOrder, Select, Condition};

pub struct TicketRepository;

impl TicketRepository {}

impl Repository<Entity, TicketFilter> for TicketRepository {
    fn apply_filter(query: Select<Entity>, filter: &TicketFilter) -> Select<Entity> {
        let mut condition = Condition::all();
        if let Some(id) = &filter.id {
            condition = i64::apply_comparison(condition, Column::Id, &id);
        }
        if let Some(assignment_id) = &filter.assignment_id {
            condition = i64::apply_comparison(condition, Column::AssignmentId, &assignment_id);
        }
        if let Some(user_id) = &filter.user_id {
            condition = i64::apply_comparison(condition, Column::UserId, &user_id);
        }
        if let Some(title) = &filter.title {
            condition = String::apply_comparison(condition, Column::Title, &title);
        }
        if let Some(description) = &filter.description {
            condition = String::apply_comparison(condition, Column::Description, &description);
        }
        if let Some(status) = &filter.status {
            condition = TicketStatus::apply_comparison(condition, Column::Status, &status);
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
                    "assignment_id" => {
                        if asc {
                            query.order_by_asc(Column::AssignmentId)
                        } else {
                            query.order_by_desc(Column::AssignmentId)
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
                    "description" => {
                        if asc {
                            query.order_by_asc(Column::Description)
                        } else {
                            query.order_by_desc(Column::Description)
                        }
                    }
                    "status" => {
                        if asc {
                            query.order_by_asc(Column::Status)
                        } else {
                            query.order_by_desc(Column::Status)
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