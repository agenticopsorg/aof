// Database Commands - Tauri handlers for database persistence operations

use crate::db::DbMcpServer;
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
    pub created_at: String,
}

impl From<DbMcpServer> for McpServerResponse {
    fn from(db: DbMcpServer) -> Self {
        Self {
            id: db.id.clone(),
            name: db.name.clone(),
            command: db.command.clone(),
            args: db.get_args_vec(),
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

            McpServerResponse {
                id: row.get("id"),
                name: row.get("name"),
                command: row.get("command"),
                args,
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

        McpServerResponse {
            id: row.get("id"),
            name: row.get("name"),
            command: row.get("command"),
            args,
            created_at: row.get("created_at"),
        }
    }))
}
