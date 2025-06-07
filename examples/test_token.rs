//! Simple test to verify API token works.

use replicate_rs::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Testing API token...");
    
    let client = Client::from_env()?;
    println!("✅ Client created successfully");
    
    // Try to list predictions (this should work with any valid token)
    match client.predictions().list(None).await {
        Ok(predictions) => {
            println!("✅ API call successful!");
            println!("   Found {} predictions", predictions.results.len());
        }
        Err(e) => {
            println!("❌ API call failed: {}", e);
            return Err(e.into());
        }
    }
    
    println!("🎉 Token test completed successfully!");
    Ok(())
} 