//! File handling types for inputs and outputs.

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Represents different ways to provide file input to a model.
#[derive(Debug, Clone)]
pub enum FileInput {
    /// A URL to a publicly accessible file
    Url(String),
    /// A local file path
    Path(PathBuf),
    /// Raw bytes with optional filename and content type
    Bytes {
        data: Bytes,
        filename: Option<String>,
        content_type: Option<String>,
    },
}

impl FileInput {
    /// Create a file input from a URL
    pub fn from_url(url: impl Into<String>) -> Self {
        Self::Url(url.into())
    }

    /// Create a file input from a local path
    pub fn from_path(path: impl AsRef<Path>) -> Self {
        Self::Path(path.as_ref().to_path_buf())
    }

    /// Create a file input from raw bytes
    pub fn from_bytes(data: impl Into<Bytes>) -> Self {
        Self::Bytes {
            data: data.into(),
            filename: None,
            content_type: None,
        }
    }

    /// Create a file input from bytes with metadata
    pub fn from_bytes_with_metadata(
        data: impl Into<Bytes>,
        filename: Option<String>,
        content_type: Option<String>,
    ) -> Self {
        Self::Bytes {
            data: data.into(),
            filename,
            content_type,
        }
    }

    /// Check if this is a URL input
    pub fn is_url(&self) -> bool {
        matches!(self, Self::Url(_))
    }

    /// Check if this is a file path input
    pub fn is_path(&self) -> bool {
        matches!(self, Self::Path(_))
    }

    /// Check if this is a bytes input
    pub fn is_bytes(&self) -> bool {
        matches!(self, Self::Bytes { .. })
    }

    /// Get the URL if this is a URL input
    pub fn as_url(&self) -> Option<&str> {
        match self {
            Self::Url(url) => Some(url),
            _ => None,
        }
    }

    /// Get the path if this is a path input
    pub fn as_path(&self) -> Option<&Path> {
        match self {
            Self::Path(path) => Some(path),
            _ => None,
        }
    }
}

impl From<String> for FileInput {
    fn from(s: String) -> Self {
        if s.starts_with("http://") || s.starts_with("https://") {
            Self::Url(s)
        } else {
            Self::Path(PathBuf::from(s))
        }
    }
}

impl From<&str> for FileInput {
    fn from(s: &str) -> Self {
        Self::from(s.to_string())
    }
}

impl From<PathBuf> for FileInput {
    fn from(path: PathBuf) -> Self {
        Self::Path(path)
    }
}

impl From<&Path> for FileInput {
    fn from(path: &Path) -> Self {
        Self::Path(path.to_path_buf())
    }
}

/// Represents a file output from a model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOutput {
    /// The URL to download the file
    pub url: String,
    /// Optional filename
    pub filename: Option<String>,
    /// Optional content type
    pub content_type: Option<String>,
    /// Optional file size in bytes
    pub size: Option<u64>,
}

impl FileOutput {
    /// Create a new file output
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            filename: None,
            content_type: None,
            size: None,
        }
    }

    /// Set the filename
    pub fn with_filename(mut self, filename: impl Into<String>) -> Self {
        self.filename = Some(filename.into());
        self
    }

    /// Set the content type
    pub fn with_content_type(mut self, content_type: impl Into<String>) -> Self {
        self.content_type = Some(content_type.into());
        self
    }

    /// Set the file size
    pub fn with_size(mut self, size: u64) -> Self {
        self.size = Some(size);
        self
    }

    /// Download the file as bytes
    pub async fn download(&self) -> crate::Result<Bytes> {
        let response = reqwest::get(&self.url).await?;
        let bytes = response.bytes().await?;
        Ok(bytes)
    }

    /// Save the file to a local path
    pub async fn save_to_path(&self, path: impl AsRef<Path>) -> crate::Result<()> {
        let bytes = self.download().await?;
        tokio::fs::write(path, bytes).await?;
        Ok(())
    }
}

impl From<String> for FileOutput {
    fn from(url: String) -> Self {
        Self::new(url)
    }
}

impl From<&str> for FileOutput {
    fn from(url: &str) -> Self {
        Self::new(url)
    }
}

/// File encoding strategy for uploads.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileEncodingStrategy {
    /// Upload files as base64-encoded data URLs
    Base64DataUrl,
    /// Upload files as multipart form data
    Multipart,
}

impl Default for FileEncodingStrategy {
    fn default() -> Self {
        Self::Multipart
    }
}
