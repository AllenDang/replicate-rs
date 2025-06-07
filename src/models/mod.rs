//! Data models and types for the Replicate API.

pub mod prediction;
pub mod file;
pub mod common;

// Re-export commonly used types
pub use prediction::{Prediction, PredictionStatus, CreatePredictionRequest};
pub use file::{FileInput, FileOutput};
pub use common::{ApiResponse, PaginatedResponse}; 