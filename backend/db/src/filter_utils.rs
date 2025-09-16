use sea_orm::{Condition, QueryOrder, ColumnTrait, DbErr, prelude::Expr};
use util::filters::{FilterParam, QueryParam, FilterValue, CompareOp};

pub struct FilterUtils;

impl FilterUtils {
    pub fn apply_filter<C>(
        condition: Condition, 
        column: C, 
        filter_param: &FilterParam
    ) -> Result<Condition, DbErr> 
    where 
        C: ColumnTrait
    {
        if filter_param.value.is_empty() {
            return Err(DbErr::Custom("Filter value cannot be empty".to_string()));
        }

        match (&filter_param.value, &filter_param.operator) {
            // String operations
            (FilterValue::String(values), CompareOp::Eq) => {
                if values.len() == 1 {
                    Ok(condition.add(column.eq(&values[0])))
                } else {
                    Ok(condition.add(column.is_in(values.clone())))
                }
            },
            (FilterValue::String(values), CompareOp::Ne) => {
                if values.len() == 1 {
                    Ok(condition.add(column.ne(&values[0])))
                } else {
                    Ok(condition.add(column.is_not_in(values.clone())))
                }
            },
            (FilterValue::String(values), CompareOp::Like) => {
                let pattern = format!("%{}%", values[0].to_lowercase());
                Ok(condition.add(
                    Expr::cust(&format!("LOWER({})", column.as_str())).like(&pattern)
                ))
            },
            (FilterValue::String(values), CompareOp::Gt) => {
                if values.len() != 1 {
                    return Err(DbErr::Custom("Greater than comparison requires exactly one value".to_string()));
                }
                Ok(condition.add(column.gt(&values[0])))
            },
            (FilterValue::String(values), CompareOp::Gte) => {
                if values.len() != 1 {
                    return Err(DbErr::Custom("Greater than or equal comparison requires exactly one value".to_string()));
                }
                Ok(condition.add(column.gte(&values[0])))
            },
            (FilterValue::String(values), CompareOp::Lt) => {
                if values.len() != 1 {
                    return Err(DbErr::Custom("Less than comparison requires exactly one value".to_string()));
                }
                Ok(condition.add(column.lt(&values[0])))
            },
            (FilterValue::String(values), CompareOp::Lte) => {
                if values.len() != 1 {
                    return Err(DbErr::Custom("Less than or equal comparison requires exactly one value".to_string()));
                }
                Ok(condition.add(column.lte(&values[0])))
            },
            
            // Integer operations
            (FilterValue::Int(values), CompareOp::Eq) => {
                if values.len() == 1 {
                    Ok(condition.add(column.eq(values[0])))
                } else {
                    Ok(condition.add(column.is_in(values.clone())))
                }
            },
            (FilterValue::Int(values), CompareOp::Ne) => {
                if values.len() == 1 {
                    Ok(condition.add(column.ne(values[0])))
                } else {
                    Ok(condition.add(column.is_not_in(values.clone())))
                }
            },
            (FilterValue::Int(values), CompareOp::Gt) => {
                if values.len() != 1 {
                    return Err(DbErr::Custom("Greater than comparison requires exactly one value".to_string()));
                }
                Ok(condition.add(column.gt(values[0])))
            },
            (FilterValue::Int(values), CompareOp::Gte) => {
                if values.len() != 1 {
                    return Err(DbErr::Custom("Greater than or equal comparison requires exactly one value".to_string()));
                }
                Ok(condition.add(column.gte(values[0])))
            },
            (FilterValue::Int(values), CompareOp::Lt) => {
                if values.len() != 1 {
                    return Err(DbErr::Custom("Less than comparison requires exactly one value".to_string()));
                }
                Ok(condition.add(column.lt(values[0])))
            },
            (FilterValue::Int(values), CompareOp::Lte) => {
                if values.len() != 1 {
                    return Err(DbErr::Custom("Less than or equal comparison requires exactly one value".to_string()));
                }
                Ok(condition.add(column.lte(values[0])))
            },
            (FilterValue::Int(values), CompareOp::Like) => {
                let pattern = format!("%{}%", values[0]);
                Ok(condition.add(column.like(&pattern)))
            },

            // Float operations
            (FilterValue::Float(values), CompareOp::Eq) => {
                if values.len() == 1 {
                    Ok(condition.add(column.eq(values[0])))
                } else {
                    Ok(condition.add(column.is_in(values.clone())))
                }
            },
            (FilterValue::Float(values), CompareOp::Ne) => {
                if values.len() == 1 {
                    Ok(condition.add(column.ne(values[0])))
                } else {
                    Ok(condition.add(column.is_not_in(values.clone())))
                }
            },
            (FilterValue::Float(values), CompareOp::Gt) => {
                if values.len() != 1 {
                    return Err(DbErr::Custom("Greater than comparison requires exactly one value".to_string()));
                }
                Ok(condition.add(column.gt(values[0])))
            },
            (FilterValue::Float(values), CompareOp::Gte) => {
                if values.len() != 1 {
                    return Err(DbErr::Custom("Greater than or equal comparison requires exactly one value".to_string()));
                }
                Ok(condition.add(column.gte(values[0])))
            },
            (FilterValue::Float(values), CompareOp::Lt) => {
                if values.len() != 1 {
                    return Err(DbErr::Custom("Less than comparison requires exactly one value".to_string()));
                }
                Ok(condition.add(column.lt(values[0])))
            },
            (FilterValue::Float(values), CompareOp::Lte) => {
                if values.len() != 1 {
                    return Err(DbErr::Custom("Less than or equal comparison requires exactly one value".to_string()));
                }
                Ok(condition.add(column.lte(values[0])))
            },
            (FilterValue::Float(values), CompareOp::Like) => {
                let pattern = format!("%{}%", values[0]);
                Ok(condition.add(column.like(&pattern)))
            },
            
            // Boolean operations
            (FilterValue::Bool(values), CompareOp::Eq) => {
                if values.len() == 1 {
                    Ok(condition.add(column.eq(values[0])))
                } else {
                    Ok(condition.add(column.is_in(values.clone())))
                }
            },
            (FilterValue::Bool(values), CompareOp::Ne) => {
                if values.len() == 1 {
                    Ok(condition.add(column.ne(values[0])))
                } else {
                    Ok(condition.add(column.is_not_in(values.clone())))
                }
            },
            
            // DateTime operations
            (FilterValue::DateTime(values), CompareOp::Eq) => {
                if values.len() == 1 {
                    Ok(condition.add(column.eq(values[0])))
                } else {
                    Ok(condition.add(column.is_in(values.clone())))
                }
            },
            (FilterValue::DateTime(values), CompareOp::Ne) => {
                if values.len() == 1 {
                    Ok(condition.add(column.ne(values[0])))
                } else {
                    Ok(condition.add(column.is_not_in(values.clone())))
                }
            },
            (FilterValue::DateTime(values), CompareOp::Gt) => {
                if values.len() != 1 {
                    return Err(DbErr::Custom("Greater than comparison requires exactly one value".to_string()));
                }
                Ok(condition.add(column.gt(values[0])))
            },
            (FilterValue::DateTime(values), CompareOp::Gte) => {
                if values.len() != 1 {
                    return Err(DbErr::Custom("Greater than or equal comparison requires exactly one value".to_string()));
                }
                Ok(condition.add(column.gte(values[0])))
            },
            (FilterValue::DateTime(values), CompareOp::Lt) => {
                if values.len() != 1 {
                    return Err(DbErr::Custom("Less than comparison requires exactly one value".to_string()));
                }
                Ok(condition.add(column.lt(values[0])))
            },
            (FilterValue::DateTime(values), CompareOp::Lte) => {
                if values.len() != 1 {
                    return Err(DbErr::Custom("Less than or equal comparison requires exactly one value".to_string()));
                }
                Ok(condition.add(column.lte(values[0])))
            },
            
            // Invalid combinations
            (FilterValue::Bool(_), CompareOp::Gt | CompareOp::Gte | CompareOp::Lt | CompareOp::Lte | CompareOp::Like) => {
                Err(DbErr::Custom(format!("Invalid operator {:?} for boolean value", filter_param.operator)))
            },
            (FilterValue::DateTime(_), CompareOp::Like) => {
                Err(DbErr::Custom("LIKE operator not supported for DateTime values".to_string()))
            },
        }
    }

