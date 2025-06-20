#[derive(Debug)]
pub enum MarkerError {
    /// Input arrays do not match in length or structure
    InputMismatch(String),
    /// JSON is malformed or does not match expected schema
    InvalidJson(String),
    /// A required field is missing from input
    MissingField(String),
    /// I/O error (file not found, unreadable, etc.)
    IoError(String),
    /// Mark allocation weights do not match expected totals
    WeightMismatch(String),
    /// A required task ID is missing in coverage or complexity report
    MissingTaskId(String),
    /// Error parsing coverage report
    ParseCoverageError(String),
    /// Error parsing mark allocator
    ParseAllocatorError(String),
    /// Error parsing complexity report
    ParseComplexityError(String),
} 