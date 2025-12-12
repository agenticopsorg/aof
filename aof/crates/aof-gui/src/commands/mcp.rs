// MCP Commands - Tauri handlers for Model Context Protocol operations

use aof_core::tool::ToolDefinition;
use aof_mcp::{McpClient, McpClientBuilder};
use serde::{Deserialize, Serialize};
use tauri::{Emitter, State};

use crate::state::AppState;

/// MCP connection info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConnectionInfo {
    pub id: String,
    pub server_command: String,
    pub status: McpConnectionStatus,
    pub tools_count: usize,
    pub connected_at: Option<String>,
}

/// MCP connection status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum McpConnectionStatus {
    Connected,
    Connecting,
    Disconnected,
    Error,
}

/// MCP tool info for frontend
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct McpToolInfo {
    pub name: String,
    pub description: Option<String>,
    pub input_schema: serde_json::Value,
}

impl From<ToolDefinition> for McpToolInfo {
    fn from(def: ToolDefinition) -> Self {
        Self {
            name: def.name,
            description: if def.description.is_empty() { None } else { Some(def.description) },
            input_schema: def.parameters,
        }
    }
}

/// MCP tool call request
#[derive(Debug, Deserialize)]
pub struct McpToolCallRequest {
    pub connection_id: String,
    pub tool_name: String,
    pub arguments: serde_json::Value,
}

/// MCP tool call response
#[derive(Debug, Serialize)]
pub struct McpToolCallResponse {
    pub success: bool,
    pub result: serde_json::Value,
    pub error: Option<String>,
    pub execution_time_ms: u64,
}

/// Saved MCP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub id: String,
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub created_at: String,
}

