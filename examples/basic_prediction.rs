//! Basic prediction example.
//!
//! This example demonstrates how to:
//! - Create a Replicate client
//! - Create and run a prediction
//! - Handle the results
//!
//! Run with: cargo run --example basic_prediction

use replicate_rs::Client;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging (optional)
    // tracing_subscriber::fmt::init();

    println!("🚀 Replicate Rust Client Demo");

    // Create client from environment variable
    let client = match Client::from_env() {
        Ok(client) => client,
        Err(_) => {
            println!("❌ Please set the REPLICATE_API_TOKEN environment variable");
            println!("   You can get your token from: https://replicate.com/account");
            return Ok(());
        }
    };

    // Example 1: Create a prediction (non-blocking)
    println!("\n📝 Creating a prediction...");

    let prediction = client
        .create_prediction("replicate/hello-world:5c7d5dc6dd8bf75c1acaa8565735e7986bc5b66206b55cca93cb72c9bf15ccaa")
        .input("text", "Hello from Rust!")
        .send()
        .await?;

    println!("✅ Prediction created with ID: {}", prediction.id);
    println!("   Status: {:?}", prediction.status);

    // Example 2: Wait for the prediction to complete
    println!("\n⏳ Waiting for prediction to complete...");

    let completed_prediction = client
        .predictions()
        .wait_for_completion(&prediction.id, Some(Duration::from_secs(60)), None)
        .await?;

    println!("✅ Prediction completed!");
    println!("   Status: {:?}", completed_prediction.status);
    println!("   Output: {:?}", completed_prediction.output);

    // Example 3: Run a model and wait (convenience method)
    println!("\n🔄 Running a model with convenience method...");

    let result = client
        .run("replicate/hello-world:5c7d5dc6dd8bf75c1acaa8565735e7986bc5b66206b55cca93cb72c9bf15ccaa")
        .input("text", "Hello from the convenience method!")
        .send_and_wait_with_timeout(Duration::from_secs(60))
        .await?;

    println!("✅ Model run completed!");
    println!("   Output: {:?}", result.output);

    // Example 4: List recent predictions
    println!("\n📋 Listing recent predictions...");

    let predictions_page = client.predictions().list(None).await?;
    println!("✅ Found {} predictions", predictions_page.results.len());

    for (i, pred) in predictions_page.results.iter().take(3).enumerate() {
        println!("   {}. {} - {:?}", i + 1, pred.id, pred.status);
    }

    if predictions_page.has_next() {
        println!("   (and more...)");
    }

    println!("\n🎉 Demo completed successfully!");

    Ok(())
}
