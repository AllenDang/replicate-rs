//! Error types for the Replicate client.

use thiserror::Error;

/// Result type alias for Replicate operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for the Replicate client.
#[derive(Error, Debug)]
pub enum Error {
    /// HTTP request failed
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// HTTP middleware error
    #[error("HTTP middleware error: {0}")]
    HttpMiddleware(#[from] reqwest_middleware::Error),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// API returned an error response
    #[error("API error: {status} - {message}")]
    Api {
        status: u16,
        message: String,
        detail: Option<String>,
    },

    /// Authentication error
    #[error("Authentication error: {0}")]
    Auth(String),

    /// Invalid input or configuration
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// File operation error
    #[error("File error: {0}")]
    File(#[from] std::io::Error),

    /// URL parsing error
    #[error("URL error: {0}")]
    Url(#[from] url::ParseError),

    /// Model execution error
    #[error("Model execution failed: {prediction_id}")]
    ModelExecution {
        prediction_id: String,
        error_message: Option<String>,
        logs: Option<String>,
    },

    /// Timeout error
    #[error("Operation timed out: {0}")]
    Timeout(String),

    /// Unsupported operation
    #[error("Unsupported operation: {0}")]
    Unsupported(String),
}

impl Error {
    /// Create a new API error
    pub fn api_error(status: u16, message: impl Into<String>) -> Self {
        Self::Api {
            status,
            message: message.into(),
            detail: None,
        }
    }

    /// Create a new API error with detail
    pub fn api_error_with_detail(
        status: u16,
        message: impl Into<String>,
        detail: impl Into<String>,
    ) -> Self {
        Self::Api {
            status,
            message: message.into(),
            detail: Some(detail.into()),
        }
    }

    /// Create an authentication error
    pub fn auth_error(message: impl Into<String>) -> Self {
        Self::Auth(message.into())
    }

    /// Create an invalid input error
    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::InvalidInput(message.into())
    }

    /// Create a model execution error
    pub fn model_execution(
        prediction_id: impl Into<String>,
        error_message: Option<String>,
        logs: Option<String>,
    ) -> Self {
        Self::ModelExecution {
            prediction_id: prediction_id.into(),
            error_message,
            logs,
        }
    }

    /// Create a timeout error
    pub fn timeout(message: impl Into<String>) -> Self {
        Self::Timeout(message.into())
    }

    /// Create an unsupported operation error
    pub fn unsupported(message: impl Into<String>) -> Self {
        Self::Unsupported(message.into())
    }
}

/// Helper trait for converting HTTP status codes to errors
pub trait StatusCodeExt {
    fn to_replicate_error(self, body: String) -> Error;
}

impl StatusCodeExt for reqwest::StatusCode {
    fn to_replicate_error(self, body: String) -> Error {
        match self.as_u16() {
            401 => Error::auth_error("Invalid API token"),
            402 => Error::auth_error("Insufficient credits"),
            403 => Error::auth_error("Forbidden"),
            404 => Error::api_error(404, "Resource not found"),
            422 => Error::api_error_with_detail(422, "Validation error", body),
            429 => Error::api_error(429, "Rate limit exceeded"),
            500..=599 => Error::api_error(self.as_u16(), "Server error"),
            _ => Error::api_error(self.as_u16(), body),
        }
    }
}
