pub mod input_parser;
pub mod allocator_parser;
pub mod output_comparator;
pub mod coverage_parser;
pub mod complexity_parser;
pub mod scorer;
pub mod feedback;
pub mod report;
pub mod error;
pub mod traits;

// High-level API stubs
pub use input_parser::parse_inputs;
// pub use ... (other high-level APIs as needed) 