//! Common types and structures used across the API.

use serde::{Deserialize, Serialize};

/// Generic API response wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    /// The response data
    #[serde(flatten)]
    pub data: T,
}

/// Paginated response structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    /// The results for this page
    pub results: Vec<T>,
    /// URL for the next page (if available)
    pub next: Option<String>,
    /// URL for the previous page (if available)
    pub previous: Option<String>,
}

impl<T> PaginatedResponse<T> {
    /// Check if there are more pages
    pub fn has_next(&self) -> bool {
        self.next.is_some()
    }
    
    /// Check if there are previous pages
    pub fn has_previous(&self) -> bool {
        self.previous.is_some()
    }
    
    /// Get the number of results in this page
    pub fn len(&self) -> usize {
        self.results.len()
    }
    
    /// Check if this page is empty
    pub fn is_empty(&self) -> bool {
        self.results.is_empty()
    }
}

/// Hardware configuration for running models.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hardware {
    /// Hardware identifier
    pub sku: String,
    /// Human-readable name
    pub name: String,
}

/// Model version metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelVersion {
    /// Version ID
    pub id: String,
    /// When this version was created
    pub created_at: String,
    /// Cog version used
    pub cog_version: Option<String>,
    /// OpenAPI schema for the model
    pub openapi_schema: Option<serde_json::Value>,
}

/// Basic model information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    /// Model owner
    pub owner: String,
    /// Model name
    pub name: String,
    /// Model description
    pub description: Option<String>,
    /// Model visibility
    pub visibility: String,
    /// GitHub URL
    pub github_url: Option<String>,
    /// Paper URL
    pub paper_url: Option<String>,
    /// License URL
    pub license_url: Option<String>,
    /// Cover image URL
    pub cover_image_url: Option<String>,
    /// Latest version
    pub latest_version: Option<ModelVersion>,
}

impl Model {
    /// Get the full model identifier (owner/name)
    pub fn identifier(&self) -> String {
        format!("{}/{}", self.owner, self.name)
    }
} 