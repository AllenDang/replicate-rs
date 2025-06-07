//! Files API for uploading and managing files.

use crate::error::{Error, Result};
use crate::http::HttpClient;
use crate::models::file::{FileEncodingStrategy, FileInput};
use base64::{Engine as _, engine::general_purpose};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Represents a file uploaded to Replicate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct File {
    /// The unique ID of the file.
    pub id: String,
    /// The name of the file.
    pub name: String,
    /// The content type of the file.
    pub content_type: String,
    /// The size of the file in bytes.
    pub size: i64,
    /// The ETag of the file.
    pub etag: String,
    /// File checksums.
    pub checksums: HashMap<String, String>,
    /// File metadata.
    pub metadata: HashMap<String, serde_json::Value>,
    /// When the file was created.
    pub created_at: String,
    /// When the file expires (optional).
    pub expires_at: Option<String>,
    /// File URLs.
    pub urls: HashMap<String, String>,
}

/// Files API for managing file uploads.
#[derive(Debug, Clone)]
pub struct FilesApi {
    http: HttpClient,
}

impl FilesApi {
    /// Create a new Files API instance.
    pub fn new(http: HttpClient) -> Self {
        Self { http }
    }

    /// Upload a file from bytes with optional metadata.
    pub async fn create_from_bytes(
        &self,
        file_content: &[u8],
        filename: Option<&str>,
        content_type: Option<&str>,
        metadata: Option<&HashMap<String, serde_json::Value>>,
    ) -> Result<File> {
        let form =
            HttpClient::create_file_form(file_content, filename, content_type, metadata).await?;

        self.http.post_multipart_json("/v1/files", form).await
    }

    /// Upload a file from a local path.
    pub async fn create_from_path(
        &self,
        file_path: &Path,
        metadata: Option<&HashMap<String, serde_json::Value>>,
    ) -> Result<File> {
        let form = HttpClient::create_file_form_from_path(file_path, metadata).await?;
        self.http.post_multipart_json("/v1/files", form).await
    }

    /// Upload a file from FileInput.
    pub async fn create_from_file_input(
        &self,
        file_input: &FileInput,
        metadata: Option<&HashMap<String, serde_json::Value>>,
    ) -> Result<File> {
        match file_input {
            FileInput::Path(path) => self.create_from_path(path, metadata).await,
            FileInput::Bytes {
                data,
                filename,
                content_type,
            } => {
                self.create_from_bytes(data, filename.as_deref(), content_type.as_deref(), metadata)
                    .await
            }
            FileInput::Url(_) => Err(Error::InvalidInput(
                "Cannot upload from URL - file must be local or bytes".to_string(),
            )),
        }
    }

    /// Get a file by ID.
    pub async fn get(&self, file_id: &str) -> Result<File> {
        self.http.get_json(&format!("/v1/files/{}", file_id)).await
    }

    /// List all uploaded files.
    pub async fn list(&self) -> Result<Vec<File>> {
        #[derive(Deserialize)]
        struct ListResponse {
            results: Vec<File>,
        }

        let response: ListResponse = self.http.get_json("/v1/files").await?;
        Ok(response.results)
    }

    /// Delete a file by ID.
    pub async fn delete(&self, file_id: &str) -> Result<bool> {
        let response = self.http.delete(&format!("/v1/files/{}", file_id)).await?;
        Ok(response.status() == 204)
    }
}

/// Helper to process file inputs based on encoding strategy.
pub async fn process_file_input(
    file_input: &FileInput,
    encoding_strategy: &FileEncodingStrategy,
    files_api: Option<&FilesApi>,
) -> Result<String> {
    match encoding_strategy {
        FileEncodingStrategy::Base64DataUrl => encode_file_as_data_url(file_input).await,
        FileEncodingStrategy::Multipart => {
            if let Some(api) = files_api {
                let file = api.create_from_file_input(file_input, None).await?;
                // Return the file URL for use in predictions
                file.urls
                    .get("get")
                    .cloned()
                    .ok_or_else(|| Error::InvalidInput("File missing URL".to_string()))
            } else {
                Err(Error::InvalidInput(
                    "Files API required for multipart upload".to_string(),
                ))
            }
        }
    }
}

/// Encode a file input as a base64 data URL.
async fn encode_file_as_data_url(file_input: &FileInput) -> Result<String> {
    match file_input {
        FileInput::Url(_url) => {
            // For URLs, we can't encode as data URL without downloading
            Err(Error::InvalidInput(
                "Cannot encode URL as data URL without downloading".to_string(),
            ))
        }
        FileInput::Path(path) => {
            let content = tokio::fs::read(path).await?;
            let content_type = mime_guess::from_path(path)
                .first_or_octet_stream()
                .to_string();

            let encoded = general_purpose::STANDARD.encode(&content);
            Ok(format!("data:{};base64,{}", content_type, encoded))
        }
        FileInput::Bytes {
            data, content_type, ..
        } => {
            let content_type = content_type
                .as_deref()
                .unwrap_or("application/octet-stream");

            let encoded = general_purpose::STANDARD.encode(data);
            Ok(format!("data:{};base64,{}", content_type, encoded))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_data_url_encoding() {
        let file_input = FileInput::from_bytes_with_metadata(
            &b"Hello, World!"[..],
            Some("test.txt".to_string()),
            Some("text/plain".to_string()),
        );

        let data_url = encode_file_as_data_url(&file_input).await.unwrap();
        assert_eq!(data_url, "data:text/plain;base64,SGVsbG8sIFdvcmxkIQ==");
    }

    #[tokio::test]
    async fn test_file_path_data_url() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        tokio::fs::write(&file_path, b"Test content").await.unwrap();

        let file_input = FileInput::from_path(&file_path);
        let data_url = encode_file_as_data_url(&file_input).await.unwrap();

        assert!(data_url.starts_with("data:text/plain;base64,"));
        assert!(data_url.contains("VGVzdCBjb250ZW50")); // "Test content" in base64
    }
}