/// Stored MCP connection
pub struct McpConnection {
    pub id: String,
    pub server_command: String,
    pub client: McpClient,
    pub tools: Vec<ToolDefinition>,
    pub status: McpConnectionStatus,
    pub connected_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Connect to an MCP server
#[tauri::command]
pub async fn mcp_connect(
    server_command: String,
    args: Vec<String>,
    state: State<'_, AppState>,
    window: tauri::Window,
) -> Result<McpConnectionInfo, String> {
    let connection_id = uuid::Uuid::new_v4().to_string();

    // Emit connecting event
    let _ = window.emit("mcp-connecting", serde_json::json!({
        "connection_id": connection_id,
        "server_command": server_command,
    }));

    // Build MCP client
    let client = McpClientBuilder::new()
        .stdio(&server_command, args.clone())
        .build()
        .map_err(|e| format!("Failed to create MCP client: {}", e))?;

    // Initialize connection
    client
        .initialize()
        .await
        .map_err(|e| format!("Failed to initialize MCP connection: {}", e))?;

    // Get available tools
    let tools = client
        .list_tools()
        .await
        .map_err(|e| format!("Failed to list MCP tools: {}", e))?;

    let tools_count = tools.len();
    let connected_at = chrono::Utc::now();

    // Store connection
    let connection = McpConnection {
        id: connection_id.clone(),
        server_command: server_command.clone(),
        client,
        tools,
        status: McpConnectionStatus::Connected,
        connected_at: Some(connected_at),
    };

    {
        let mut connections = state.mcp_connections.write().await;
        connections.insert(connection_id.clone(), connection);
    }

    let info = McpConnectionInfo {
        id: connection_id.clone(),
        server_command,
        status: McpConnectionStatus::Connected,
        tools_count,
        connected_at: Some(connected_at.to_rfc3339()),
    };

    // Emit connected event
    let _ = window.emit("mcp-connected", serde_json::json!({
        "connection_id": connection_id,
        "tools_count": tools_count,
    }));

    Ok(info)
}

/// Disconnect from MCP server
#[tauri::command]
pub async fn mcp_disconnect(
    connection_id: String,
    state: State<'_, AppState>,
    window: tauri::Window,
) -> Result<(), String> {
    let mut connections = state.mcp_connections.write().await;

    if let Some(connection) = connections.remove(&connection_id) {
        // Shutdown client
        let _ = connection.client.shutdown().await;

        let _ = window.emit("mcp-disconnected", serde_json::json!({
            "connection_id": connection_id,
        }));

        Ok(())
    } else {
        Err(format!("Connection {} not found", connection_id))
    }
}

/// List MCP connections
#[tauri::command]
pub async fn mcp_list_connections(
    state: State<'_, AppState>,
) -> Result<Vec<McpConnectionInfo>, String> {
    let connections = state.mcp_connections.read().await;

    let list: Vec<McpConnectionInfo> = connections
        .values()
        .map(|conn| McpConnectionInfo {
            id: conn.id.clone(),
            server_command: conn.server_command.clone(),
            status: conn.status.clone(),
            tools_count: conn.tools.len(),
            connected_at: conn.connected_at.map(|t| t.to_rfc3339()),
        })
        .collect();

    Ok(list)
}

/// List tools for a connection
#[tauri::command]
pub async fn mcp_list_tools(
    connection_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<McpToolInfo>, String> {
    let connections = state.mcp_connections.read().await;

    if let Some(connection) = connections.get(&connection_id) {
        let tools: Vec<McpToolInfo> = connection
            .tools
            .iter()
            .cloned()
            .map(|t| t.into())
            .collect();
        Ok(tools)
    } else {
        Err(format!("Connection {} not found", connection_id))
    }
}

/// Call an MCP tool
#[tauri::command]
pub async fn mcp_call_tool(
    request: McpToolCallRequest,
    state: State<'_, AppState>,
    window: tauri::Window,
) -> Result<McpToolCallResponse, String> {
    let start_time = std::time::Instant::now();

    // Emit calling event
    let _ = window.emit("mcp-tool-calling", serde_json::json!({
        "connection_id": request.connection_id,
        "tool_name": request.tool_name,
    }));

    let connections = state.mcp_connections.read().await;

    if let Some(connection) = connections.get(&request.connection_id) {
        match connection
            .client
            .call_tool(&request.tool_name, request.arguments)
            .await
        {
            Ok(result) => {
                let execution_time = start_time.elapsed().as_millis() as u64;

                let _ = window.emit("mcp-tool-completed", serde_json::json!({
                    "connection_id": request.connection_id,
                    "tool_name": request.tool_name,
                    "execution_time_ms": execution_time,
                }));

                Ok(McpToolCallResponse {
                    success: true,
                    result,
                    error: None,
                    execution_time_ms: execution_time,
                })
            }
            Err(e) => {
                let execution_time = start_time.elapsed().as_millis() as u64;

                let _ = window.emit("mcp-tool-error", serde_json::json!({
                    "connection_id": request.connection_id,
                    "tool_name": request.tool_name,
                    "error": e.to_string(),
                }));

                Ok(McpToolCallResponse {
                    success: false,
                    result: serde_json::Value::Null,
                    error: Some(e.to_string()),
                    execution_time_ms: execution_time,
                })
            }
        }
    } else {
        Err(format!("Connection {} not found", request.connection_id))
    }
}

/// Get tool schema/details
#[tauri::command]
pub async fn mcp_get_tool(
    connection_id: String,
    tool_name: String,
    state: State<'_, AppState>,
) -> Result<McpToolInfo, String> {
    let connections = state.mcp_connections.read().await;

    if let Some(connection) = connections.get(&connection_id) {
        connection
            .tools
            .iter()
            .find(|t| t.name == tool_name)
            .cloned()
            .map(|t| t.into())
            .ok_or_else(|| format!("Tool {} not found", tool_name))
    } else {
        Err(format!("Connection {} not found", connection_id))
    }
}

/// Auto-connect to MCP servers that provide the required tools
/// This is called internally when an agent is run with tools configured
pub async fn auto_connect_for_tools(
    required_tools: &[String],
    state: &AppState,
    window: &tauri::Window,
) -> Result<Vec<String>, String> {
    if required_tools.is_empty() {
        return Ok(vec![]);
    }

    tracing::info!("[AUTO_CONNECT] Checking for tools: {:?}", required_tools);

    // Check if already have the tools connected
    let connected_tools: Vec<String> = {
        let connections = state.mcp_connections.read().await;
        connections.values()
            .flat_map(|c| c.tools.iter().map(|t| t.name.clone()))
            .collect()
    };

    let missing_tools: Vec<&String> = required_tools.iter()
        .filter(|t| !connected_tools.contains(t))
        .collect();

    if missing_tools.is_empty() {
        tracing::info!("[AUTO_CONNECT] All required tools already connected");
        return Ok(vec![]);
    }

    tracing::info!("[AUTO_CONNECT] Need to connect for tools: {:?}", missing_tools);

    // Load saved MCP servers from database
    let db = state.get_db().await?;
    let rows = sqlx::query("SELECT * FROM mcp_servers")
        .fetch_all(&db)
        .await
        .map_err(|e| format!("Failed to load MCP servers: {}", e))?;

    // Find servers that might have the tools we need
    let mut connected_server_ids = vec![];

    for row in rows {
        let server_id: String = sqlx::Row::get(&row, "id");
        let server_name: String = sqlx::Row::get(&row, "name");
        let server_command: String = sqlx::Row::get(&row, "command");
        let server_args_json: String = sqlx::Row::get(&row, "args");
        let server_tools_json: Option<String> = sqlx::Row::get(&row, "tools");

        // Check if this server has the tools we need (from stored tools list)
        let server_tools: Vec<String> = server_tools_json
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();

        let has_needed_tools = missing_tools.iter()
            .any(|t| server_tools.contains(t));

        // Also check if server name matches a tool name (common pattern)
        let name_matches_tool = missing_tools.iter()
            .any(|t| server_name.to_lowercase().contains(&t.to_lowercase()) ||
                     t.to_lowercase().contains(&server_name.to_lowercase()));

        if !has_needed_tools && !name_matches_tool && !server_tools.is_empty() {
            continue; // Skip this server, it doesn't have what we need
        }

        // Check if already connected
        let already_connected = {
            let connections = state.mcp_connections.read().await;
            connections.values().any(|c| c.server_command == server_command)
        };

        if already_connected {
            tracing::info!("[AUTO_CONNECT] Server {} already connected", server_name);
            continue;
        }

        tracing::info!("[AUTO_CONNECT] Auto-connecting to server: {} ({})", server_name, server_command);

        // Parse args
        let args: Vec<String> = serde_json::from_str(&server_args_json).unwrap_or_default();

        // Emit connecting event
        let _ = window.emit("mcp-auto-connecting", serde_json::json!({
            "server_name": server_name,
            "server_command": server_command,
        }));

        // Connect to the server
        match connect_mcp_server_internal(&server_command, args.clone(), state, window).await {
            Ok(connection_id) => {
                tracing::info!("[AUTO_CONNECT] Successfully connected to {}", server_name);
                connected_server_ids.push(connection_id.clone());

                // Update the database with discovered tools
                if let Ok(tools) = get_connection_tools(&connection_id, state).await {
                    let tools_json = serde_json::to_string(&tools).unwrap_or_else(|_| "[]".to_string());
                    let _ = sqlx::query("UPDATE mcp_servers SET tools = ?, updated_at = ? WHERE id = ?")
                        .bind(&tools_json)
                        .bind(chrono::Utc::now().to_rfc3339())
                        .bind(&server_id)
                        .execute(&db)
                        .await;
                    tracing::info!("[AUTO_CONNECT] Updated server {} with discovered tools: {:?}", server_name, tools);
                }
            }
            Err(e) => {
                tracing::warn!("[AUTO_CONNECT] Failed to connect to {}: {}", server_name, e);
                let _ = window.emit("mcp-auto-connect-failed", serde_json::json!({
                    "server_name": server_name,
                    "error": e,
                }));
            }
        }
    }

    Ok(connected_server_ids)
}

/// Internal helper to connect to MCP server
async fn connect_mcp_server_internal(
    server_command: &str,
    args: Vec<String>,
    state: &AppState,
    window: &tauri::Window,
) -> Result<String, String> {
    let connection_id = uuid::Uuid::new_v4().to_string();

    // Build MCP client
    let client = McpClientBuilder::new()
        .stdio(server_command, args)
        .build()
        .map_err(|e| format!("Failed to create MCP client: {}", e))?;

    // Initialize connection
    client
        .initialize()
        .await
        .map_err(|e| format!("Failed to initialize MCP connection: {}", e))?;

    // Get available tools
    let tools = client
        .list_tools()
        .await
        .map_err(|e| format!("Failed to list MCP tools: {}", e))?;

    let tools_count = tools.len();
    let connected_at = chrono::Utc::now();

    // Store connection
    let connection = McpConnection {
        id: connection_id.clone(),
        server_command: server_command.to_string(),
        client,
        tools,
        status: McpConnectionStatus::Connected,
        connected_at: Some(connected_at),
    };

    {
        let mut connections = state.mcp_connections.write().await;
        connections.insert(connection_id.clone(), connection);
    }

    // Emit connected event
    let _ = window.emit("mcp-connected", serde_json::json!({
        "connection_id": connection_id,
        "tools_count": tools_count,
        "auto_connected": true,
    }));

    Ok(connection_id)
}

/// Get tool names from a connection
async fn get_connection_tools(connection_id: &str, state: &AppState) -> Result<Vec<String>, String> {
    let connections = state.mcp_connections.read().await;
    if let Some(connection) = connections.get(connection_id) {
        Ok(connection.tools.iter().map(|t| t.name.clone()).collect())
    } else {
        Err("Connection not found".to_string())
    }
}
