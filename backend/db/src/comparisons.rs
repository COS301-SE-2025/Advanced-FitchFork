// use sea_orm::{ColumnTrait, Condition, prelude::Expr};
// use chrono::{DateTime, Utc};
// use crate::models::assignment_file::FileType;
// use crate::models::assignment::{AssignmentType, Status as AssignmentStatus};
// use crate::models::plagiarism_case::Status as CaseStatus;
// use crate::models::tickets::TicketStatus;
// use crate::models::user_module_role::Role;

// #[derive(Debug, Clone, PartialEq)]
// pub enum CompareOp {
//     Eq,
//     Gt,
//     Gte,
//     Lt,
//     Lte,
//     Like,
//     NotEq,
// }

// #[derive(Debug, Clone)]
// pub struct Comparison<T> {
//     pub value: T,
//     pub op: CompareOp,
// }

// impl<T> Comparison<T> {
//     pub fn eq(value: T) -> Self {
//         Self { value, op: CompareOp::Eq }
//     }
    
//     pub fn gt(value: T) -> Self {
//         Self { value, op: CompareOp::Gt }
//     }
    
//     pub fn gte(value: T) -> Self {
//         Self { value, op: CompareOp::Gte }
//     }
    
//     pub fn lt(value: T) -> Self {
//         Self { value, op: CompareOp::Lt }
//     }
    
//     pub fn lte(value: T) -> Self {
//         Self { value, op: CompareOp::Lte }
//     }
    
//     pub fn like(value: T) -> Self {
//         Self { value, op: CompareOp::Like }
//     }
    
//     pub fn not_eq(value: T) -> Self {
//         Self { value, op: CompareOp::NotEq }
//     }
// }

// impl<T> From<T> for Comparison<T> {
//     fn from(value: T) -> Self {
//         Self::eq(value)
//     }
// }

// pub trait ApplyComparison<T> {
//     fn apply_comparison<C: ColumnTrait>(
//         condition: Condition, 
//         column: C, 
//         comparison: &Comparison<T>
//     ) -> Condition;
// }

// impl ApplyComparison<i64> for i64 {
//     fn apply_comparison<C: ColumnTrait>(
//         condition: Condition,
//         column: C,
//         comparison: &Comparison<i64>
//     ) -> Condition {
//         match comparison.op {
//             CompareOp::Eq => condition.add(column.eq(comparison.value)),
//             CompareOp::Gt => condition.add(column.gt(comparison.value)),
//             CompareOp::Gte => condition.add(column.gte(comparison.value)),
//             CompareOp::Lt => condition.add(column.lt(comparison.value)),
//             CompareOp::Lte => condition.add(column.lte(comparison.value)),
//             CompareOp::NotEq => condition.add(column.ne(comparison.value)),
//             CompareOp::Like => condition.add(column.like(&format!("%{}%", comparison.value))),
//         }
//     }
// }

// impl ApplyComparison<String> for String {
//     fn apply_comparison<C: ColumnTrait>(
//         condition: Condition,
//         column: C,
//         comparison: &Comparison<String>
//     ) -> Condition {
//         match comparison.op {
//             CompareOp::Eq => condition.add(column.eq(&comparison.value)),
//             CompareOp::NotEq => condition.add(column.ne(&comparison.value)),
//             CompareOp::Like => {
//                 let pattern = format!("%{}%", comparison.value.to_lowercase());
//                 condition.add(Expr::cust(&format!("LOWER({})", column.as_str())).like(&pattern))
//             },
//             CompareOp::Gt => condition.add(column.gt(&comparison.value)),
//             CompareOp::Gte => condition.add(column.gte(&comparison.value)),
//             CompareOp::Lt => condition.add(column.lt(&comparison.value)),
//             CompareOp::Lte => condition.add(column.lte(&comparison.value)),
//         }
//     }
// }

// impl ApplyComparison<DateTime<Utc>> for DateTime<Utc> {
//     fn apply_comparison<C: ColumnTrait>(
//         condition: Condition,
//         column: C,
//         comparison: &Comparison<DateTime<Utc>>
//     ) -> Condition {
//         match comparison.op {
//             CompareOp::Eq => condition.add(column.eq(comparison.value)),
//             CompareOp::Gt => condition.add(column.gt(comparison.value)),
//             CompareOp::Gte => condition.add(column.gte(comparison.value)),
//             CompareOp::Lt => condition.add(column.lt(comparison.value)),
//             CompareOp::Lte => condition.add(column.lte(comparison.value)),
//             CompareOp::NotEq => condition.add(column.ne(comparison.value)),
//             CompareOp::Like => condition, // Like doesn't make sense for dates
//         }
//     }
// }

// impl ApplyComparison<bool> for bool {
//     fn apply_comparison<C: ColumnTrait>(
//         condition: Condition,
//         column: C,
//         comparison: &Comparison<bool>
//     ) -> Condition {
//         match comparison.op {
//             CompareOp::Eq => condition.add(column.eq(comparison.value)),
//             CompareOp::NotEq => condition.add(column.ne(comparison.value)),
//             _ => condition, // Other operations don't make sense for bool
//         }
//     }
// }

