use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::{AofError, AofResult};

/// Memory backend trait - pluggable persistence for agent state
///
/// Implementations should optimize for read performance and minimize allocations.
#[async_trait]
pub trait MemoryBackend: Send + Sync {
    /// Store memory entry
    async fn store(&self, key: &str, entry: MemoryEntry) -> AofResult<()>;

    /// Retrieve memory entry
    async fn retrieve(&self, key: &str) -> AofResult<Option<MemoryEntry>>;

    /// Delete memory entry
    async fn delete(&self, key: &str) -> AofResult<()>;

    /// List keys (with optional prefix filter)
    async fn list_keys(&self, prefix: Option<&str>) -> AofResult<Vec<String>>;

    /// Clear all entries
    async fn clear(&self) -> AofResult<()>;

    /// Search entries by metadata
    async fn search(&self, query: &MemoryQuery) -> AofResult<Vec<MemoryEntry>> {
        // Default implementation: filter in-memory
        let keys = self.list_keys(query.prefix.as_deref()).await?;
        let mut results = Vec::new();

        for key in keys {
            if let Some(entry) = self.retrieve(&key).await? {
                if query.matches(&entry) {
                    results.push(entry);
                }
            }
        }

        Ok(results)
    }
}

/// High-level memory interface for agents
#[async_trait]
pub trait Memory: Send + Sync {
    /// Store a value
    async fn store(&self, key: &str, value: serde_json::Value) -> AofResult<()>;

    /// Retrieve a value
    async fn retrieve<T: serde::de::DeserializeOwned>(&self, key: &str) -> AofResult<Option<T>>;

    /// Delete a value
    async fn delete(&self, key: &str) -> AofResult<()>;

    /// List all keys
    async fn list_keys(&self) -> AofResult<Vec<String>>;

    /// Clear all memory
    async fn clear(&self) -> AofResult<()>;
}

/// Memory entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    /// Entry key
    pub key: String,

    /// Entry value (JSON)
    pub value: serde_json::Value,

    /// Timestamp (Unix epoch ms)
    pub timestamp: u64,

    /// Entry metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,

    /// TTL (seconds, optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<u64>,
}

impl MemoryEntry {
    /// Create new entry
    pub fn new(key: impl Into<String>, value: serde_json::Value) -> Self {
        Self {
            key: key.into(),
            value,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            metadata: HashMap::new(),
            ttl: None,
        }
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Set TTL
    pub fn with_ttl(mut self, ttl_secs: u64) -> Self {
        self.ttl = Some(ttl_secs);
        self
    }

    /// Check if entry is expired
    pub fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;
            let expiry = self.timestamp + (ttl * 1000);
            now > expiry
        } else {
            false
        }
    }
}

/// Memory query for searching
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryQuery {
    /// Key prefix filter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,

    /// Metadata filters (key-value pairs)
    #[serde(default)]
    pub metadata: HashMap<String, String>,

    /// Limit results
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,

    /// Include expired entries
    #[serde(default)]
    pub include_expired: bool,
}

impl MemoryQuery {
    /// Check if entry matches query
    pub fn matches(&self, entry: &MemoryEntry) -> bool {
        // Check expiry
        if !self.include_expired && entry.is_expired() {
            return false;
        }

        // Check metadata filters
        for (key, value) in &self.metadata {
            if entry.metadata.get(key) != Some(value) {
                return false;
            }
        }

        true
    }
}

/// Reference-counted memory backend
pub type MemoryBackendRef = Arc<dyn MemoryBackend>;

/// Reference-counted memory
pub type MemoryRef = Arc<dyn Memory>;
