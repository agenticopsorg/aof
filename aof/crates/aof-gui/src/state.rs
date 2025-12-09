// Application State - Shared state managed by Tauri

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::commands::agent::AgentRuntime;
use crate::commands::config::ConfigMetadata;
use crate::commands::mcp::McpConnection;

/// Main application state
#[derive(Clone)]
pub struct AppState {
    /// Active agent runtimes
    pub agents: Arc<RwLock<HashMap<String, AgentRuntime>>>,

    /// Saved configurations (id -> (metadata, yaml_content))
    pub configs: Arc<RwLock<HashMap<String, (ConfigMetadata, String)>>>,

    /// Active MCP connections
    pub mcp_connections: Arc<RwLock<HashMap<String, McpConnection>>>,

    /// Application settings
    pub settings: Arc<RwLock<AppSettings>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            configs: Arc::new(RwLock::new(HashMap::new())),
            mcp_connections: Arc::new(RwLock::new(HashMap::new())),
            settings: Arc::new(RwLock::new(AppSettings::default())),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

/// Application settings
#[derive(Debug, Clone)]
pub struct AppSettings {
    /// Default model to use
    pub default_model: String,

    /// Default temperature
    pub default_temperature: f32,

    /// Auto-save configurations
    pub auto_save: bool,

    /// Theme (dark/light)
    pub theme: String,

    /// Log level
    pub log_level: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            default_model: "claude-3-5-sonnet-20241022".to_string(),
            default_temperature: 0.7,
            auto_save: true,
            theme: "dark".to_string(),
            log_level: "info".to_string(),
        }
    }
}
