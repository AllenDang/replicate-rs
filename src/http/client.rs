//! HTTP client implementation for the Replicate API with retry logic.

use crate::VERSION;
use crate::error::{Error, Result, StatusCodeExt};
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue, USER_AGENT};
use reqwest::{Method, Response};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
use retry_policies::Jitter;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;

/// Base URL for the Replicate API.
const DEFAULT_BASE_URL: &str = "https://api.replicate.com";

/// Configuration for retry behavior.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub min_delay: Duration,
    pub max_delay: Duration,
    pub base_multiplier: u32,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            min_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(30),
            base_multiplier: 2,
        }
    }
}

/// Configuration for HTTP timeouts.
#[derive(Debug, Clone)]
pub struct TimeoutConfig {
    pub connect_timeout: Option<Duration>,
    pub request_timeout: Option<Duration>,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            connect_timeout: Some(Duration::from_secs(30)),
            request_timeout: Some(Duration::from_secs(60)),
        }
    }
}

/// Combined HTTP client configuration.
#[derive(Debug, Clone, Default)]
pub struct HttpConfig {
    pub retry: RetryConfig,
    pub timeout: TimeoutConfig,
}

/// HTTP client for making requests to the Replicate API with retry logic.
#[derive(Debug, Clone)]
pub struct HttpClient {
    client: ClientWithMiddleware,
    base_url: String,
    api_token: String,
    http_config: HttpConfig,
}

impl HttpClient {
    /// Create a new HTTP client with the given API token and default retry logic.
    pub fn new(api_token: impl Into<String>) -> Result<Self> {
        Self::with_retry_config(api_token, RetryConfig::default())
    }

    /// Create a new HTTP client with the given API token and custom retry configuration.
    pub fn with_retry_config(
        api_token: impl Into<String>,
        retry_config: RetryConfig,
    ) -> Result<Self> {
        let http_config = HttpConfig {
            retry: retry_config,
            timeout: TimeoutConfig::default(),
        };
        Self::with_http_config(api_token, http_config)
    }

    /// Create a new HTTP client with the given API token and custom HTTP configuration.
    pub fn with_http_config(api_token: impl Into<String>, http_config: HttpConfig) -> Result<Self> {
        let api_token = api_token.into();
        if api_token.is_empty() {
            return Err(Error::auth_error("API token cannot be empty"));
        }

        let client = Self::build_client_with_config(&http_config)?;

        Ok(Self {
            client,
            base_url: DEFAULT_BASE_URL.to_string(),
            api_token,
            http_config,
        })
    }

    /// Build a reqwest client with retry middleware and timeout configuration.
    fn build_client_with_config(http_config: &HttpConfig) -> Result<ClientWithMiddleware> {
        // Create exponential backoff retry policy
        let retry_policy = ExponentialBackoff::builder()
            .retry_bounds(http_config.retry.min_delay, http_config.retry.max_delay)
            .jitter(Jitter::Bounded)
            .base(http_config.retry.base_multiplier)
            .build_with_max_retries(http_config.retry.max_retries);

        // Build reqwest client with timeout configuration
        let mut client_builder =
            reqwest::Client::builder().user_agent(format!("replicate-rs/{}", crate::VERSION));

        if let Some(connect_timeout) = http_config.timeout.connect_timeout {
            client_builder = client_builder.connect_timeout(connect_timeout);
        }

        if let Some(request_timeout) = http_config.timeout.request_timeout {
            client_builder = client_builder.timeout(request_timeout);
        }

        let reqwest_client = client_builder.build()?;

        // Build client with retry middleware
        let client = ClientBuilder::new(reqwest_client)
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();

        Ok(client)
    }

    /// Create a new HTTP client with custom base URL.
    pub fn with_base_url(
        api_token: impl Into<String>,
        base_url: impl Into<String>,
    ) -> Result<Self> {
        let mut client = Self::new(api_token)?;
        client.base_url = base_url.into();
        Ok(client)
    }

