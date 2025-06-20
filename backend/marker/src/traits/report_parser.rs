use crate::error::MarkerError;
use serde_json::Value;

pub trait ReportParser<T> {
    fn parse(&self, raw: &Value) -> Result<T, MarkerError>;
} 