    pub fn apply_all_filters<C>(
        filter_params: &[FilterParam],
        column_resolver: impl Fn(&str) -> Result<C, DbErr>
    ) -> Result<Condition, DbErr>
    where
        C: ColumnTrait
    {
        let mut condition = Condition::all();
        
        for filter_param in filter_params {
            let column = column_resolver(&filter_param.column)?;
            condition = Self::apply_filter(condition, column, filter_param)?;
        }
        
        Ok(condition)
    }
}

pub struct QueryUtils;

impl QueryUtils {
    pub fn apply_query<C>(
        condition: Condition, 
        query_param: &QueryParam,
        column_resolver: impl Fn(&str) -> Result<C, DbErr>
    ) -> Result<Condition, DbErr>
    where
        C: ColumnTrait
    {
        if query_param.query.trim().is_empty() {
            return Ok(condition);
        }

        if query_param.columns.is_empty() {
            return Err(DbErr::Custom("Query parameter must specify at least one column".to_string()));
        }

        let mut or_condition = Condition::any();
        let pattern = format!("%{}%", query_param.query.to_lowercase());

        for column_name in &query_param.columns {
            let column = column_resolver(column_name)?;
            or_condition = or_condition.add(
                Expr::cust(&format!("LOWER({})", column.as_str())).like(&pattern)
            );
        }

        Ok(condition.add(or_condition))
    }

    pub fn apply_all_queries<C>(
        query_params: &[QueryParam],
        column_resolver: impl Fn(&str) -> Result<C, DbErr>
    ) -> Result<Condition, DbErr>
    where
        C: ColumnTrait
    {
        let mut condition = Condition::all();
        
        for query_param in query_params {
            condition = Self::apply_query(condition, query_param, &column_resolver)?;
        }
        
        Ok(condition)
    }
}

pub struct SortUtils;

impl SortUtils {
    pub fn apply_sorting<E, C>(
        mut query: sea_orm::Select<E>, 
        sort_by: Option<String>,
        column_resolver: impl Fn(&str) -> Result<C, DbErr>
    ) -> Result<sea_orm::Select<E>, DbErr>
    where
        E: sea_orm::EntityTrait,
        C: ColumnTrait
    {
        if let Some(sort) = sort_by {
            let (column_name, asc) = if sort.starts_with('-') {
                (&sort[1..], false)
            } else {
                (sort.as_str(), true)
            };

            let column = column_resolver(column_name)?;
            query = if asc {
                query.order_by_asc(column)
            } else {
                query.order_by_desc(column)
            };
        }
        Ok(query)
    }
}