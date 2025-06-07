//! Main client implementation for the Replicate API.

use std::{env, time::Duration};
use crate::error::{Error, Result};
use crate::http::{HttpClient, HttpConfig, TimeoutConfig};
use crate::api::{PredictionsApi, FilesApi, predictions::PredictionBuilder};

/// Main client for interacting with the Replicate API.
#[derive(Debug, Clone)]
pub struct Client {
    http: HttpClient,
    predictions_api: PredictionsApi,
    files_api: FilesApi,
}

impl Client {
    /// Create a new client with the given API token.
    pub fn new(api_token: impl Into<String>) -> Result<Self> {
        let http = HttpClient::new(api_token)?;
        let predictions_api = PredictionsApi::new(http.clone());
        let files_api = FilesApi::new(http.clone());
        
        Ok(Self {
            http,
            predictions_api,
            files_api,
        })
    }
    
    /// Create a new client using the API token from the environment.
    /// 
    /// Looks for the token in the `REPLICATE_API_TOKEN` environment variable.
    pub fn from_env() -> Result<Self> {
        let api_token = env::var("REPLICATE_API_TOKEN")
            .map_err(|_| Error::auth_error("REPLICATE_API_TOKEN environment variable not found"))?;
        Self::new(api_token)
    }
    
    /// Create a new client with custom base URL.
    pub fn with_base_url(
        api_token: impl Into<String>,
        base_url: impl Into<String>,
    ) -> Result<Self> {
        let http = HttpClient::with_base_url(api_token, base_url)?;
        let predictions_api = PredictionsApi::new(http.clone());
        let files_api = FilesApi::new(http.clone());
        
        Ok(Self {
            http,
            predictions_api,
            files_api,
        })
    }
    
    /// Get access to the predictions API.
    pub fn predictions(&self) -> &PredictionsApi {
        &self.predictions_api
    }
    
    /// Get access to the files API.
    pub fn files(&self) -> &FilesApi {
        &self.files_api
    }
    
    /// Create a new prediction with a fluent builder API.
    /// 
    /// # Examples
    /// 
    /// ```no_run
    /// # use replicate_rs::Client;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("your-api-token")?;
    /// 
    /// let prediction = client
    ///     .create_prediction("stability-ai/sdxl:version-id")
    ///     .input("prompt", "A futuristic city skyline")
    ///     .input("width", 1024)
    ///     .input("height", 1024)
    ///     .send()
    ///     .await?;
    /// 
    /// println!("Prediction ID: {}", prediction.id);
    /// # Ok(())
    /// # }
    /// ```
    pub fn create_prediction(&self, version: impl Into<String>) -> PredictionBuilder {
        PredictionBuilder::new(self.predictions_api.clone(), version)
    }
    
    /// Run a model and wait for completion (convenience method).
    /// 
    /// This is equivalent to creating a prediction and waiting for it to complete.
    /// 
    /// # Examples
    /// 
    /// ```no_run
    /// # use replicate_rs::Client;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("your-api-token")?;
    /// 
    /// let result = client
    ///     .run("stability-ai/sdxl:version-id")
    ///     .input("prompt", "A futuristic city skyline")
    ///     .send_and_wait()
    ///     .await?;
    /// 
    /// println!("Result: {:?}", result.output);
    /// # Ok(())
    /// # }
    /// ```
    pub fn run(&self, version: impl Into<String>) -> PredictionBuilder {
        self.create_prediction(version)
    }
    
    /// Get the underlying HTTP client.
    pub fn http_client(&self) -> &HttpClient {
        &self.http
    }
    
    /// Get mutable access to the underlying HTTP client.
    /// 
    /// This allows configuring retry settings after client creation.
    pub fn http_client_mut(&mut self) -> &mut HttpClient {
        &mut self.http
    }
    
    /// Configure retry settings for this client.
    /// 
    /// This is a convenience method that delegates to the HTTP client.
    /// 
    /// # Examples
    /// 
    /// ```no_run
    /// # use replicate_rs::Client;
    /// # use std::time::Duration;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client = Client::new("your-api-token")?;
    /// 
    /// // Configure more aggressive retry settings
    /// client.configure_retries(
    ///     5,                               // max_retries
    ///     Duration::from_millis(100),      // min_delay
    ///     Duration::from_secs(60),         // max_delay
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn configure_retries(&mut self, max_retries: u32, min_delay: Duration, max_delay: Duration) -> Result<()> {
        self.http.configure_retries(max_retries, min_delay, max_delay)
    }
    
    /// Configure timeout settings for this client.
    /// 
    /// This is a convenience method that delegates to the HTTP client.
    /// 
    /// # Examples
    /// 
    /// ```no_run
    /// # use replicate_rs::Client;
    /// # use std::time::Duration;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client = Client::new("your-api-token")?;
    /// 
    /// // Configure custom timeouts
    /// client.configure_timeouts(
    ///     Some(Duration::from_secs(10)),   // connect_timeout
    ///     Some(Duration::from_secs(120)),  // request_timeout
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn configure_timeouts(&mut self, connect_timeout: Option<Duration>, request_timeout: Option<Duration>) -> Result<()> {
        self.http.configure_timeouts(connect_timeout, request_timeout)
    }
    
    /// Create a new client with custom HTTP configuration.
    pub fn with_http_config(api_token: impl Into<String>, http_config: HttpConfig) -> Result<Self> {
        let http = HttpClient::with_http_config(api_token, http_config)?;
        let predictions_api = PredictionsApi::new(http.clone());
        let files_api = FilesApi::new(http.clone());
        
        Ok(Self {
            http,
            predictions_api,
            files_api,
        })
    }
    
    /// Get the current timeout configuration.
    pub fn timeout_config(&self) -> &TimeoutConfig {
        self.http.timeout_config()
    }
    
    /// Get the current HTTP configuration.
    pub fn http_config(&self) -> &HttpConfig {
        self.http.http_config()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_client_creation() {
        let client = Client::new("test-token");
        assert!(client.is_ok());
    }
    
    #[test]
    fn test_client_empty_token() {
        let client = Client::new("");
        assert!(client.is_err());
        assert!(matches!(client.unwrap_err(), Error::Auth(_)));
    }
    
    #[test]
    fn test_client_from_env_missing() {
        // Save current value and remove it for test
        let original = env::var("REPLICATE_API_TOKEN").ok();
        unsafe {
            env::remove_var("REPLICATE_API_TOKEN");
        }
        
        let client = Client::from_env();
        assert!(client.is_err());
        assert!(matches!(client.unwrap_err(), Error::Auth(_)));
        
        // Restore original value if it existed
        if let Some(value) = original {
            unsafe {
                env::set_var("REPLICATE_API_TOKEN", value);
            }
        }
    }
    
    #[test] 
    fn test_client_from_env_present() {
        // Save current value
        let original = env::var("REPLICATE_API_TOKEN").ok();
        
        unsafe {
            env::set_var("REPLICATE_API_TOKEN", "test-token");
        }
        let client = Client::from_env();
        assert!(client.is_ok());
        
        // Restore original value or remove if it didn't exist
        unsafe {
            match original {
                Some(value) => env::set_var("REPLICATE_API_TOKEN", value),
                None => env::remove_var("REPLICATE_API_TOKEN"),
            }
        }
    }
} 