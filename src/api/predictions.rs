//! Predictions API implementation.

use std::collections::HashMap;
use std::time::Duration;
use serde_json::Value;
use tokio::time::{interval, timeout};

use crate::error::{Error, Result};
use crate::http::HttpClient;
use crate::models::{
    prediction::{Prediction, CreatePredictionRequest},
    common::PaginatedResponse,
    file::{FileInput, FileEncodingStrategy},
};
use crate::api::files::{FilesApi, process_file_input};

/// API for managing predictions.
#[derive(Debug, Clone)]
pub struct PredictionsApi {
    http: HttpClient,
    files_api: Option<FilesApi>,
}

impl PredictionsApi {
    /// Create a new predictions API instance.
    pub fn new(http: HttpClient) -> Self {
        Self { 
            http: http.clone(),
            files_api: Some(FilesApi::new(http)),
        }
    }
    
    /// Create a new prediction.
    pub async fn create(&self, mut request: CreatePredictionRequest) -> Result<Prediction> {
        // Process file inputs if any
        if !request.file_inputs.is_empty() {
            for (key, file_input) in request.file_inputs.iter() {
                let processed_value = process_file_input(
                    file_input,
                    &request.file_encoding_strategy,
                    self.files_api.as_ref(),
                ).await?;
                
                request.input.insert(key.clone(), serde_json::Value::String(processed_value));
            }
        }
        
        let prediction: Prediction = self.http
            .post_json("/v1/predictions", &request)
            .await?;
        Ok(prediction)
    }
    
    /// Get a prediction by ID.
    pub async fn get(&self, id: &str) -> Result<Prediction> {
        let path = format!("/v1/predictions/{}", id);
        let prediction: Prediction = self.http.get_json(&path).await?;
        Ok(prediction)
    }
    
    /// List predictions with optional pagination.
    pub async fn list(&self, cursor: Option<&str>) -> Result<PaginatedResponse<Prediction>> {
        let path = match cursor {
            Some(cursor) => cursor.to_string(),
            None => "/v1/predictions".to_string(),
        };
        
        let response: PaginatedResponse<Prediction> = self.http.get_json(&path).await?;
        Ok(response)
    }
    
    /// Cancel a prediction.
    pub async fn cancel(&self, id: &str) -> Result<Prediction> {
        let path = format!("/v1/predictions/{}/cancel", id);
        let prediction: Prediction = self.http.post_empty_json(&path).await?;
        Ok(prediction)
    }
    
    /// Wait for a prediction to complete with polling.
    pub async fn wait_for_completion(
        &self,
        id: &str,
        max_duration: Option<Duration>,
        poll_interval: Option<Duration>,
    ) -> Result<Prediction> {
        let poll_interval = poll_interval.unwrap_or(Duration::from_millis(500));
        let mut interval = interval(poll_interval);
        
        let wait_future = async {
            loop {
                interval.tick().await;
                let prediction = self.get(id).await?;
                
                if prediction.status.is_terminal() {
                    if prediction.is_failed() {
                        return Err(Error::model_execution(
                            id,
                            prediction.error.clone(),
                            prediction.logs.clone(),
                        ));
                    }
                    return Ok(prediction);
                }
            }
        };
        
        match max_duration {
            Some(duration) => timeout(duration, wait_future).await
                .map_err(|_| Error::Timeout(format!("Prediction {} did not complete within {:?}", id, duration)))?,
            None => wait_future.await,
        }
    }
}

/// Builder for creating predictions with a fluent API.
#[derive(Debug)]
pub struct PredictionBuilder {
    api: PredictionsApi,
    request: CreatePredictionRequest,
}

impl PredictionBuilder {
    /// Create a new prediction builder.
    pub fn new(api: PredictionsApi, version: impl Into<String>) -> Self {
        Self {
            api,
            request: CreatePredictionRequest::new(version),
        }
    }
    
    /// Add an input parameter.
    pub fn input<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<Value>,
    {
        self.request = self.request.with_input(key, value);
        self
    }
    
    /// Add multiple input parameters from a HashMap.
    pub fn inputs(mut self, inputs: HashMap<String, Value>) -> Self {
        for (key, value) in inputs {
            self.request = self.request.with_input(key, value);
        }
        self
    }
    
    /// Add a file input parameter.
    pub fn file_input<K>(mut self, key: K, file: FileInput) -> Self
    where
        K: Into<String>,
    {
        // Store the file input for later processing
        self.request.file_inputs.insert(key.into(), file);
        self
    }
    
    /// Add a file input with specific encoding strategy.
    pub fn file_input_with_strategy<K>(
        mut self, 
        key: K, 
        file: FileInput,
        strategy: FileEncodingStrategy,
    ) -> Self
    where
        K: Into<String>,
    {
        // Store the file input and strategy for later processing
        self.request.file_inputs.insert(key.into(), file);
        self.request.file_encoding_strategy = strategy;
        self
    }
    
    /// Set a webhook URL.
    pub fn webhook(mut self, webhook: impl Into<String>) -> Self {
        self.request = self.request.with_webhook(webhook);
        self
    }
    
    /// Enable streaming output.
    pub fn stream(mut self) -> Self {
        self.request = self.request.with_streaming();
        self
    }
    
    /// Send the prediction request.
    pub async fn send(self) -> Result<Prediction> {
        self.api.create(self.request).await
    }
    
    /// Send the prediction request and wait for completion.
    pub async fn send_and_wait(self) -> Result<Prediction> {
        let prediction = self.api.create(self.request).await?;
        self.api
            .wait_for_completion(&prediction.id, None, None)
            .await
    }
    
    /// Send the prediction request and wait for completion with custom timeout.
    pub async fn send_and_wait_with_timeout(
        self,
        max_duration: Duration,
    ) -> Result<Prediction> {
        let prediction = self.api.create(self.request).await?;
        self.api
            .wait_for_completion(&prediction.id, Some(max_duration), None)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http::HttpClient;
    
    fn create_test_api() -> PredictionsApi {
        let http = HttpClient::new("test-token").unwrap();
        PredictionsApi::new(http)
    }
    
    #[test]
    fn test_prediction_builder() {
        let api = create_test_api();
        let builder = PredictionBuilder::new(api, "test-version")
            .input("prompt", "test prompt")
            .webhook("https://example.com/webhook")
            .stream();
        
        assert_eq!(builder.request.version, "test-version");
        assert_eq!(
            builder.request.input.get("prompt"),
            Some(&Value::String("test prompt".to_string()))
        );
        assert_eq!(
            builder.request.webhook,
            Some("https://example.com/webhook".to_string())
        );
        assert_eq!(builder.request.stream, Some(true));
    }
} 