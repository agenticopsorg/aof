// Application State - Shared state managed by Tauri

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use aof_runtime::RuntimeOrchestrator;
use crate::commands::agent::AgentRuntime;
use crate::commands::config::ConfigMetadata;
use crate::commands::mcp::{McpConnection, McpServerConfig};

/// Main application state
#[derive(Clone)]
pub struct AppState {
    /// Active agent runtimes
    pub agents: Arc<RwLock<HashMap<String, AgentRuntime>>>,

    /// Saved configurations (id -> (metadata, yaml_content))
    pub configs: Arc<RwLock<HashMap<String, (ConfigMetadata, String)>>>,

    /// Active MCP connections
    pub mcp_connections: Arc<RwLock<HashMap<String, McpConnection>>>,

    /// Saved MCP server configurations
    pub mcp_server_configs: Arc<RwLock<HashMap<String, McpServerConfig>>>,

    /// Application settings
    pub settings: Arc<RwLock<AppSettings>>,

    /// Runtime orchestrator for task execution
    pub orchestrator: Arc<RuntimeOrchestrator>,

    /// SQLite database pool
    pub db: Arc<RwLock<Option<sqlx::SqlitePool>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            configs: Arc::new(RwLock::new(HashMap::new())),
            mcp_connections: Arc::new(RwLock::new(HashMap::new())),
            mcp_server_configs: Arc::new(RwLock::new(HashMap::new())),
            settings: Arc::new(RwLock::new(AppSettings::default())),
            orchestrator: Arc::new(RuntimeOrchestrator::with_max_concurrent(5)),
            db: Arc::new(RwLock::new(None)),
        }
    }

    /// Initialize the database pool
    pub async fn init_db(&self, db_path: PathBuf) -> Result<(), String> {
        use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

        tracing::info!("Initializing database at: {:?}", db_path);

        // Create parent directory if needed
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create database directory: {}", e))?;
        }

        // Use SqliteConnectOptions for proper path handling
        let options = SqliteConnectOptions::new()
            .filename(&db_path)
            .create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
            .map_err(|e| format!("Failed to create database pool: {}", e))?;

        tracing::info!("✓ Database pool created successfully");

        // Run migrations
        let migrations = crate::db::get_migrations();
        for migration in migrations {
            tracing::info!("Running migration v{}: {}", migration.version, migration.description);
            if let Err(e) = sqlx::query(&migration.sql).execute(&pool).await {
                // Ignore "already exists" errors for IF NOT EXISTS statements
                if !e.to_string().contains("already exists") {
                    tracing::warn!("Migration warning: {}", e);
                }
            }
        }

        tracing::info!("✓ Migrations completed");

        // Store the pool
        let mut db_guard = self.db.write().await;
        *db_guard = Some(pool);

        tracing::info!("✓ Database ready");
        Ok(())
    }

    /// Get the database pool
    pub async fn get_db(&self) -> Result<sqlx::SqlitePool, String> {
        let db_guard = self.db.read().await;
        db_guard.clone().ok_or_else(|| "Database not initialized".to_string())
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
            default_model: "gemini-2.0-flash".to_string(),
            default_temperature: 0.7,
            auto_save: true,
            theme: "dark".to_string(),
            log_level: "info".to_string(),
        }
    }
}
