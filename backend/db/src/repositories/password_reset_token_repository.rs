use crate::models::password_reset_token;
use crate::repositories::repository::Repository;
use crate::filters::PasswordResetTokenFilter;
use sea_orm::{prelude::Expr, QueryFilter, QueryOrder, ColumnTrait, Select, Condition};

pub struct PasswordResetTokenRepository;

impl PasswordResetTokenRepository {}

impl Repository<password_reset_token::Entity, PasswordResetTokenFilter> for PasswordResetTokenRepository {
    fn apply_filter(query: Select<password_reset_token::Entity>, filter: &PasswordResetTokenFilter) -> Select<password_reset_token::Entity> {
        let mut condition = Condition::all();

        if let Some(id) = filter.id {
            condition = condition.add(password_reset_token::Column::Id.eq(id));
        }

        if let Some(user_id) = filter.user_id {
            condition = condition.add(password_reset_token::Column::UserId.eq(user_id));
        }

        if let Some(ref token) = filter.token {
            let pattern = format!("%{}%", token.to_lowercase());
            condition = condition.add(Expr::cust("LOWER(token)").like(&pattern));
        }

        if let Some(expires_at) = filter.expires_at {
            condition = condition.add(password_reset_token::Column::ExpiresAt.eq(expires_at));
        }

        if let Some(used) = filter.used {
            condition = condition.add(password_reset_token::Column::Used.eq(used));
        }

        if let Some(ref query_text) = filter.query {
            let pattern = format!("%{}%", query_text.to_lowercase());
            let search_condition = Condition::any()
                .add(Expr::cust("LOWER(token)").like(&pattern));
            condition = condition.add(search_condition);
        }

        query.filter(condition)
    }

    fn apply_sorting(mut query: Select<password_reset_token::Entity>, sort_by: Option<String>) -> Select<password_reset_token::Entity> {
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
                            query.order_by_asc(password_reset_token::Column::Id)
                        } else {
                            query.order_by_desc(password_reset_token::Column::Id)
                        }
                    }
                    "user_id" => {
                        if asc {
                            query.order_by_asc(password_reset_token::Column::UserId)
                        } else {
                            query.order_by_desc(password_reset_token::Column::UserId)
                        }
                    }
                    "token" => {
                        if asc {
                            query.order_by_asc(password_reset_token::Column::Token)
                        } else {
                            query.order_by_desc(password_reset_token::Column::Token)
                        }
                    }
                    "expires_at" => {
                        if asc {
                            query.order_by_asc(password_reset_token::Column::ExpiresAt)
                        } else {
                            query.order_by_desc(password_reset_token::Column::ExpiresAt)
                        }
                    }
                    "used" => {
                        if asc {
                            query.order_by_asc(password_reset_token::Column::Used)
                        } else {
                            query.order_by_desc(password_reset_token::Column::Used)
                        }
                    }
                    "created_at" => {
                        if asc {
                            query.order_by_asc(password_reset_token::Column::CreatedAt)
                        } else {
                            query.order_by_desc(password_reset_token::Column::CreatedAt)
                        }
                    }
                    _ => query,
                };
            }
        }
        query
    }
}