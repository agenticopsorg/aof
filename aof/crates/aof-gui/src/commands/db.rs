// Database Commands - Tauri handlers for database persistence operations

use crate::db::{DbMcpServer, DbProviderApiKey};
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use tauri::State;

/// MCP server save request
#[derive(Debug, Deserialize)]
pub struct SaveMcpServerRequest {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
}

/// MCP server response
#[derive(Debug, Serialize)]
pub struct McpServerResponse {
    pub id: String,
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub tools: Vec<String>,  // Discovered tools from this server
    pub created_at: String,
}

impl From<DbMcpServer> for McpServerResponse {
    fn from(db: DbMcpServer) -> Self {
        Self {
            id: db.id.clone(),
            name: db.name.clone(),
            command: db.command.clone(),
            args: db.get_args_vec(),
            tools: db.get_tools_vec(),
            created_at: db.created_at,
        }
    }
}

/// Save MCP server configuration to database
#[tauri::command]
pub async fn db_save_mcp_server(
    request: SaveMcpServerRequest,
    state: State<'_, AppState>,
) -> Result<McpServerResponse, String> {
    tracing::info!("Saving MCP server: {} ({})", request.name, request.command);

    let db = state.get_db().await?;
    let server = DbMcpServer::new(request.name, request.command, request.args);

    sqlx::query(
        "INSERT OR REPLACE INTO mcp_servers (id, name, command, args, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(&server.id)
    .bind(&server.name)
    .bind(&server.command)
    .bind(&server.args)
    .bind(&server.created_at)
    .bind(&server.updated_at)
    .execute(&db)
    .await
    .map_err(|e| format!("Failed to save MCP server: {}", e))?;

    tracing::info!("✓ Saved MCP server: {}", server.name);
    Ok(server.into())
}

/// Load all MCP servers from database
#[tauri::command]
pub async fn db_load_mcp_servers(state: State<'_, AppState>) -> Result<Vec<McpServerResponse>, String> {
    let db = state.get_db().await?;

    let rows = sqlx::query("SELECT * FROM mcp_servers ORDER BY created_at DESC")
        .fetch_all(&db)
        .await
        .map_err(|e| format!("Failed to load MCP servers: {}", e))?;

    let servers: Vec<McpServerResponse> = rows
        .into_iter()
        .map(|row| {
            let args_json: String = row.get("args");
            let args: Vec<String> = serde_json::from_str(&args_json).unwrap_or_default();
            let tools_json: Option<String> = row.get("tools");
            let tools: Vec<String> = tools_json
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default();

            McpServerResponse {
                id: row.get("id"),
                name: row.get("name"),
                command: row.get("command"),
                args,
                tools,
                created_at: row.get("created_at"),
            }
        })
        .collect();

    tracing::debug!("Loaded {} MCP servers from database", servers.len());
    Ok(servers)
}

/// Delete MCP server from database
#[tauri::command]
pub async fn db_delete_mcp_server(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.get_db().await?;

    sqlx::query("DELETE FROM mcp_servers WHERE id = ?")
        .bind(&id)
        .execute(&db)
        .await
        .map_err(|e| format!("Failed to delete MCP server: {}", e))?;

    tracing::info!("✓ Deleted MCP server: {}", id);
    Ok(())
}

/// Get a specific MCP server
#[tauri::command]
pub async fn db_get_mcp_server(
    id: String,
    state: State<'_, AppState>,
) -> Result<Option<McpServerResponse>, String> {
    let db = state.get_db().await?;

    let row = sqlx::query("SELECT * FROM mcp_servers WHERE id = ?")
        .bind(&id)
        .fetch_optional(&db)
        .await
        .map_err(|e| format!("Failed to get MCP server: {}", e))?;

    Ok(row.map(|row| {
        let args_json: String = row.get("args");
        let args: Vec<String> = serde_json::from_str(&args_json).unwrap_or_default();
        let tools_json: Option<String> = row.get("tools");
        let tools: Vec<String> = tools_json
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();

        McpServerResponse {
            id: row.get("id"),
            name: row.get("name"),
            command: row.get("command"),
            args,
            tools,
            created_at: row.get("created_at"),
        }
    }))
}

// ============================================================================
// Provider API Key Commands
// ============================================================================

/// Provider API key save request
#[derive(Debug, Deserialize)]
pub struct SaveProviderApiKeyRequest {
    pub provider: String,
    pub api_key: String,
    pub base_url: Option<String>,
    pub default_model: Option<String>,
}

/// Provider API key response (masks the actual key)
#[derive(Debug, Serialize)]
pub struct ProviderApiKeyResponse {
    pub provider: String,
    pub api_key_masked: String,
    pub base_url: Option<String>,
    pub default_model: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<DbProviderApiKey> for ProviderApiKeyResponse {
    fn from(db: DbProviderApiKey) -> Self {
        // Mask API key for display (show first 4 and last 4 chars)
        let masked = if db.api_key.len() > 12 {
            format!("{}...{}", &db.api_key[..4], &db.api_key[db.api_key.len()-4..])
        } else if db.api_key.len() > 4 {
            format!("{}...", &db.api_key[..4])
        } else {
            "****".to_string()
        };

        Self {
            provider: db.provider,
            api_key_masked: masked,
            base_url: db.base_url,
            default_model: db.default_model,
            created_at: db.created_at,
            updated_at: db.updated_at,
        }
    }
}

/// Save provider API key to database
#[tauri::command]
pub async fn db_save_provider_api_key(
    request: SaveProviderApiKeyRequest,
    state: State<'_, AppState>,
) -> Result<ProviderApiKeyResponse, String> {
    tracing::info!("Saving API key for provider: {}", request.provider);

    let db = state.get_db().await?;
    let api_key = DbProviderApiKey::new(
        request.provider.clone(),
        request.api_key,
        request.base_url,
        request.default_model,
    );

    sqlx::query(
        "INSERT OR REPLACE INTO provider_api_keys (provider, api_key, base_url, default_model, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(&api_key.provider)
    .bind(&api_key.api_key)
    .bind(&api_key.base_url)
    .bind(&api_key.default_model)
    .bind(&api_key.created_at)
    .bind(&api_key.updated_at)
    .execute(&db)
    .await
    .map_err(|e| format!("Failed to save API key: {}", e))?;

    tracing::info!("✓ Saved API key for provider: {}", api_key.provider);
    Ok(api_key.into())
}

/// Load all provider API keys from database
#[tauri::command]
pub async fn db_load_provider_api_keys(state: State<'_, AppState>) -> Result<Vec<ProviderApiKeyResponse>, String> {
    let db = state.get_db().await?;

    let rows = sqlx::query("SELECT * FROM provider_api_keys ORDER BY provider")
        .fetch_all(&db)
        .await
        .map_err(|e| format!("Failed to load API keys: {}", e))?;

    let keys: Vec<ProviderApiKeyResponse> = rows
        .into_iter()
        .map(|row| {
            DbProviderApiKey {
                provider: row.get("provider"),
                api_key: row.get("api_key"),
                base_url: row.get("base_url"),
                default_model: row.get("default_model"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }.into()
        })
        .collect();

    tracing::debug!("Loaded {} provider API keys from database", keys.len());
    Ok(keys)
}

/// Delete provider API key from database
#[tauri::command]
pub async fn db_delete_provider_api_key(provider: String, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.get_db().await?;

    sqlx::query("DELETE FROM provider_api_keys WHERE provider = ?")
        .bind(&provider)
        .execute(&db)
        .await
        .map_err(|e| format!("Failed to delete API key: {}", e))?;

    tracing::info!("✓ Deleted API key for provider: {}", provider);
    Ok(())
}

/// Get API key for a specific provider (returns actual key for internal use)
#[tauri::command]
pub async fn db_get_provider_api_key(
    provider: String,
    state: State<'_, AppState>,
) -> Result<Option<String>, String> {
    let db = state.get_db().await?;

    let row = sqlx::query("SELECT api_key FROM provider_api_keys WHERE provider = ?")
        .bind(&provider)
        .fetch_optional(&db)
        .await
        .map_err(|e| format!("Failed to get API key: {}", e))?;

    Ok(row.map(|r| r.get("api_key")))
}

/// Get provider configuration (returns masked key and config)
#[tauri::command]
pub async fn db_get_provider_config(
    provider: String,
    state: State<'_, AppState>,
) -> Result<Option<ProviderApiKeyResponse>, String> {
    let db = state.get_db().await?;

    let row = sqlx::query("SELECT * FROM provider_api_keys WHERE provider = ?")
        .bind(&provider)
        .fetch_optional(&db)
        .await
        .map_err(|e| format!("Failed to get provider config: {}", e))?;

    Ok(row.map(|row| {
        DbProviderApiKey {
            provider: row.get("provider"),
            api_key: row.get("api_key"),
            base_url: row.get("base_url"),
            default_model: row.get("default_model"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }.into()
    }))
}
