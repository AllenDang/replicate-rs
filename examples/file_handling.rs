//! File handling example.
//!
//! This example demonstrates how to:
//! - Upload files to models
//! - Use different file input types (URL, path, bytes)
//! - Download file outputs
//!
//! Run with: cargo run --example file_handling

use replicate_client::{Client, FileInput, FileOutput};
use tempfile::tempdir;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🗂️ File Handling Demo");

    // Create client (for demonstration - not actually used in this example)
    let _client = match Client::from_env() {
        Ok(client) => client,
        Err(_) => {
            println!("❌ Please set the REPLICATE_API_TOKEN environment variable");
            return Ok(());
        }
    };

    // Example 1: File input from URL
    println!("\n🌐 Using file input from URL...");

    let file_url = "https://replicate.delivery/pbxt/IJjJHBfbGfNT4gmhLkmHvV6XDKQDV3LJXhXH4C0WMPBozWqTwg1/view.jpeg";
    let file_input = FileInput::from_url(file_url);

    println!("✅ Created file input from URL: {}", file_url);
    println!("   Type: URL = {}", file_input.is_url());

    // Example 2: File input from local path
    println!("\n📁 Using file input from local path...");

    // Create a temporary file for demonstration
    let temp_dir = tempdir()?;
    let temp_file_path = temp_dir.path().join("test_image.txt");
    tokio::fs::write(&temp_file_path, b"This is a test file").await?;

    let file_input = FileInput::from_path(&temp_file_path);
    println!("✅ Created file input from path: {:?}", temp_file_path);
    println!("   Type: Path = {}", file_input.is_path());

    // Example 3: File input from bytes
    println!("\n💾 Using file input from bytes...");

    let file_data = b"Binary file data here";
    let file_input = FileInput::from_bytes_with_metadata(
        file_data.as_slice(),
        Some("test.bin".to_string()),
        Some("application/octet-stream".to_string()),
    );

    println!("✅ Created file input from bytes");
    println!("   Type: Bytes = {}", file_input.is_bytes());

    // Example 4: Working with file outputs
    println!("\n📤 Working with file outputs...");

    let file_output = FileOutput::new("https://example.com/output.png")
        .with_filename("generated_image.png")
        .with_content_type("image/png")
        .with_size(1024 * 1024); // 1MB

    println!("✅ Created file output");
    println!("   URL: {}", file_output.url);
    println!("   Filename: {:?}", file_output.filename);
    println!("   Content Type: {:?}", file_output.content_type);
    println!("   Size: {:?} bytes", file_output.size);

    // Example 5: Using file inputs in predictions (commented out - requires actual model)
    /*
    println!("\n🤖 Using file input in a prediction...");

    let prediction = client
        .create_prediction("some-model-that-takes-images:version-id")
        .file_input("image", FileInput::from_url(file_url))
        .input("prompt", "Analyze this image")
        .send()
        .await?;

    println!("✅ Prediction created with file input: {}", prediction.id);
    */

    // Example 6: File download simulation
    println!("\n⬇️ File download capabilities...");

    // For demo purposes, we'll just show the API
    println!("   File output can be downloaded with:");
    println!("   - output.download().await  // Gets bytes");
    println!("   - output.save_to_path(path).await  // Saves to file");

    // Clean up
    temp_dir.close()?;

    println!("\n🎉 File handling demo completed!");

    Ok(())
}
