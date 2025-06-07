# replicate-rs

A Rust client library for [Replicate](https://replicate.com). This library allows you to run AI models, create predictions, stream outputs, and manage various Replicate resources from your Rust applications.

## Overview

`replicate-rs` is an async-first Rust implementation of the Replicate API client, designed to provide a safe, efficient, and ergonomic interface to Replicate's AI model platform. Built with `tokio` for async runtime and `reqwest` for HTTP client capabilities.

**Production-ready features include:** retry logic with exponential backoff, configurable timeouts, comprehensive error handling, and full async support.

## Features

### Core Functionality
- ✅ **Async/Await Support**: Built on tokio for high-performance async operations
- ✅ **Type Safety**: Leverages Rust's type system for compile-time correctness
- ✅ **HTTP Client**: Uses reqwest for robust HTTP communications
- ✅ **Authentication**: Secure API token management
- ✅ **Error Handling**: Comprehensive error types with detailed context

### API Operations
- ✅ **Predictions**: Create, get, list, and cancel predictions
- 🔲 **Models**: Access and manage AI models
- 🔲 **Streaming**: Real-time server-sent events for model outputs
- ✅ **Files**: Upload and manage files with multipart form data
- 🔲 **Versions**: Access specific model versions
- 🔲 **Collections**: Browse model collections
- 🔲 **Deployments**: Manage model deployments
- 🔲 **Training**: Create and manage fine-tuning jobs
- 🔲 **Webhooks**: Configure webhooks for async notifications
- 🔲 **Hardware**: Query available hardware options

### Advanced Features
- ✅ **Pagination**: Efficient handling of paginated responses
- ✅ **Retry Logic**: Automatic retry with exponential backoff
- ✅ **Timeout Configuration**: Configurable connect and request timeouts
- 🔲 **Rate Limiting**: Built-in rate limit handling
- ✅ **File Handling**: Multipart uploads, encoding strategies, metadata support
- 🔲 **Progress Tracking**: Monitor long-running predictions
- 🔲 **Concurrent Operations**: Run multiple predictions in parallel

## Quick Start

```toml
[dependencies]
replicate-rs = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
```

```rust
use replicate_client::{Client, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize the client with your API token
    let client = Client::new("your-api-token")?;
    
    // Run a simple prediction
    let output = client
        .run("stability-ai/sdxl")
        .input("prompt", "A futuristic city skyline")
        .await?;
    
    println!("Generated image: {}", output);
    Ok(())
}
```

## Configuration

### Retry Configuration

The client includes built-in retry logic with exponential backoff for handling transient failures:

```rust
use replicate_client::{Client, RetryConfig};
use std::time::Duration;

// Create client with custom retry settings
let retry_config = RetryConfig {
    max_retries: 5,
    min_delay: Duration::from_millis(100),
    max_delay: Duration::from_secs(30),
    base_multiplier: 2,
};

let client = Client::with_retry_config("your-api-token", retry_config)?;

// Or configure an existing client
let mut client = Client::new("your-api-token")?;
client.configure_retries(
    3,                               // max_retries
    Duration::from_millis(500),      // min_delay  
    Duration::from_secs(30),         // max_delay
)?;
```

### Timeout Configuration

Configure connection and request timeouts for better control over network operations:

```rust
use replicate_client::{Client, TimeoutConfig, HttpConfig, RetryConfig};
use std::time::Duration;

// Configure timeouts on existing client
let mut client = Client::new("your-api-token")?;
client.configure_timeouts(
    Some(Duration::from_secs(10)),   // connect_timeout
    Some(Duration::from_secs(120)),  // request_timeout
)?;

// Create client with custom timeout and retry configuration
let http_config = HttpConfig {
    retry: RetryConfig {
        max_retries: 3,
        min_delay: Duration::from_millis(500),
        max_delay: Duration::from_secs(30),
        base_multiplier: 2,
    },
    timeout: TimeoutConfig {
        connect_timeout: Some(Duration::from_secs(15)),
        request_timeout: Some(Duration::from_secs(90)),
    },
};

let client = Client::with_http_config("your-api-token", http_config)?;

// Disable timeouts (use with caution!)
client.configure_timeouts(None, None)?;
```

## File Uploads and Multipart Form Data

The library provides comprehensive file handling with multipart form data support for efficient file uploads:

### Basic File Upload

```rust
use replicate_client::{Client, FileInput, FileEncodingStrategy};
use std::collections::HashMap;

let client = Client::new("your-api-token")?;

// Upload file from bytes with metadata
let file_content = b"Hello, World!";
let mut metadata = HashMap::new();
metadata.insert("description".to_string(), serde_json::Value::String("Test file".to_string()));

let uploaded_file = client.files().create_from_bytes(
    file_content,
    Some("hello.txt"),
    Some("text/plain"),
    Some(&metadata),
).await?;

println!("Uploaded file ID: {}", uploaded_file.id);
println!("File URL: {}", uploaded_file.urls.get("get").unwrap());

// Upload from local file path
let file = client.files().create_from_path("./image.jpg", None).await?;

// Upload using FileInput abstraction
let file_input = FileInput::from_bytes_with_metadata(
    image_data,
    Some("image.jpg".to_string()),
    Some("image/jpeg".to_string()),
);
let file = client.files().create_from_file_input(&file_input, None).await?;
```

### File Management

```rust
// List all uploaded files
let files = client.files().list().await?;
for file in files {
    println!("File: {} ({})", file.name, file.id);
}

// Get file by ID
let file = client.files().get("file-id").await?;
println!("File size: {} bytes", file.size);

// Delete file
let deleted = client.files().delete("file-id").await?;
assert!(deleted);
```

### File Encoding Strategies

The library supports two file encoding strategies for use with predictions:

```rust
// 1. Multipart Upload (recommended for larger files)
let prediction = client
    .create_prediction("stability-ai/sdxl:version")
    .file_input_with_strategy("image", file_input, FileEncodingStrategy::Multipart)
    .input("prompt", "Enhance this image")
    .send()
    .await?;

// 2. Base64 Data URL (for smaller files < 1MB)
let prediction = client
    .create_prediction("stability-ai/sdxl:version")
    .file_input_with_strategy("image", file_input, FileEncodingStrategy::Base64DataUrl)
    .input("prompt", "Analyze this image")
    .send()
    .await?;

// Default strategy is Multipart
let prediction = client
    .create_prediction("stability-ai/sdxl:version")
    .file_input("image", file_input)  // Uses Multipart by default
    .send()
    .await?;
```

### File Input Types

```rust
// From URL (for reference, not upload)
let file_from_url = FileInput::from_url("https://example.com/image.jpg");

// From local file path
let file_from_path = FileInput::from_path("./local_image.jpg");

// From bytes with metadata
let file_from_bytes = FileInput::from_bytes_with_metadata(
    image_data,
    Some("image.jpg".to_string()),
    Some("image/jpeg".to_string()),
);

// Simple bytes input
let file_simple = FileInput::from_bytes(image_data);
```

### Advanced File Operations

```rust
// Upload with custom metadata
let mut metadata = HashMap::new();
metadata.insert("source".to_string(), serde_json::Value::String("user_upload".to_string()));
metadata.insert("category".to_string(), serde_json::Value::String("profile_image".to_string()));
metadata.insert("user_id".to_string(), serde_json::Value::Number(serde_json::Number::from(12345)));

let file = client.files().create_from_bytes(
    image_data,
    Some("profile.jpg"),
    Some("image/jpeg"),
    Some(&metadata),
).await?;

// Access file metadata
println!("File metadata: {:?}", file.metadata);
println!("File checksums: {:?}", file.checksums);
println!("Created at: {}", file.created_at);
```

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details on:

- Development setup
- Code style and formatting
- Testing requirements
- Pull request process
- Issue reporting

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [Replicate](https://replicate.com) for providing the AI model platform
- [replicate-python](https://github.com/replicate/replicate-python) for API design inspiration
- The Rust community for excellent async and HTTP libraries

## Status

🚧 **This project is under active development.** APIs may change before the 1.0 release.

Current status: **Phase 4 - Production Features (60% complete)**

**Recent milestones:**
- ✅ Phase 1: Core Infrastructure (100% complete)
- ✅ Phase 2: Core API Operations (85% complete)
- ✅ Phase 3: Advanced Features (80% complete)
- 🔄 Phase 4: Production Features - Retry logic ✅, Timeout configuration ✅
- ⏳ Phase 5: Polish & Release

For the latest updates, check our [project milestones](https://github.com/user/replicate-rs/milestones) and [issues](https://github.com/user/replicate-rs/issues). 