// Database Commands - Tauri handlers for database persistence operations

use crate::db::DbMcpServer;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::ops::Deref;
use tauri::AppHandle;

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
    app: AppHandle,
) -> Result<McpServerResponse, String> {
    use tauri::Manager;

    let instances = app.state::<tauri_plugin_sql::DbInstances>();
    let pools = instances.0.read().await;
    let pool = pools
        .get("sqlite:aof.db")
        .ok_or_else(|| "Database not found".to_string())?;

    let db = match pool {
        tauri_plugin_sql::DbPool::Sqlite(db) => db,
        _ => return Err("Expected SQLite database".to_string()),
    };

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
    .execute(db)
    .await
    .map_err(|e| format!("Failed to save MCP server: {}", e))?;

    Ok(server.into())
}

/// Load all MCP servers from database
#[tauri::command]
pub async fn db_load_mcp_servers(app: AppHandle) -> Result<Vec<McpServerResponse>, String> {
    use tauri::Manager;

    let instances = app.state::<tauri_plugin_sql::DbInstances>();
    let pools = instances.0.read().await;
    let pool = pools
        .get("sqlite:aof.db")
        .ok_or_else(|| "Database not found".to_string())?;

    let db = match pool {
        tauri_plugin_sql::DbPool::Sqlite(db) => db,
        _ => return Err("Expected SQLite database".to_string()),
    };

    let rows = sqlx::query("SELECT * FROM mcp_servers ORDER BY created_at DESC")
        .fetch_all(db)
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

    Ok(servers)
}

/// Delete MCP server from database
#[tauri::command]
pub async fn db_delete_mcp_server(id: String, app: AppHandle) -> Result<(), String> {
    use tauri::Manager;

    let instances = app.state::<tauri_plugin_sql::DbInstances>();
    let pools = instances.0.read().await;
    let pool = pools
        .get("sqlite:aof.db")
        .ok_or_else(|| "Database not found".to_string())?;

    let db = match pool {
        tauri_plugin_sql::DbPool::Sqlite(db) => db,
        _ => return Err("Expected SQLite database".to_string()),
    };

    sqlx::query("DELETE FROM mcp_servers WHERE id = ?")
        .bind(&id)
        .execute(db)
        .await
        .map_err(|e| format!("Failed to delete MCP server: {}", e))?;

    Ok(())
}

/// Get a specific MCP server
#[tauri::command]
pub async fn db_get_mcp_server(
    id: String,
    app: AppHandle,
) -> Result<Option<McpServerResponse>, String> {
    use tauri::Manager;

    let instances = app.state::<tauri_plugin_sql::DbInstances>();
    let pools = instances.0.read().await;
    let pool = pools
        .get("sqlite:aof.db")
        .ok_or_else(|| "Database not found".to_string())?;

    let db = match pool {
        tauri_plugin_sql::DbPool::Sqlite(db) => db,
        _ => return Err("Expected SQLite database".to_string()),
    };

    let row = sqlx::query("SELECT * FROM mcp_servers WHERE id = ?")
        .bind(&id)
        .fetch_optional(db)
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
