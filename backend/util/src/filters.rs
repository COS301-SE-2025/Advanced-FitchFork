use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub enum FilterValue {
    String(Vec<String>),
    Int(Vec<i64>),
    Float(Vec<f32>),
    Bool(Vec<bool>),
    DateTime(Vec<DateTime<Utc>>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CompareOp {
    Eq,
    Gt,
    Gte,
    Lt,
    Lte,
    Like,
    Ne,
}

#[derive(Debug, Clone)]
pub struct FilterParam {
    pub column: String,
    pub operator: CompareOp,
    pub value: FilterValue,
}

// Implementations for single values
pub trait IntoFilterValue<T> {
    fn into_filter_value(self) -> FilterValue;
}

impl IntoFilterValue<String> for String {
    fn into_filter_value(self) -> FilterValue {
        FilterValue::String(vec![self])
    }
}

impl IntoFilterValue<String> for &str {
    fn into_filter_value(self) -> FilterValue {
        FilterValue::String(vec![self.to_string()])
    }
}

impl IntoFilterValue<i64> for i64 {
    fn into_filter_value(self) -> FilterValue {
        FilterValue::Int(vec![self])
    }
}

impl IntoFilterValue<f32> for f32 {
    fn into_filter_value(self) -> FilterValue {
        FilterValue::Float(vec![self])
    }
}

impl IntoFilterValue<bool> for bool {
    fn into_filter_value(self) -> FilterValue {
        FilterValue::Bool(vec![self])
    }
}

impl IntoFilterValue<DateTime<Utc>> for DateTime<Utc> {
    fn into_filter_value(self) -> FilterValue {
        FilterValue::DateTime(vec![self])
    }
}

// Implementations for vectors
impl IntoFilterValue<String> for Vec<String> {
    fn into_filter_value(self) -> FilterValue {
        FilterValue::String(self)
    }
}

impl IntoFilterValue<String> for Vec<&str> {
    fn into_filter_value(self) -> FilterValue {
        FilterValue::String(self.into_iter().map(|s| s.to_string()).collect())
    }
}

impl IntoFilterValue<i64> for Vec<i64> {
    fn into_filter_value(self) -> FilterValue {
        FilterValue::Int(self)
    }
}

impl IntoFilterValue<f32> for Vec<f32> {
    fn into_filter_value(self) -> FilterValue {
        FilterValue::Float(self)
    }
}

impl IntoFilterValue<bool> for Vec<bool> {
    fn into_filter_value(self) -> FilterValue {
        FilterValue::Bool(self)
    }
}

impl IntoFilterValue<DateTime<Utc>> for Vec<DateTime<Utc>> {
    fn into_filter_value(self) -> FilterValue {
        FilterValue::DateTime(self)
    }
}

// Implementations for arrays
impl<const N: usize> IntoFilterValue<String> for [&str; N] {
    fn into_filter_value(self) -> FilterValue {
        FilterValue::String(self.into_iter().map(|s| s.to_string()).collect())
    }
}

impl<const N: usize> IntoFilterValue<i64> for [i64; N] {
    fn into_filter_value(self) -> FilterValue {
        FilterValue::Int(self.to_vec())
    }
}

impl<const N: usize> IntoFilterValue<f32> for [f32; N] {
    fn into_filter_value(self) -> FilterValue {
        FilterValue::Float(self.to_vec())
    }
}

impl<const N: usize> IntoFilterValue<bool> for [bool; N] {
    fn into_filter_value(self) -> FilterValue {
        FilterValue::Bool(self.to_vec())
    }
}

impl FilterValue {
    pub fn is_single(&self) -> bool {
        match self {
            FilterValue::String(v) => v.len() == 1,
            FilterValue::Int(v) => v.len() == 1,
            FilterValue::Float(v) => v.len() == 1,
            FilterValue::Bool(v) => v.len() == 1,
            FilterValue::DateTime(v) => v.len() == 1,
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            FilterValue::String(v) => v.is_empty(),
            FilterValue::Int(v) => v.is_empty(),
            FilterValue::Float(v) => v.is_empty(),
            FilterValue::Bool(v) => v.is_empty(),
            FilterValue::DateTime(v) => v.is_empty(),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            FilterValue::String(v) => v.len(),
            FilterValue::Int(v) => v.len(),
            FilterValue::Float(v) => v.len(),
            FilterValue::Bool(v) => v.len(),
            FilterValue::DateTime(v) => v.len(),
        }
    }
}

impl FilterParam {
    pub fn eq<T>(column: &str, value: impl IntoFilterValue<T>) -> Self {
        Self {
            column: column.to_string(),
            operator: CompareOp::Eq,
            value: value.into_filter_value(),
        }
    }

    pub fn ne<T>(column: &str, value: impl IntoFilterValue<T>) -> Self {
        Self {
            column: column.to_string(),
            operator: CompareOp::Ne,
            value: value.into_filter_value(),
        }
    }

    pub fn like(column: &str, value: impl IntoFilterValue<String>) -> Self {
        let filter_value = value.into_filter_value();
        Self {
            column: column.to_string(),
            operator: CompareOp::Like,
            value: filter_value,
        }
    }

    pub fn gt<T>(column: &str, value: impl IntoFilterValue<T>) -> Self {
        let filter_value = value.into_filter_value();
        if !filter_value.is_single() {
            panic!("Greater than comparison requires a single value");
        }
        Self {
            column: column.to_string(),
            operator: CompareOp::Gt,
            value: filter_value,
        }
    }

    pub fn gte<T>(column: &str, value: impl IntoFilterValue<T>) -> Self {
        let filter_value = value.into_filter_value();
        if !filter_value.is_single() {
            panic!("Greater than or equal comparison requires a single value");
        }
        Self {
            column: column.to_string(),
            operator: CompareOp::Gte,
            value: filter_value,
        }
    }

    pub fn lt<T>(column: &str, value: impl IntoFilterValue<T>) -> Self {
        let filter_value = value.into_filter_value();
        if !filter_value.is_single() {
            panic!("Less than comparison requires a single value");
        }
        Self {
            column: column.to_string(),
            operator: CompareOp::Lt,
            value: filter_value,
        }
    }

    pub fn lte<T>(column: &str, value: impl IntoFilterValue<T>) -> Self {
        let filter_value = value.into_filter_value();
        if !filter_value.is_single() {
            panic!("Less than or equal comparison requires a single value");
        }
        Self {
            column: column.to_string(),
            operator: CompareOp::Lte,
            value: filter_value,
        }
    }
}

#[derive(Debug, Clone)]
pub struct QueryParam {
    pub columns: Vec<String>,
    pub query: String,
}

impl QueryParam {
    pub fn new(columns: Vec<String>, query: String) -> Self {
        Self { columns, query }
    }
}