    /// Create a new HTTP client with custom base URL and retry configuration.
    pub fn with_base_url_and_retry(
        api_token: impl Into<String>,
        base_url: impl Into<String>,
        retry_config: RetryConfig,
    ) -> Result<Self> {
        let mut client = Self::with_retry_config(api_token, retry_config)?;
        client.base_url = base_url.into();
        Ok(client)
    }

    /// Create a new HTTP client with custom base URL and HTTP configuration.
    pub fn with_base_url_and_http_config(
        api_token: impl Into<String>,
        base_url: impl Into<String>,
        http_config: HttpConfig,
    ) -> Result<Self> {
        let mut client = Self::with_http_config(api_token, http_config)?;
        client.base_url = base_url.into();
        Ok(client)
    }

    /// Get a reference to the underlying client with middleware.
    pub fn inner(&self) -> &ClientWithMiddleware {
        &self.client
    }

    /// Build a full URL from a path.
    fn build_url(&self, path: &str) -> String {
        let path = path.strip_prefix('/').unwrap_or(path);
        format!("{}/{}", self.base_url.trim_end_matches('/'), path)
    }

    /// Execute a request and handle errors.
    async fn execute_request(&self, method: Method, path: &str) -> Result<Response> {
        let url = self.build_url(path);
        let response = self
            .client
            .request(method, &url)
            .header("Authorization", format!("Token {}", self.api_token))
            .header("Content-Type", "application/json")
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response)
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(status.to_replicate_error(body))
        }
    }

    /// Execute a request with JSON body and handle errors.
    async fn execute_request_with_json<T: Serialize>(
        &self,
        method: Method,
        path: &str,
        body: &T,
    ) -> Result<Response> {
        let url = self.build_url(path);
        let json_body = serde_json::to_vec(body)?;
        let response = self
            .client
            .request(method, &url)
            .header("Authorization", format!("Token {}", self.api_token))
            .header("Content-Type", "application/json")
            .body(json_body)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response)
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(status.to_replicate_error(body))
        }
    }

    /// Make a GET request.
    pub async fn get(&self, path: &str) -> Result<Response> {
        self.execute_request(Method::GET, path).await
    }

    /// Make a POST request with JSON body.
    pub async fn post<T: Serialize>(&self, path: &str, body: &T) -> Result<Response> {
        self.execute_request_with_json(Method::POST, path, body)
            .await
    }

    /// Make a POST request without a body.
    pub async fn post_empty(&self, path: &str) -> Result<Response> {
        self.execute_request(Method::POST, path).await
    }

    /// Make a PUT request with JSON body.
    pub async fn put<T: Serialize>(&self, path: &str, body: &T) -> Result<Response> {
        self.execute_request_with_json(Method::PUT, path, body)
            .await
    }

    /// Make a DELETE request.
    pub async fn delete(&self, path: &str) -> Result<Response> {
        self.execute_request(Method::DELETE, path).await
    }

    /// Make a GET request and deserialize the response as JSON.
    pub async fn get_json<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<T> {
        let response = self.get(path).await?;
        let json = response.json().await?;
        Ok(json)
    }

    /// Make a POST request and deserialize the response as JSON.
    pub async fn post_json<B: Serialize, T: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let response = self.post(path, body).await?;
        let json = response.json().await?;
        Ok(json)
    }

    /// Make a POST request without body and deserialize the response as JSON.
    pub async fn post_empty_json<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<T> {
        let response = self.post_empty(path).await?;
        let json = response.json().await?;
        Ok(json)
    }

    /// Configure retry policy for this client.
    ///
    /// This rebuilds the underlying HTTP client with new retry settings.
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
    /// client.http_client_mut().configure_retries(
    ///     5,                               // max_retries
    ///     Duration::from_millis(100),      // min_delay
    ///     Duration::from_secs(60),         // max_delay
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn configure_retries(
        &mut self,
        max_retries: u32,
        min_delay: Duration,
        max_delay: Duration,
    ) -> Result<()> {
        self.configure_retries_advanced(max_retries, min_delay, max_delay, 2)
    }

    /// Configure retry policy with advanced settings.
    ///
    /// This rebuilds the underlying HTTP client with new retry settings.
    ///
    /// # Arguments
    ///
    /// * `max_retries` - Maximum number of retry attempts
    /// * `min_delay` - Minimum delay between retries
    /// * `max_delay` - Maximum delay between retries
    /// * `base_multiplier` - Base multiplier for exponential backoff (typically 2)
    pub fn configure_retries_advanced(
        &mut self,
        max_retries: u32,
        min_delay: Duration,
        max_delay: Duration,
        base_multiplier: u32,
    ) -> Result<()> {
        let new_retry_config = RetryConfig {
            max_retries,
            min_delay,
            max_delay,
            base_multiplier,
        };

        let new_http_config = HttpConfig {
            retry: new_retry_config,
            timeout: self.http_config.timeout.clone(),
        };

        // Rebuild the client with new configuration
        let new_client = Self::build_client_with_config(&new_http_config)?;

        // Update the client and configuration
        self.client = new_client;
        self.http_config = new_http_config;

        Ok(())
    }

    /// Configure timeout settings for this client.
    ///
    /// This rebuilds the underlying HTTP client with new timeout settings.
    ///
    /// # Arguments
    ///
    /// * `connect_timeout` - Maximum time to wait for connection establishment (None = no timeout)
    /// * `request_timeout` - Maximum time to wait for complete request (None = no timeout)
    pub fn configure_timeouts(
        &mut self,
        connect_timeout: Option<Duration>,
        request_timeout: Option<Duration>,
    ) -> Result<()> {
        let new_timeout_config = TimeoutConfig {
            connect_timeout,
            request_timeout,
        };

        let new_http_config = HttpConfig {
            retry: self.http_config.retry.clone(),
            timeout: new_timeout_config,
        };

        // Rebuild the client with new configuration
        let new_client = Self::build_client_with_config(&new_http_config)?;

        // Update the client and configuration
        self.client = new_client;
        self.http_config = new_http_config;

        Ok(())
    }

    /// Get the current retry configuration.
    pub fn retry_config(&self) -> &RetryConfig {
        &self.http_config.retry
    }

    /// Get the current timeout configuration.
    pub fn timeout_config(&self) -> &TimeoutConfig {
        &self.http_config.timeout
    }

    /// Get the current HTTP configuration.
    pub fn http_config(&self) -> &HttpConfig {
        &self.http_config
    }

    /// Execute a multipart form request.
    async fn execute_multipart_request(
        &self,
        method: Method,
        path: &str,
        form: reqwest::multipart::Form,
    ) -> Result<Response> {
        let url = self.build_url(path);

        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Token {}", self.api_token))
                .map_err(|_| Error::auth_error("Invalid API token format"))?,
        );
        headers.insert(
            USER_AGENT,
            HeaderValue::from_str(&format!("replicate-rs/{}", VERSION))
                .map_err(|_| Error::InvalidInput("Invalid user agent format".to_string()))?,
        );

        // For multipart requests, we need to use the underlying reqwest client directly
        // since reqwest-middleware doesn't support multipart forms
        let inner_client = reqwest::Client::new();
        let request = inner_client
            .request(method, &url)
            .headers(headers)
            .multipart(form);

        let response = request.send().await?;

        if response.status().is_success() {
            Ok(response)
        } else {
            let status = response.status().as_u16();
            let text = response.text().await.unwrap_or_default();

            // Try to parse as JSON error
            if let Ok(api_error) = serde_json::from_str::<serde_json::Value>(&text) {
                let message = api_error
                    .get("detail")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown API error");

                Err(Error::Api {
                    status,
                    message: message.to_string(),
                    detail: Some(text),
                })
            } else {
                Err(Error::Api {
                    status,
                    message: text,
                    detail: None,
                })
            }
        }
    }

    /// POST request with multipart form data.
    pub async fn post_multipart(
        &self,
        path: &str,
        form: reqwest::multipart::Form,
    ) -> Result<Response> {
        self.execute_multipart_request(Method::POST, path, form)
            .await
    }

    /// POST multipart form data and parse JSON response.
    pub async fn post_multipart_json<T: for<'de> serde::Deserialize<'de>>(
        &self,
        path: &str,
        form: reqwest::multipart::Form,
    ) -> Result<T> {
        let response = self.post_multipart(path, form).await?;
        let text = response.text().await?;
        serde_json::from_str(&text).map_err(Into::into)
    }

    /// Create a multipart form from file and optional metadata.
    pub async fn create_file_form(
        file_content: &[u8],
        filename: Option<&str>,
        content_type: Option<&str>,
        metadata: Option<&std::collections::HashMap<String, serde_json::Value>>,
    ) -> Result<reqwest::multipart::Form> {
        let filename = filename.unwrap_or("file").to_string();
        let content_type = content_type
            .unwrap_or("application/octet-stream")
            .to_string();

        let file_part = reqwest::multipart::Part::bytes(file_content.to_vec())
            .file_name(filename)
            .mime_str(&content_type)
            .map_err(|e| Error::InvalidInput(format!("Invalid content type: {}", e)))?;

        let mut form = reqwest::multipart::Form::new().part("content", file_part);

        // Add metadata if provided
        if let Some(metadata) = metadata {
            let metadata_json = serde_json::to_string(metadata)?;
            form = form.text("metadata", metadata_json);
        }

        Ok(form)
    }

    /// Create a multipart form from a file path.
    pub async fn create_file_form_from_path(
        file_path: &Path,
        metadata: Option<&std::collections::HashMap<String, serde_json::Value>>,
    ) -> Result<reqwest::multipart::Form> {
        // Read file content
        let file_content = tokio::fs::read(file_path).await?;

        // Determine filename and content type
        let filename = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file");

        let content_type = mime_guess::from_path(file_path)
            .first_or_octet_stream()
            .to_string();

        Self::create_file_form(&file_content, Some(filename), Some(&content_type), metadata).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_url() {
        let client = HttpClient::new("test-token").unwrap();

        assert_eq!(
            client.build_url("/v1/predictions"),
            "https://api.replicate.com/v1/predictions"
        );

        assert_eq!(
            client.build_url("v1/predictions"),
            "https://api.replicate.com/v1/predictions"
        );
    }

    #[test]
    fn test_empty_token_error() {
        let result = HttpClient::new("");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::Auth(_)));
    }

    #[test]
    fn test_client_creation_with_retry() {
        let client = HttpClient::new("test-token");
        assert!(client.is_ok());

        // Verify the client has retry capabilities by checking it's using middleware
        let client = client.unwrap();
        let _inner = client.inner(); // Should be ClientWithMiddleware

        // Verify default retry configuration
        let retry_config = client.retry_config();
        assert_eq!(retry_config.max_retries, 3);
        assert_eq!(retry_config.min_delay, Duration::from_millis(500));
        assert_eq!(retry_config.max_delay, Duration::from_secs(30));
        assert_eq!(retry_config.base_multiplier, 2);
    }

    #[test]
    fn test_retry_configuration() {
        let mut client = HttpClient::new("test-token").unwrap();

        // Test initial configuration
        let initial_config = client.retry_config();
        assert_eq!(initial_config.max_retries, 3);

        // Test configuration update
        let result =
            client.configure_retries(5, Duration::from_millis(100), Duration::from_secs(60));
        assert!(result.is_ok());

        // Verify new configuration
        let new_config = client.retry_config();
        assert_eq!(new_config.max_retries, 5);
        assert_eq!(new_config.min_delay, Duration::from_millis(100));
        assert_eq!(new_config.max_delay, Duration::from_secs(60));
        assert_eq!(new_config.base_multiplier, 2);
    }

    #[test]
    fn test_custom_retry_config() {
        let custom_config = RetryConfig {
            max_retries: 2,
            min_delay: Duration::from_millis(200),
            max_delay: Duration::from_secs(10),
            base_multiplier: 3,
        };

        let client = HttpClient::with_retry_config("test-token", custom_config.clone());
        assert!(client.is_ok());

        let client = client.unwrap();
        let actual_config = client.retry_config();
        assert_eq!(actual_config.max_retries, custom_config.max_retries);
        assert_eq!(actual_config.min_delay, custom_config.min_delay);
        assert_eq!(actual_config.max_delay, custom_config.max_delay);
        assert_eq!(actual_config.base_multiplier, custom_config.base_multiplier);
    }

    #[test]
    fn test_timeout_configuration() {
        let timeout_config = TimeoutConfig {
            connect_timeout: Some(Duration::from_secs(15)),
            request_timeout: Some(Duration::from_secs(90)),
        };

        let http_config = HttpConfig {
            retry: RetryConfig::default(),
            timeout: timeout_config,
        };

        let client = HttpClient::with_http_config("test-token", http_config);
        assert!(client.is_ok());

        let client = client.unwrap();
        let returned_timeout_config = client.timeout_config();
        assert_eq!(
            returned_timeout_config.connect_timeout,
            Some(Duration::from_secs(15))
        );
        assert_eq!(
            returned_timeout_config.request_timeout,
            Some(Duration::from_secs(90))
        );
    }

    #[test]
    fn test_timeout_reconfiguration() {
        let mut client = HttpClient::new("test-token").unwrap();

        // Initial state should be default
        let initial_config = client.timeout_config();
        assert_eq!(
            initial_config.connect_timeout,
            Some(Duration::from_secs(30))
        );
        assert_eq!(
            initial_config.request_timeout,
            Some(Duration::from_secs(60))
        );

        // Configure new timeouts
        let result =
            client.configure_timeouts(Some(Duration::from_secs(5)), Some(Duration::from_secs(120)));
        assert!(result.is_ok());

        let updated_config = client.timeout_config();
        assert_eq!(updated_config.connect_timeout, Some(Duration::from_secs(5)));
        assert_eq!(
            updated_config.request_timeout,
            Some(Duration::from_secs(120))
        );
    }

    #[test]
    fn test_timeout_disable() {
        let mut client = HttpClient::new("test-token").unwrap();

        // Disable all timeouts
        let result = client.configure_timeouts(None, None);
        assert!(result.is_ok());

        let config = client.timeout_config();
        assert_eq!(config.connect_timeout, None);
        assert_eq!(config.request_timeout, None);
    }

    #[test]
    fn test_http_config_accessors() {
        let http_config = HttpConfig {
            retry: RetryConfig {
                max_retries: 2,
                min_delay: Duration::from_millis(100),
                max_delay: Duration::from_secs(20),
                base_multiplier: 4,
            },
            timeout: TimeoutConfig {
                connect_timeout: Some(Duration::from_secs(10)),
                request_timeout: Some(Duration::from_secs(45)),
            },
        };

        let client = HttpClient::with_http_config("test-token", http_config);
        assert!(client.is_ok());

        let client = client.unwrap();
        let returned_config = client.http_config();
        assert_eq!(returned_config.retry.max_retries, 2);
        assert_eq!(returned_config.retry.min_delay, Duration::from_millis(100));
        assert_eq!(returned_config.retry.max_delay, Duration::from_secs(20));
        assert_eq!(returned_config.retry.base_multiplier, 4);
        assert_eq!(
            returned_config.timeout.connect_timeout,
            Some(Duration::from_secs(10))
        );
        assert_eq!(
            returned_config.timeout.request_timeout,
            Some(Duration::from_secs(45))
        );
    }
}
