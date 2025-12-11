// AOF GUI - Tauri Command Handlers
// Wiring up AOF crates with the desktop application

pub mod agent;
pub mod config;
pub mod db;
pub mod mcp;
pub mod settings;

use serde::{Deserialize, Serialize};

// Re-export all commands for easy registration
pub use agent::*;
pub use config::*;
pub use db::*;
pub use mcp::*;
pub use settings::*;

/// Application info
#[derive(Debug, Serialize, Deserialize)]
pub struct AppInfo {
    pub version: String,
    pub name: String,
    pub aof_core_version: String,
}

/// Greet command - simple test command
#[tauri::command]
pub fn greet(name: String) -> String {
    format!("Hello, {}! Welcome to AOF Desktop.", name)
}

/// Get application version
#[tauri::command]
pub fn get_version() -> AppInfo {
    AppInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        name: env!("CARGO_PKG_NAME").to_string(),
        aof_core_version: aof_core::VERSION.to_string(),
    }
}
