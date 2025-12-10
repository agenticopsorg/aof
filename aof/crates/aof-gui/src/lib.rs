// AOF GUI - Desktop Application Library
// Integrates aof-core, aof-mcp, aof-llm, aof-memory with Tauri

pub mod commands;
pub mod state;

use state::AppState;
use tauri::Manager;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Run the Tauri application
pub fn run() {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "aof_gui=debug,aof_core=debug,aof_mcp=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting AOF Desktop v{}", env!("CARGO_PKG_VERSION"));
    tracing::info!("AOF Core v{}", aof_core::VERSION);

    tauri::Builder::default()
        .manage(AppState::new())
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }

            tracing::info!("AOF Desktop initialized successfully");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Basic commands
            commands::greet,
            commands::get_version,
            // Agent commands
            commands::agent_run,
            commands::agent_stop,
            commands::agent_status,
            commands::agent_list,
            commands::agent_clear_completed,
            commands::agent_orchestrator_stats,
            // Config commands
            commands::config_validate,
            commands::config_save,
            commands::config_load,
            commands::config_list,
            commands::config_delete,
            commands::config_generate_example,
            // MCP commands
            commands::mcp_connect,
            commands::mcp_disconnect,
            commands::mcp_list_connections,
            commands::mcp_list_tools,
            commands::mcp_call_tool,
            commands::mcp_get_tool,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
