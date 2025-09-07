use crate::models::password_reset_token;
use crate::repositories::repository::Repository;
use crate::comparisons::ApplyComparison;
use crate::filters::PasswordResetTokenFilter;
use sea_orm::{QueryFilter, QueryOrder, Select, Condition};
use chrono::{DateTime, Utc};

pub struct PasswordResetTokenRepository;

impl PasswordResetTokenRepository {}

impl Repository<password_reset_token::Entity, PasswordResetTokenFilter> for PasswordResetTokenRepository {
    fn apply_filter(query: Select<password_reset_token::Entity>, filter: &PasswordResetTokenFilter) -> Select<password_reset_token::Entity> {
        let mut condition = Condition::all();
        if let Some(id) = &filter.id {
            condition = i64::apply_comparison(condition, password_reset_token::Column::Id, &id);
        }
        if let Some(user_id) = &filter.user_id {
            condition = i64::apply_comparison(condition, password_reset_token::Column::UserId, &user_id);
        }
        if let Some(token) = &filter.token {
            condition = String::apply_comparison(condition, password_reset_token::Column::Token, token);
        }
        if let Some(expires_at) = &filter.expires_at {
            condition = DateTime::<Utc>::apply_comparison(condition, password_reset_token::Column::ExpiresAt, &expires_at);
        }
        if let Some(used) = &filter.used {
            condition = bool::apply_comparison(condition, password_reset_token::Column::Used, &used);
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