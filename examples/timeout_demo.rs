use replicate_rs::{Client, HttpConfig, RetryConfig, TimeoutConfig};
use std::time::{Duration, Instant};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client with default configuration
    let api_token = match std::env::var("REPLICATE_API_TOKEN") {
        Ok(token) => token,
        Err(_) => {
            eprintln!("Please set the REPLICATE_API_TOKEN environment variable");
            return Ok(());
        }
    };
    let mut client = Client::new(&api_token)?;

    println!("=== Timeout Configuration Demo ===\n");

    // Display default timeout configuration
    let default_timeout_config = client.timeout_config();
    println!("Default timeout configuration:");
    println!(
        "  Connect timeout: {:?}",
        default_timeout_config.connect_timeout
    );
    println!(
        "  Request timeout: {:?}",
        default_timeout_config.request_timeout
    );
    println!();

    // Test with default timeouts
    println!("Testing with default timeouts...");
    let start = Instant::now();
    match client.predictions().list(None).await {
        Ok(predictions) => {
            println!("✓ Request succeeded in {:?}", start.elapsed());
            println!("  Found {} predictions", predictions.results.len());
        }
        Err(e) => {
            println!("✗ Request failed in {:?}: {}", start.elapsed(), e);
        }
    }
    println!();

    // Configure aggressive timeouts (short)
    println!("Configuring aggressive timeouts (10s connect, 30s request)...");
    client.configure_timeouts(
        Some(Duration::from_secs(10)), // 10s connect timeout
        Some(Duration::from_secs(30)), // 30s request timeout
    )?;

    let updated_config = client.timeout_config();
    println!("Updated timeout configuration:");
    println!("  Connect timeout: {:?}", updated_config.connect_timeout);
    println!("  Request timeout: {:?}", updated_config.request_timeout);
    println!();

    // Test with aggressive timeouts
    println!("Testing with aggressive timeouts...");
    let start = Instant::now();
    match client.predictions().list(None).await {
        Ok(predictions) => {
            println!("✓ Request succeeded in {:?}", start.elapsed());
            println!("  Found {} predictions", predictions.results.len());
        }
        Err(e) => {
            println!("✗ Request failed in {:?}: {}", start.elapsed(), e);
        }
    }
    println!();

    // Configure very long timeouts
    println!("Configuring very long timeouts (1m connect, 5m request)...");
    client.configure_timeouts(
        Some(Duration::from_secs(60)),  // 1 minute connect timeout
        Some(Duration::from_secs(300)), // 5 minutes request timeout
    )?;

    let long_config = client.timeout_config();
    println!("Long timeout configuration:");
    println!("  Connect timeout: {:?}", long_config.connect_timeout);
    println!("  Request timeout: {:?}", long_config.request_timeout);
    println!();

    // Demonstrate creating a client with custom configuration from the start
    println!("=== Creating Client with Custom Configuration ===\n");

    let custom_timeout_config = TimeoutConfig {
        connect_timeout: Some(Duration::from_secs(15)),
        request_timeout: Some(Duration::from_secs(90)),
    };

    let custom_retry_config = RetryConfig {
        max_retries: 2,
        min_delay: Duration::from_millis(200),
        max_delay: Duration::from_secs(10),
        base_multiplier: 3,
    };

    let custom_http_config = HttpConfig {
        retry: custom_retry_config,
        timeout: custom_timeout_config,
    };

    let custom_client = Client::with_http_config(&api_token, custom_http_config)?;

    println!("Custom client configuration:");
    let config = custom_client.http_config();
    println!("  Retry config:");
    println!("    Max retries: {}", config.retry.max_retries);
    println!("    Min delay: {:?}", config.retry.min_delay);
    println!("    Max delay: {:?}", config.retry.max_delay);
    println!("    Base multiplier: {}", config.retry.base_multiplier);
    println!("  Timeout config:");
    println!("    Connect timeout: {:?}", config.timeout.connect_timeout);
    println!("    Request timeout: {:?}", config.timeout.request_timeout);
    println!();

    // Test the custom client
    println!("Testing custom client...");
    let start = Instant::now();
    match custom_client.predictions().list(None).await {
        Ok(predictions) => {
            println!("✓ Request succeeded in {:?}", start.elapsed());
            println!("  Found {} predictions", predictions.results.len());
        }
        Err(e) => {
            println!("✗ Request failed in {:?}: {}", start.elapsed(), e);
        }
    }
    println!();

    // Demonstrate disabling timeouts (None values)
    println!("=== Disabling Timeouts ===\n");

    let mut no_timeout_client = Client::new(&api_token)?;

    println!("Disabling all timeouts (None values)...");
    no_timeout_client.configure_timeouts(None, None)?;

    let no_timeout_config = no_timeout_client.timeout_config();
    println!("No timeout configuration:");
    println!("  Connect timeout: {:?}", no_timeout_config.connect_timeout);
    println!("  Request timeout: {:?}", no_timeout_config.request_timeout);
    println!();

    // Note: Be careful with disabled timeouts in production!
    println!("⚠️  Note: Disabled timeouts can cause requests to hang indefinitely!");
    println!("   Use with caution in production environments.");

    Ok(())
}
