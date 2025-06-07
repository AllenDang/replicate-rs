use replicate_rs::{Client, Error};

#[tokio::test]
async fn test_api_token_validation() -> Result<(), Error> {
    // Test with valid token from environment
    if let Ok(client) = Client::from_env() {
        // Try to list predictions to verify authentication
        let predictions = client.predictions().list(None).await?;
        // Should succeed with valid token
        println!("✅ API authentication successful with {} predictions", predictions.results.len());
    } else {
        println!("⚠️ No REPLICATE_API_TOKEN environment variable set, skipping test");
    }
    
    Ok(())
}

#[tokio::test] 
async fn test_invalid_token_rejection() {
    // Test with invalid token
    let client = Client::new("invalid_token_123").unwrap();
    let result = client.predictions().list(None).await;
    
    // Should fail with authentication error
    assert!(result.is_err());
    if let Err(error) = result {
        println!("✅ Invalid token correctly rejected: {}", error);
    }
} 