//! # replicate-rs
//!
//! A Rust client library for [Replicate](https://replicate.com), allowing you to run AI models,
//! create predictions, stream outputs, and manage various Replicate resources.
//!
//! ## Quick Start
//!
//! ```no_run
//! use replicate_rs::{Client, Error};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Error> {
//!     // Initialize the client with your API token
//!     let client = Client::new("your-api-token")?;
//!     
//!     // Run a simple prediction
//!     let prediction = client
//!         .create_prediction("stability-ai/sdxl:version-id")
//!         .input("prompt", "A futuristic city skyline")
//!         .send()
//!         .await?;
//!     
//!     println!("Prediction ID: {}", prediction.id);
//!     Ok(())
//! }
//! ```

pub mod api;
pub mod client;
pub mod error;
pub mod http;
pub mod models;

// Re-export main types for convenience
pub use api::files::{File, FilesApi};
pub use client::Client;
pub use error::{Error, Result};
pub use http::{HttpConfig, RetryConfig, TimeoutConfig};
pub use models::{
    file::{FileEncodingStrategy, FileInput, FileOutput},
    prediction::{Prediction, PredictionStatus},
};

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
