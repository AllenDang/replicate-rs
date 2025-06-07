//! Prediction-related types and structures.

use crate::models::file::{FileEncodingStrategy, FileInput};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Status of a prediction.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PredictionStatus {
    /// The prediction is starting up
    Starting,
    /// The prediction is currently processing
    Processing,
    /// The prediction completed successfully
    Succeeded,
    /// The prediction failed
    Failed,
    /// The prediction was canceled
    Canceled,
}

impl PredictionStatus {
    /// Check if the prediction is in a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Succeeded | Self::Failed | Self::Canceled)
    }

    /// Check if the prediction is still running
    pub fn is_running(&self) -> bool {
        matches!(self, Self::Starting | Self::Processing)
    }
}

/// URLs associated with a prediction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionUrls {
    /// URL to fetch the prediction
    pub get: String,
    /// URL to cancel the prediction
    pub cancel: String,
    /// URL to stream the prediction output (if supported)
    pub stream: Option<String>,
}

/// A prediction made by a model hosted on Replicate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prediction {
    /// The unique ID of the prediction
    pub id: String,

    /// The model used to create the prediction (format: owner/name)
    pub model: String,

    /// The version ID of the model used
    pub version: String,

    /// The current status of the prediction
    pub status: PredictionStatus,

    /// The input parameters for the prediction
    pub input: Option<HashMap<String, Value>>,

    /// The output of the prediction (if completed)
    pub output: Option<Value>,

    /// Logs from the prediction execution
    pub logs: Option<String>,

    /// Error message if the prediction failed
    pub error: Option<String>,

    /// Metrics about the prediction performance
    pub metrics: Option<HashMap<String, Value>>,

    /// When the prediction was created
    pub created_at: Option<String>,

    /// When the prediction started processing
    pub started_at: Option<String>,

    /// When the prediction completed
    pub completed_at: Option<String>,

    /// URLs associated with the prediction
    pub urls: Option<PredictionUrls>,
}

impl Prediction {
    /// Check if the prediction is complete
    pub fn is_complete(&self) -> bool {
        self.status.is_terminal()
    }

    /// Check if the prediction succeeded
    pub fn is_successful(&self) -> bool {
        self.status == PredictionStatus::Succeeded
    }

    /// Check if the prediction failed
    pub fn is_failed(&self) -> bool {
        self.status == PredictionStatus::Failed
    }

    /// Check if the prediction was canceled
    pub fn is_canceled(&self) -> bool {
        self.status == PredictionStatus::Canceled
    }
}

/// Request to create a new prediction.
#[derive(Debug, Clone, Serialize)]
pub struct CreatePredictionRequest {
    /// The version ID of the model to run
    pub version: String,

    /// Input parameters for the model
    pub input: HashMap<String, Value>,

    /// Optional webhook URL for notifications
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,

    /// Optional webhook URL for completion notifications
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook_completed: Option<String>,

    /// Events to filter for webhooks
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook_events_filter: Option<Vec<String>>,

    /// Enable streaming of output
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    /// File inputs that need to be processed
    #[serde(skip)]
    pub file_inputs: HashMap<String, FileInput>,

    /// File encoding strategy
    #[serde(skip)]
    pub file_encoding_strategy: FileEncodingStrategy,
}

impl CreatePredictionRequest {
    /// Create a new prediction request
    pub fn new(version: impl Into<String>) -> Self {
        Self {
            version: version.into(),
            input: HashMap::new(),
            webhook: None,
            webhook_completed: None,
            webhook_events_filter: None,
            stream: None,
            file_inputs: HashMap::new(),
            file_encoding_strategy: FileEncodingStrategy::default(),
        }
    }

    /// Add an input parameter
    pub fn with_input(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        self.input.insert(key.into(), value.into());
        self
    }

    /// Set the webhook URL
    pub fn with_webhook(mut self, webhook: impl Into<String>) -> Self {
        self.webhook = Some(webhook.into());
        self
    }

    /// Enable streaming
    pub fn with_streaming(mut self) -> Self {
        self.stream = Some(true);
        self
    }
}
