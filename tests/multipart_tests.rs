//! Integration tests for multipart file upload functionality.

use replicate_rs::{Client, Error, FileInput};
use std::collections::HashMap;
use tempfile::tempdir;

fn get_test_client() -> Option<Client> {
    std::env::var("REPLICATE_API_TOKEN")
        .ok()
        .and_then(|token| Client::new(token).ok())
}

/// Test file upload from bytes with metadata
#[tokio::test]
async fn test_file_upload_from_bytes() {
    let client = match get_test_client() {
        Some(client) => client,
        None => {
            println!("Skipping test - no valid API token");
            return;
        }
    };

    let file_content = b"Test file content for multipart upload";
    let mut metadata = HashMap::new();
    metadata.insert(
        "test".to_string(),
        serde_json::Value::String("multipart_test".to_string()),
    );
    metadata.insert(
        "source".to_string(),
        serde_json::Value::String("unit_test".to_string()),
    );

    let result = client
        .files()
        .create_from_bytes(
            file_content,
            Some("test_upload.txt"),
            Some("text/plain"),
            Some(&metadata),
        )
        .await;

    match result {
        Ok(file) => {
            assert_eq!(file.name, "test_upload.txt");
            assert_eq!(file.content_type, "text/plain");
            assert_eq!(file.size, file_content.len() as i64);
            assert!(!file.id.is_empty());
            assert!(!file.etag.is_empty());

            // Verify metadata
            assert_eq!(
                file.metadata.get("test").unwrap().as_str().unwrap(),
                "multipart_test"
            );
            assert_eq!(
                file.metadata.get("source").unwrap().as_str().unwrap(),
                "unit_test"
            );

            // Clean up
            let deleted = client.files().delete(&file.id).await.unwrap_or(false);
            assert!(deleted, "File should be deleted successfully");
        }
        Err(e) => {
            panic!("File upload failed: {}", e);
        }
    }
}

/// Test file upload from local path
#[tokio::test]
async fn test_file_upload_from_path() {
    let client = match get_test_client() {
        Some(client) => client,
        None => {
            println!("Skipping test - no valid API token");
            return;
        }
    };

    // Create temporary file
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("test_file.txt");
    let file_content = b"Content from file path upload test";
    tokio::fs::write(&file_path, file_content)
        .await
        .expect("Failed to write temp file");

    let result = client.files().create_from_path(&file_path, None).await;

    match result {
        Ok(file) => {
            assert_eq!(file.name, "test_file.txt");
            assert_eq!(file.content_type, "text/plain");
            assert_eq!(file.size, file_content.len() as i64);

            // Clean up
            let deleted = client.files().delete(&file.id).await.unwrap_or(false);
            assert!(deleted, "File should be deleted successfully");
        }
        Err(e) => {
            panic!("File upload from path failed: {}", e);
        }
    }

    temp_dir.close().expect("Failed to clean up temp dir");
}

/// Test file upload using FileInput abstraction
#[tokio::test]
async fn test_file_upload_via_file_input() {
    let client = match get_test_client() {
        Some(client) => client,
        None => {
            println!("Skipping test - no valid API token");
            return;
        }
    };

    let file_input = FileInput::from_bytes_with_metadata(
        &b"FileInput test content"[..],
        Some("fileinput_test.txt".to_string()),
        Some("text/plain".to_string()),
    );

    let result = client
        .files()
        .create_from_file_input(&file_input, None)
        .await;

    match result {
        Ok(file) => {
            assert_eq!(file.name, "fileinput_test.txt");
            assert_eq!(file.content_type, "text/plain");

            // Clean up
            let deleted = client.files().delete(&file.id).await.unwrap_or(false);
            assert!(deleted, "File should be deleted successfully");
        }
        Err(e) => {
            panic!("FileInput upload failed: {}", e);
        }
    }
}

/// Test error handling for invalid file operations
#[tokio::test]
async fn test_file_error_handling() {
    let client = match get_test_client() {
        Some(client) => client,
        None => {
            println!("Skipping test - no valid API token");
            return;
        }
    };

    // Test FileInput with URL (should fail for upload)
    let url_input = FileInput::from_url("https://example.com/test.jpg");
    let result = client
        .files()
        .create_from_file_input(&url_input, None)
        .await;
    assert!(result.is_err(), "Uploading from URL should fail");

    if let Err(Error::InvalidInput(msg)) = result {
        assert!(
            msg.contains("Cannot upload from URL"),
            "Error message should mention URL limitation"
        );
    } else {
        panic!("Expected InvalidInput error for URL upload");
    }
}
