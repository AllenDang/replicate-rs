//! Data models and types for the Replicate API.

pub mod common;
pub mod file;
pub mod prediction;

// Re-export commonly used types
pub use common::{ApiResponse, PaginatedResponse};
pub use file::{FileInput, FileOutput};
pub use prediction::{CreatePredictionRequest, Prediction, PredictionStatus};
