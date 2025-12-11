// Memory Commands - Tauri handlers for memory/state viewing

use serde::{Deserialize, Serialize};
use tauri::State;
use crate::state::AppState;

/// Memory entry for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: String,
    pub key: String,
    pub value: serde_json::Value,
    pub entry_type: String,
    pub created_at: String,
    pub updated_at: String,
    pub metadata: Option<serde_json::Value>,
}

/// Get memory entries
#[tauri::command]
pub async fn memory_get_entries(state: State<'_, AppState>) -> Result<Vec<MemoryEntry>, String> {
    // Collect memory entries from various sources
    let mut entries = Vec::new();

    // Add agent states as memory entries
    let agents = state.agents.read().await;
    for (id, runtime) in agents.iter() {
        entries.push(MemoryEntry {
            id: format!("agent_{}", id),
            key: format!("agent.{}.state", runtime.name),
            value: serde_json::json!({
                "status": format!("{:?}", runtime.status),
                "output_lines": runtime.output.len(),
                "has_error": runtime.error.is_some(),
            }),
            entry_type: "agent_state".to_string(),
            created_at: runtime.started_at.map(|t| t.to_rfc3339()).unwrap_or_default(),
            updated_at: runtime.finished_at.map(|t| t.to_rfc3339()).unwrap_or_else(|| chrono::Utc::now().to_rfc3339()),
            metadata: None,
        });
    }

    // Add saved configurations as memory entries
    let configs = state.configs.read().await;
    for (id, (meta, _yaml)) in configs.iter() {
        entries.push(MemoryEntry {
            id: format!("config_{}", id),
            key: format!("config.{}", meta.name),
            value: serde_json::json!({
                "name": meta.name,
                "description": meta.description,
            }),
            entry_type: "config".to_string(),
            created_at: meta.created_at.clone(),
            updated_at: meta.modified_at.clone(),
            metadata: None,
        });
    }

    // Add MCP connections as memory entries
    let mcp_connections = state.mcp_connections.read().await;
    for (id, conn) in mcp_connections.iter() {
        entries.push(MemoryEntry {
            id: format!("mcp_{}", id),
            key: format!("mcp.connection.{}", id),
            value: serde_json::json!({
                "server_command": conn.server_command,
                "status": format!("{:?}", conn.status),
                "tools_count": conn.tools.len(),
            }),
            entry_type: "mcp_connection".to_string(),
            created_at: conn.connected_at.map(|t| t.to_rfc3339()).unwrap_or_default(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            metadata: Some(serde_json::json!({
                "tools": conn.tools.iter().map(|t| &t.name).collect::<Vec<_>>(),
            })),
        });
    }

    Ok(entries)
}
