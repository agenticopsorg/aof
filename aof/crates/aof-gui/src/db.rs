// Database module for SQLite persistence

use serde::{Deserialize, Serialize};
use tauri_plugin_sql::{Migration, MigrationKind};

/// Initialize the database with migrations
pub fn get_migrations() -> Vec<Migration> {
    vec![
        // Migration 1: Create mcp_servers table
        Migration {
            version: 1,
            description: "Create mcp_servers table",
            sql: r#"
                CREATE TABLE IF NOT EXISTS mcp_servers (
                    id TEXT PRIMARY KEY NOT NULL,
                    name TEXT NOT NULL,
                    command TEXT NOT NULL,
                    args TEXT NOT NULL,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL
                );
                CREATE INDEX IF NOT EXISTS idx_mcp_servers_created_at ON mcp_servers(created_at);
            "#,
            kind: MigrationKind::Up,
        },
        // Migration 2: Create agent_configs table
        Migration {
            version: 2,
            description: "Create agent_configs table",
            sql: r#"
                CREATE TABLE IF NOT EXISTS agent_configs (
                    id TEXT PRIMARY KEY NOT NULL,
                    name TEXT NOT NULL,
                    provider TEXT NOT NULL,
                    model TEXT NOT NULL,
                    temperature REAL NOT NULL DEFAULT 0.7,
                    max_tokens INTEGER,
                    system_prompt TEXT,
                    tools TEXT,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL
                );
                CREATE INDEX IF NOT EXISTS idx_agent_configs_created_at ON agent_configs(created_at);
            "#,
            kind: MigrationKind::Up,
        },
        // Migration 3: Create agent_runs table
        Migration {
            version: 3,
            description: "Create agent_runs table",
            sql: r#"
                CREATE TABLE IF NOT EXISTS agent_runs (
                    id TEXT PRIMARY KEY NOT NULL,
                    agent_config_id TEXT NOT NULL,
                    status TEXT NOT NULL,
                    started_at TEXT NOT NULL,
                    completed_at TEXT,
                    error TEXT,
                    total_tokens INTEGER DEFAULT 0,
                    total_cost REAL DEFAULT 0.0,
                    FOREIGN KEY (agent_config_id) REFERENCES agent_configs(id) ON DELETE CASCADE
                );
                CREATE INDEX IF NOT EXISTS idx_agent_runs_started_at ON agent_runs(started_at);
                CREATE INDEX IF NOT EXISTS idx_agent_runs_agent_config_id ON agent_runs(agent_config_id);
            "#,
            kind: MigrationKind::Up,
        },
        // Migration 4: Create conversations table
        Migration {
            version: 4,
            description: "Create conversations table",
            sql: r#"
                CREATE TABLE IF NOT EXISTS conversations (
                    id TEXT PRIMARY KEY NOT NULL,
                    agent_run_id TEXT NOT NULL,
                    role TEXT NOT NULL,
                    content TEXT NOT NULL,
                    timestamp TEXT NOT NULL,
                    metadata TEXT,
                    FOREIGN KEY (agent_run_id) REFERENCES agent_runs(id) ON DELETE CASCADE
                );
                CREATE INDEX IF NOT EXISTS idx_conversations_timestamp ON conversations(timestamp);
                CREATE INDEX IF NOT EXISTS idx_conversations_agent_run_id ON conversations(agent_run_id);
            "#,
            kind: MigrationKind::Up,
        },
        // Migration 5: Create app_settings table
        Migration {
            version: 5,
            description: "Create app_settings table",
            sql: r#"
                CREATE TABLE IF NOT EXISTS app_settings (
                    key TEXT PRIMARY KEY NOT NULL,
                    value TEXT NOT NULL,
                    updated_at TEXT NOT NULL
                );
            "#,
            kind: MigrationKind::Up,
        },
        // Migration 6: Create provider_api_keys table
        Migration {
            version: 6,
            description: "Create provider_api_keys table",
            sql: r#"
                CREATE TABLE IF NOT EXISTS provider_api_keys (
                    provider TEXT PRIMARY KEY NOT NULL,
                    api_key TEXT NOT NULL,
                    base_url TEXT,
                    default_model TEXT,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL
                );
            "#,
            kind: MigrationKind::Up,
        },
    ]
}

/// MCP Server database model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbMcpServer {
    pub id: String,
    pub name: String,
    pub command: String,
    pub args: String, // JSON array stored as string
    pub created_at: String,
    pub updated_at: String,
}

impl DbMcpServer {
    pub fn new(name: String, command: String, args: Vec<String>) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            command,
            args: serde_json::to_string(&args).unwrap_or_else(|_| "[]".to_string()),
            created_at: now.clone(),
            updated_at: now,
        }
    }

    pub fn get_args_vec(&self) -> Vec<String> {
        serde_json::from_str(&self.args).unwrap_or_default()
    }
}

/// Agent Config database model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbAgentConfig {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: Option<i32>,
    pub system_prompt: Option<String>,
    pub tools: Option<String>, // JSON array stored as string
    pub created_at: String,
    pub updated_at: String,
}

/// Agent Run database model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbAgentRun {
    pub id: String,
    pub agent_config_id: String,
    pub status: String,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub error: Option<String>,
    pub total_tokens: i32,
    pub total_cost: f32,
}

/// Conversation message database model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbConversation {
    pub id: String,
    pub agent_run_id: String,
    pub role: String,
    pub content: String,
    pub timestamp: String,
    pub metadata: Option<String>, // JSON object stored as string
}

/// Provider API Key database model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbProviderApiKey {
    pub provider: String,
    pub api_key: String,
    pub base_url: Option<String>,
    pub default_model: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl DbProviderApiKey {
    pub fn new(provider: String, api_key: String, base_url: Option<String>, default_model: Option<String>) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            provider,
            api_key,
            base_url,
            default_model,
            created_at: now.clone(),
            updated_at: now,
        }
    }
}
