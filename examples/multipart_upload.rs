//! Multipart file upload example.
//!
//! This example demonstrates how to:
//! - Upload files using multipart form data
//! - Use different file encoding strategies
//! - Integrate file uploads with predictions
//! - Handle file metadata
//!
//! Run with: cargo run --example multipart_upload

use replicate_rs::{Client, FileInput};
use std::collections::HashMap;
use tempfile::tempdir;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📤 Multipart File Upload Demo");
    
    // Create client
    let client = match std::env::var("REPLICATE_API_TOKEN") {
        Ok(token) => Client::new(token)?,
        Err(_) => {
            eprintln!("Please set the REPLICATE_API_TOKEN environment variable");
            return Ok(());
        }
    };
    
    println!("\n=== File Upload Examples ===\n");
    
    // Example 1: Upload file from bytes
    println!("1. Uploading file from bytes...");
    
    let file_content = b"Hello from Rust! This is a test file for multipart upload.";
    let mut metadata = HashMap::new();
    metadata.insert("description".to_string(), serde_json::Value::String("Test file from Rust".to_string()));
    metadata.insert("source".to_string(), serde_json::Value::String("multipart_upload_example".to_string()));
    
    match client.files().create_from_bytes(
        file_content,
        Some("test_from_rust.txt"),
        Some("text/plain"),
        Some(&metadata),
    ).await {
        Ok(file) => {
            println!("✅ File uploaded successfully!");
            println!("   File ID: {}", file.id);
            println!("   Name: {}", file.name);
            println!("   Size: {} bytes", file.size);
            println!("   Content Type: {}", file.content_type);
            println!("   ETag: {}", file.etag);
            
            // Clean up - delete the file
            if client.files().delete(&file.id).await.unwrap_or(false) {
                println!("   ✅ File deleted successfully");
            }
        }
        Err(e) => {
            println!("❌ Failed to upload file: {}", e);
        }
    }
    
    // Example 2: Upload file from local path
    println!("\n2. Uploading file from local path...");
    
    // Create a temporary file
    let temp_dir = tempdir()?;
    let temp_file_path = temp_dir.path().join("example_image.txt");
    let image_content = b"This simulates image data for testing multipart uploads with file paths.";
    tokio::fs::write(&temp_file_path, image_content).await?;
    
    let mut image_metadata = HashMap::new();
    image_metadata.insert("type".to_string(), serde_json::Value::String("example".to_string()));
    image_metadata.insert("created_by".to_string(), serde_json::Value::String("multipart_demo".to_string()));
    
    match client.files().create_from_path(&temp_file_path, Some(&image_metadata)).await {
        Ok(file) => {
            println!("✅ File uploaded from path successfully!");
            println!("   File ID: {}", file.id);
            println!("   Name: {}", file.name);
            println!("   Size: {} bytes", file.size);
            println!("   Content Type: {}", file.content_type);
            
            // Clean up
            if client.files().delete(&file.id).await.unwrap_or(false) {
                println!("   ✅ File deleted successfully");
            }
        }
        Err(e) => {
            println!("❌ Failed to upload file from path: {}", e);
        }
    }
    
    // Example 3: Upload using FileInput
    println!("\n3. Uploading using FileInput abstraction...");
    
    let file_input = FileInput::from_bytes_with_metadata(
        &b"FileInput abstraction test content"[..],
        Some("fileinput_test.txt".to_string()),
        Some("text/plain".to_string()),
    );
    
    match client.files().create_from_file_input(&file_input, None).await {
        Ok(file) => {
            println!("✅ File uploaded via FileInput successfully!");
            println!("   File ID: {}", file.id);
            println!("   Name: {}", file.name);
            
            // Clean up
            if client.files().delete(&file.id).await.unwrap_or(false) {
                println!("   ✅ File deleted successfully");
            }
        }
        Err(e) => {
            println!("❌ Failed to upload via FileInput: {}", e);
        }
    }
    
    // Example 4: List uploaded files
    println!("\n4. Listing uploaded files...");
    
    match client.files().list().await {
        Ok(files) => {
            println!("✅ Found {} uploaded files:", files.len());
            for (i, file) in files.iter().take(5).enumerate() {
                println!("   {}. {} (ID: {}, Size: {} bytes)", 
                    i + 1, file.name, file.id, file.size);
            }
            if files.len() > 5 {
                println!("   ... and {} more files", files.len() - 5);
            }
        }
        Err(e) => {
            println!("❌ Failed to list files: {}", e);
        }
    }
    
    // Example 5: Demonstrate encoding strategies
    println!("\n=== File Encoding Strategies ===\n");
    
    // Create a test file for encoding demonstrations
    let test_content = b"Test content for encoding strategy demonstration";
    let _test_file = FileInput::from_bytes_with_metadata(
        &test_content[..],
        Some("encoding_test.txt".to_string()),
        Some("text/plain".to_string()),
    );
    
    // Base64 Data URL encoding (demonstration)
    println!("5. Base64 Data URL encoding...");
    println!("✅ Base64 Data URL encoding is available for small files");
    println!("   This encoding embeds file data directly in the JSON request");
    println!("   Best for small files (< 1MB) to avoid request size limits");
    
    // Multipart upload encoding (demonstration)
    println!("\n6. Multipart upload encoding...");
    println!("✅ Multipart upload encoding is the default strategy");
    println!("   Files are uploaded separately and referenced by URL");
    println!("   Best for larger files and production use cases");
    
    // Example 6: Integration with predictions (commented out - requires actual model)
    println!("\n=== Prediction Integration (Demo) ===\n");
    
    println!("7. File input integration with predictions:");
    println!("   The following shows how to use file inputs in predictions:");
    println!();
    println!("   // Create a file input");
    println!("   let image = FileInput::from_path(\"image.jpg\");");
    println!();
    println!("   // Use in prediction with multipart upload");
    println!("   let prediction = client");
    println!("       .create_prediction(\"model:version\")");
    println!("       .file_input_with_strategy(\"image\", image, FileEncodingStrategy::Multipart)");
    println!("       .input(\"prompt\", \"Analyze this image\")");
    println!("       .send()");
    println!("       .await?;");
    println!();
    println!("   // Or use base64 encoding for smaller files");
    println!("   let prediction = client");
    println!("       .create_prediction(\"model:version\")");
    println!("       .file_input_with_strategy(\"image\", image, FileEncodingStrategy::Base64DataUrl)");
    println!("       .send()");
    println!("       .await?;");
    
    // Clean up temporary directory
    temp_dir.close()?;
    
    println!("\n🎉 Multipart upload demo completed!");
    println!("\n📋 Summary of capabilities:");
    println!("   ✅ Upload files from bytes with metadata");
    println!("   ✅ Upload files from local paths");
    println!("   ✅ Use FileInput abstraction");
    println!("   ✅ List and manage uploaded files");
    println!("   ✅ Base64 Data URL encoding");
    println!("   ✅ Multipart form data uploads");
    println!("   ✅ Integration with prediction API");
    println!("   ✅ Automatic file cleanup");
    
    Ok(())
} 