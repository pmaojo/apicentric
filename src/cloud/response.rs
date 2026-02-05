use serde::Serialize;

/// A generic API response.
#[derive(Serialize)]
pub struct ApiResponse<T> {
    /// Whether the request was successful.
    pub success: bool,
    /// The data returned by the request.
    pub data: Option<T>,
    /// An error message if the request was not successful.
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    /// Creates a new successful `ApiResponse`.
    ///
    /// # Arguments
    ///
    /// * `data` - The data to include in the response.
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    /// Creates a new error `ApiResponse`.
    ///
    /// # Arguments
    ///
    /// * `message` - The error message.
    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
        }
    }
}
