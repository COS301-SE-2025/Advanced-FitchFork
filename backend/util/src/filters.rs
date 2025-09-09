use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct FilterParam {
    pub column: String,
    pub operator: CompareOp,
    pub value: FilterValue,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CompareOp {
    Eq,
    Gt,
    Gte,
    Lt,
    Lte,
    Like,
    NotEq,
}

#[derive(Debug, Clone)]
pub enum FilterValue {
    String(String),
    Int(i64),
    Bool(bool),
    DateTime(DateTime<Utc>),
}

impl FilterParam {
    pub fn eq(column: &str, value: FilterValue) -> Self {
        Self { column: column.to_string(), operator: CompareOp::Eq, value }
    }
    
    pub fn like(column: &str, value: FilterValue) -> Self {
        Self { column: column.to_string(), operator: CompareOp::Like, value }
    }
    
    pub fn gt(column: &str, value: FilterValue) -> Self {
        Self { column: column.to_string(), operator: CompareOp::Gt, value }
    }
    
    pub fn gte(column: &str, value: FilterValue) -> Self {
        Self { column: column.to_string(), operator: CompareOp::Gte, value }
    }
    
    pub fn lt(column: &str, value: FilterValue) -> Self {
        Self { column: column.to_string(), operator: CompareOp::Lt, value }
    }
    
    pub fn lte(column: &str, value: FilterValue) -> Self {
        Self { column: column.to_string(), operator: CompareOp::Lte, value }
    }
    
    pub fn not_eq(column: &str, value: FilterValue) -> Self {
        Self { column: column.to_string(), operator: CompareOp::NotEq, value }
    }
}