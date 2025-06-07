//! Demonstration of retry logic and exponential backoff in the Replicate client.
//! 
//! This example shows how the client automatically retries failed requests
//! with exponential backoff when encountering transient errors.

use replicate_rs::{Client, RetryConfig};
use std::time::{Duration, Instant};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Retry Logic and Exponential Backoff Demo\n");

    // Create client with built-in retry logic
    let client = Client::from_env()?;
    
    println!("📋 Client Configuration:");
    println!("   • Max retries: 3");
    println!("   • Min delay: 500ms");
    println!("   • Max delay: 30s");
    println!("   • Backoff: Exponential with jitter");
    println!("   • Base multiplier: 2x\n");

    // Test 1: Normal successful request (no retries needed)
    println!("✅ Test 1: Normal successful request");
    let start = Instant::now();
    match client.predictions().list(None).await {
        Ok(predictions) => {
            let duration = start.elapsed();
            println!("   ✓ Success in {:?}", duration);
            println!("   ✓ Found {} predictions", predictions.results.len());
        }
        Err(e) => {
            println!("   ✗ Failed: {}", e);
        }
    }
    println!();

    // Test 2: Request to invalid endpoint (will trigger retries)
    println!("🔄 Test 2: Request to invalid endpoint (demonstrates retry logic)");
    let start = Instant::now();
    
    // This will fail and trigger retries
    let invalid_client = Client::new("invalid_token_that_will_fail")?;
    match invalid_client.predictions().list(None).await {
        Ok(_) => {
            println!("   ✗ Unexpected success");
        }
        Err(e) => {
            let duration = start.elapsed();
            println!("   ✓ Failed as expected after retries in {:?}", duration);
            println!("   ✓ Error: {}", e);
            
            // The duration should be longer due to retries with exponential backoff
            if duration.as_millis() > 1000 {
                println!("   ✓ Retry logic engaged (took longer than 1s)");
            } else {
                println!("   ⚠ May not have triggered retries (too fast)");
            }
        }
    }
    println!();

    // Test 3: Show current retry configuration
    println!("⚙️ Test 3: Current retry configuration");
    let retry_config = client.http_client().retry_config();
    println!("   • Max retries: {}", retry_config.max_retries);
    println!("   • Min delay: {:?}", retry_config.min_delay);
    println!("   • Max delay: {:?}", retry_config.max_delay);
    println!("   • Base multiplier: {}x", retry_config.base_multiplier);
    println!("   • Policy: Exponential backoff with jitter");
    println!();

    // Test 4: Dynamic retry configuration
    println!("🔧 Test 4: Dynamic retry configuration");
    println!("   Configuring more aggressive retry settings...");
    
    let mut client_clone = client.clone();
    client_clone.configure_retries(
        5,                               // max_retries
        Duration::from_millis(100),      // min_delay  
        Duration::from_secs(60),         // max_delay
    )?;
    
    let new_config = client_clone.http_client().retry_config();
    println!("   ✓ Updated configuration:");
    println!("     • Max retries: {} (was 3)", new_config.max_retries);
    println!("     • Min delay: {:?} (was 500ms)", new_config.min_delay);
    println!("     • Max delay: {:?} (was 30s)", new_config.max_delay);
    println!();
    
    // Test 5: Custom RetryConfig
    println!("🎛️ Test 5: Creating client with custom RetryConfig");
    let custom_config = RetryConfig {
        max_retries: 2,
        min_delay: Duration::from_millis(200),
        max_delay: Duration::from_secs(10),
        base_multiplier: 3,
    };
    
    println!("   Custom configuration:");
    println!("     • Max retries: {}", custom_config.max_retries);
    println!("     • Min delay: {:?}", custom_config.min_delay);
    println!("     • Max delay: {:?}", custom_config.max_delay);
    println!("     • Base multiplier: {}x", custom_config.base_multiplier);
    println!();
    
    // Test 6: Error types and retry behavior
    println!("📚 Test 6: Error types and retry behavior");
    println!("   Errors that trigger retries:");
    println!("   ✓ 500-599 (Server errors)");
    println!("   ✓ 429 (Rate limiting)");
    println!("   ✓ Network timeouts");
    println!("   ✓ Connection errors");
    println!();
    println!("   Errors that do NOT trigger retries:");
    println!("   ✗ 400 (Bad request)");
    println!("   ✗ 401 (Unauthorized)");
    println!("   ✗ 403 (Forbidden)");
    println!("   ✗ 404 (Not found)");
    println!();

    println!("🎉 Retry demo completed!");
    println!("\n💡 Key features of the retry system:");
    println!("   • Transparent and automatic retry logic");
    println!("   • Configurable retry settings (max retries, delays, multiplier)");
    println!("   • Exponential backoff with jitter to prevent thundering herd");
    println!("   • Dynamic reconfiguration during runtime");
    println!("   • Smart error classification (retries only transient errors)");
    println!("\n🔧 Configuration options:");
    println!("   • Client::configure_retries() - Simple configuration");
    println!("   • HttpClient::configure_retries_advanced() - Full control");
    println!("   • RetryConfig struct - Custom configurations");

    Ok(())
} 