// impl ApplyComparison<FileType> for FileType {
//     fn apply_comparison<C: ColumnTrait>(
//         condition: Condition,
//         column: C,
//         comparison: &Comparison<FileType>
//     ) -> Condition {
//         let value_str = comparison.value.to_string();
        
//         match comparison.op {
//             CompareOp::Eq => condition.add(column.eq(value_str)),
//             CompareOp::NotEq => condition.add(column.ne(value_str)),
//             CompareOp::Like => {
//                 let pattern = format!("%{}%", value_str.to_lowercase());
//                 condition.add(Expr::cust(&format!("LOWER({})", column.as_str())).like(&pattern))
//             },
//             _ => condition, // Other operations don't make sense for enum types
//         }
//     }
// }

// impl ApplyComparison<AssignmentType> for AssignmentType {
//     fn apply_comparison<C: ColumnTrait>(
//         condition: Condition,
//         column: C,
//         comparison: &Comparison<AssignmentType>
//     ) -> Condition {
//         // Convert enum to its string representation for database comparison
//         let value_str = comparison.value.to_string().to_lowercase(); // Uses strum's Display + lowercase
        
//         match comparison.op {
//             CompareOp::Eq => condition.add(column.eq(&value_str)),
//             CompareOp::NotEq => condition.add(column.ne(&value_str)),
//             CompareOp::Like => {
//                 // For enum LIKE operations, partial matching
//                 let pattern = format!("%{}%", value_str);
//                 condition.add(Expr::cust(&format!("LOWER({})", column.as_str())).like(&pattern))
//             },
//             _ => condition, // Other operations don't make sense for enum types
//         }
//     }
// }

// impl ApplyComparison<AssignmentStatus> for AssignmentStatus {
//     fn apply_comparison<C: ColumnTrait>(
//         condition: Condition,
//         column: C,
//         comparison: &Comparison<AssignmentStatus>
//     ) -> Condition {
//         let value_str = comparison.value.to_string().to_lowercase();
        
//         match comparison.op {
//             CompareOp::Eq => condition.add(column.eq(&value_str)),
//             CompareOp::NotEq => condition.add(column.ne(&value_str)),
//             CompareOp::Like => {
//                 let pattern = format!("%{}%", value_str);
//                 condition.add(Expr::cust(&format!("LOWER({})", column.as_str())).like(&pattern))
//             },
//             _ => condition, // Other operations don't make sense for enum types
//         }
//     }
// }

// impl ApplyComparison<CaseStatus> for CaseStatus {
//     fn apply_comparison<C: ColumnTrait>(
//         condition: Condition,
//         column: C,
//         comparison: &Comparison<CaseStatus>
//     ) -> Condition {
//         let value_str = comparison.value.to_string().to_lowercase();
        
//         match comparison.op {
//             CompareOp::Eq => condition.add(column.eq(&value_str)),
//             CompareOp::NotEq => condition.add(column.ne(&value_str)),
//             CompareOp::Like => {
//                 let pattern = format!("%{}%", value_str);
//                 condition.add(Expr::cust(&format!("LOWER({})", column.as_str())).like(&pattern))
//             },
//             _ => condition, // Other operations don't make sense for enum types
//         }
//     }
// }

// impl ApplyComparison<TicketStatus> for TicketStatus {
//     fn apply_comparison<C: ColumnTrait>(
//         condition: Condition,
//         column: C,
//         comparison: &Comparison<TicketStatus>
//     ) -> Condition {
//         let value_str = comparison.value.to_string().to_lowercase();
        
//         match comparison.op {
//             CompareOp::Eq => condition.add(column.eq(&value_str)),
//             CompareOp::NotEq => condition.add(column.ne(&value_str)),
//             CompareOp::Like => {
//                 let pattern = format!("%{}%", value_str);
//                 condition.add(Expr::cust(&format!("LOWER({})", column.as_str())).like(&pattern))
//             },
//             _ => condition, // Other operations don't make sense for enum types
//         }
//     }
// }

// impl ApplyComparison<Role> for Role {
//     fn apply_comparison<C: ColumnTrait>(
//         condition: Condition,
//         column: C,
//         comparison: &Comparison<Role>
//     ) -> Condition {
//         let value_str = comparison.value.to_string().to_lowercase();
        
//         match comparison.op {
//             CompareOp::Eq => condition.add(column.eq(&value_str)),
//             CompareOp::NotEq => condition.add(column.ne(&value_str)),
//             CompareOp::Like => {
//                 let pattern = format!("%{}%", value_str);
//                 condition.add(Expr::cust(&format!("LOWER({})", column.as_str())).like(&pattern))
//             },
//             _ => condition, // Other operations don't make sense for enum types
//         }
//     }
// }