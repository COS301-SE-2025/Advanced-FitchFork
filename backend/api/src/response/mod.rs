use serde::Serialize;

/// Standardized API response wrapper for all outgoing JSON responses.
///
/// This struct enforces a consistent response structure across all endpoints:
/// ```json
/// {
///   "success": true,
///   "data": { ... },
///   "message": "Some message"
/// }
/// ```
///
/// - `T` is the type of the `data` payload.
/// - `success` is a boolean indicating operation status.
/// - `message` provides a human-readable context string.
///
/// ## Example (success):
/// ```json
/// {
///   "success": true,
///   "data": { "id": 1, "name": "Alice" },
///   "message": "User fetched successfully"
/// }
/// ```
///
/// ## Example (error):
/// ```json
/// {
///   "success": false,
///   "data": {},
///   "message": "User not found"
/// }
/// ```
#[derive(Serialize)]
pub struct ApiResponse<T>
where
    T: Serialize,
{
    pub success: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,

    pub message: String,
}

impl<T> ApiResponse<T>
where
    T: Serialize,
{
    /// Constructs a success response with the given data and message.
    ///
    /// # Arguments
    /// - `data`: The result payload.
    /// - `message`: A descriptive message to accompany the success.
    pub fn success(data: T, message: impl Into<String>) -> Self {
        ApiResponse {
            success: true,
            data: Some(data),
            message: message.into(),
        }
    }

    /// Constructs an error response with a message and no `data`.
    ///
    /// # Arguments
    /// - `message`: A description of the error.
    pub fn error(message: impl Into<String>) -> Self {
        ApiResponse {
            success: false,
            data: None,
            message: message.into(),
        }
    }
}