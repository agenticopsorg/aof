//! Central message handler for routing and execution
//!
//! This module coordinates message handling across platforms,
//! parsing commands, and executing them through the runtime.

use std::collections::HashMap;
use std::sync::Arc;

use tracing::{debug, error, info, warn};

use crate::command::{CommandError, CommandType, TriggerCommand, TriggerTarget};
use crate::platforms::{PlatformError, TriggerMessage, TriggerPlatform};
use crate::response::{TriggerResponse, TriggerResponseBuilder};
use aof_core::{AofError, AofResult};
use aof_runtime::{RuntimeOrchestrator, Task};

/// Helper trait to convert CommandError to AofError
trait CommandErrorExt<T> {
    fn map_cmd_err(self) -> AofResult<T>;
}

impl<T> CommandErrorExt<T> for Result<T, CommandError> {
    fn map_cmd_err(self) -> AofResult<T> {
        self.map_err(|e| AofError::Config(e.to_string()))
    }
}

/// Handler configuration
#[derive(Debug, Clone)]
pub struct TriggerHandlerConfig {
    /// Enable verbose logging
    pub verbose: bool,

    /// Auto-acknowledge commands
    pub auto_ack: bool,

    /// Maximum concurrent tasks per user
    pub max_tasks_per_user: usize,

    /// Command timeout in seconds
    pub command_timeout_secs: u64,
}

impl Default for TriggerHandlerConfig {
    fn default() -> Self {
        Self {
            verbose: false,
            auto_ack: true,
            max_tasks_per_user: 3,
            command_timeout_secs: 300, // 5 minutes
        }
    }
}

/// Central trigger handler
///
/// Routes messages from platforms to appropriate handlers and
/// executes commands through the runtime orchestrator.
pub struct TriggerHandler {
    /// Runtime orchestrator for task execution
    orchestrator: Arc<RuntimeOrchestrator>,

    /// Registered platforms
    platforms: HashMap<String, Arc<dyn TriggerPlatform>>,

    /// Handler configuration
    config: TriggerHandlerConfig,

    /// User task counters (user_id -> active task count)
    user_tasks: Arc<dashmap::DashMap<String, usize>>,
}

impl TriggerHandler {
    /// Create a new trigger handler
    pub fn new(orchestrator: Arc<RuntimeOrchestrator>) -> Self {
        Self {
            orchestrator,
            platforms: HashMap::new(),
            config: TriggerHandlerConfig::default(),
            user_tasks: Arc::new(dashmap::DashMap::new()),
        }
    }

    /// Create handler with custom configuration
    pub fn with_config(orchestrator: Arc<RuntimeOrchestrator>, config: TriggerHandlerConfig) -> Self {
        Self {
            orchestrator,
            platforms: HashMap::new(),
            config,
            user_tasks: Arc::new(dashmap::DashMap::new()),
        }
    }

    /// Register a platform
    pub fn register_platform(&mut self, platform: Arc<dyn TriggerPlatform>) {
        let name = platform.platform_name();
        info!("Registering platform: {}", name);
        self.platforms.insert(name.to_string(), platform);
    }

    /// Get registered platform
    pub fn get_platform(&self, name: &str) -> Option<&Arc<dyn TriggerPlatform>> {
        self.platforms.get(name)
    }

    /// Handle incoming message from platform
    pub async fn handle_message(&self, platform: &str, message: TriggerMessage) -> AofResult<()> {
        debug!(
            "Handling message from {}: {} (user: {})",
            platform, message.id, message.user.id
        );

        // Get platform for response
        let platform_impl = self
            .platforms
            .get(platform)
            .ok_or_else(|| aof_core::AofError::agent(format!("Unknown platform: {}", platform)))?;

        // Check if user has too many active tasks
        if let Some(count) = self.user_tasks.get(&message.user.id) {
            if *count >= self.config.max_tasks_per_user {
                let response = TriggerResponseBuilder::new()
                    .text(format!(
                        "You have too many active tasks ({}). Please wait for some to complete.",
                        *count
                    ))
                    .error()
                    .build();

                let _ = platform_impl.send_response(&message.channel_id, response).await;
                return Ok(());
            }
        }

        // Parse command
        let cmd = match TriggerCommand::parse(&message) {
            Ok(cmd) => cmd,
            Err(e) => {
                warn!("Failed to parse command: {}", e);
                let response = self.handle_parse_error(&message, e).await;
                let _ = platform_impl.send_response(&message.channel_id, response).await;
                return Ok(());
            }
        };

        // Auto-acknowledge if enabled
        if self.config.auto_ack {
            let ack = TriggerResponseBuilder::new()
                .text("Processing your request...")
                .build();
            let _ = platform_impl.send_response(&message.channel_id, ack).await;
        }

        // Execute command
        let response = match self.execute_command(cmd).await {
            Ok(resp) => resp,
            Err(e) => {
                error!("Command execution failed: {}", e);
                TriggerResponseBuilder::new()
                    .text(format!("Command failed: {}", e))
                    .error()
                    .build()
            }
        };

        // Send response
        if let Err(e) = platform_impl.send_response(&message.channel_id, response).await {
            error!("Failed to send response: {:?}", e);
        }

        Ok(())
    }

    /// Execute a parsed command
    pub async fn execute_command(&self, cmd: TriggerCommand) -> AofResult<TriggerResponse> {
        info!(
            "Executing command: {:?} {:?} (user: {})",
            cmd.command_type, cmd.target, cmd.context.user_id
        );

        match cmd.command_type {
            CommandType::Run => self.handle_run_command(cmd).await,
            CommandType::Create => self.handle_create_command(cmd).await,
            CommandType::Status => self.handle_status_command(cmd).await,
            CommandType::Cancel => self.handle_cancel_command(cmd).await,
            CommandType::List => self.handle_list_command(cmd).await,
            CommandType::Help => Ok(self.handle_help_command(cmd).await),
            CommandType::Info => Ok(self.handle_info_command(cmd).await),
        }
    }

    /// Handle run command
    async fn handle_run_command(&self, cmd: TriggerCommand) -> AofResult<TriggerResponse> {
        match cmd.target {
            TriggerTarget::Agent => {
                let agent_name = cmd.get_arg(0).map_cmd_err()?;
                let input = cmd.args[1..].join(" ");

                // Create task
                let task_id = format!("trigger-{}-{}", cmd.context.user_id, uuid::Uuid::new_v4());
                let task = Task::new(task_id.clone(), agent_name.to_string(), agent_name.to_string(), input);

                // Submit to orchestrator
                let handle = self.orchestrator.submit_task(task);

                // Track user task
                self.increment_user_tasks(&cmd.context.user_id);

                // Start execution (simplified - in real implementation would use actual agent executor)
                let user_id = cmd.context.user_id.clone();
                let user_tasks = Arc::clone(&self.user_tasks);
                let handle_clone = Arc::clone(&handle);

                tokio::spawn(async move {
                    // Placeholder: would actually execute agent here
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

                    // Decrement user task count
                    if let Some(mut count) = user_tasks.get_mut(&user_id) {
                        if *count > 0 {
                            *count -= 1;
                        }
                    }
                });

                Ok(TriggerResponseBuilder::new()
                    .text(format!(
                        "✓ Task started: `{}`\nAgent: {}\nUse `/status task {}` to check progress",
                        task_id, agent_name, task_id
                    ))
                    .success()
                    .build())
            }
            _ => Ok(TriggerResponseBuilder::new()
                .text(format!("Run command not supported for {:?}", cmd.target))
                .error()
                .build()),
        }
    }

    /// Handle create command
    async fn handle_create_command(&self, cmd: TriggerCommand) -> AofResult<TriggerResponse> {
        Ok(TriggerResponseBuilder::new()
            .text("Create command not yet implemented")
            .build())
    }

    /// Handle status command
    async fn handle_status_command(&self, cmd: TriggerCommand) -> AofResult<TriggerResponse> {
        match cmd.target {
            TriggerTarget::Task => {
                let task_id = cmd.get_arg(0).map_cmd_err()?;

                if let Some(handle) = self.orchestrator.get_task(task_id) {
                    let task = handle.task().await;
                    let status = handle.status().await;

                    Ok(TriggerResponseBuilder::new()
                        .text(format!(
                            "Task: {}\nStatus: {:?}\nAgent: {}",
                            task.id, status, task.agent_name
                        ))
                        .build())
                } else {
                    Ok(TriggerResponseBuilder::new()
                        .text(format!("Task not found: {}", task_id))
                        .error()
                        .build())
                }
            }
            _ => Ok(TriggerResponseBuilder::new()
                .text(format!("Status not supported for {:?}", cmd.target))
                .error()
                .build()),
        }
    }

    /// Handle cancel command
    async fn handle_cancel_command(&self, cmd: TriggerCommand) -> AofResult<TriggerResponse> {
        match cmd.target {
            TriggerTarget::Task => {
                let task_id = cmd.get_arg(0).map_cmd_err()?;

                match self.orchestrator.cancel_task(task_id).await {
                    Ok(_) => Ok(TriggerResponseBuilder::new()
                        .text(format!("✓ Task cancelled: {}", task_id))
                        .success()
                        .build()),
                    Err(e) => Ok(TriggerResponseBuilder::new()
                        .text(format!("Failed to cancel task: {}", e))
                        .error()
                        .build()),
                }
            }
            _ => Ok(TriggerResponseBuilder::new()
                .text(format!("Cancel not supported for {:?}", cmd.target))
                .error()
                .build()),
        }
    }

    /// Handle list command
    async fn handle_list_command(&self, cmd: TriggerCommand) -> AofResult<TriggerResponse> {
        match cmd.target {
            TriggerTarget::Task => {
                let task_ids = self.orchestrator.list_tasks();
                let stats = self.orchestrator.stats().await;

                let text = format!(
                    "Tasks:\n• Pending: {}\n• Running: {}\n• Completed: {}\n• Failed: {}\n\nTask IDs:\n{}",
                    stats.pending,
                    stats.running,
                    stats.completed,
                    stats.failed,
                    task_ids.join("\n")
                );

                Ok(TriggerResponseBuilder::new().text(text).build())
            }
            _ => Ok(TriggerResponseBuilder::new()
                .text(format!("List not supported for {:?}", cmd.target))
                .error()
                .build()),
        }
    }

    /// Handle help command
    async fn handle_help_command(&self, _cmd: TriggerCommand) -> TriggerResponse {
        let help_text = r#"
**AOF Bot Commands**

**Basic Commands:**
• `/run agent <name> <input>` - Run an agent
• `/status task <id>` - Check task status
• `/cancel task <id>` - Cancel a running task
• `/list tasks` - List all tasks
• `/help` - Show this help

**Examples:**
• `/run agent monitor Check server health`
• `/status task trigger-user123-abc`
• `/list tasks`

**Support:** https://github.com/yourusername/aof
        "#;

        TriggerResponseBuilder::new()
            .text(help_text.trim())
            .build()
    }

    /// Handle info command
    async fn handle_info_command(&self, _cmd: TriggerCommand) -> TriggerResponse {
        let stats = self.orchestrator.stats().await;

        let info_text = format!(
            r#"
**AOF System Info**

**Version:** {}
**Runtime Stats:**
• Max Concurrent: {}
• Available Permits: {}
• Active Tasks: {}
• Pending: {}
• Running: {}
• Completed: {}
• Failed: {}

**Platforms:** {}
            "#,
            crate::VERSION,
            stats.max_concurrent,
            stats.available_permits,
            stats.pending + stats.running,
            stats.pending,
            stats.running,
            stats.completed,
            stats.failed,
            self.platforms.keys().map(|k| k.as_str()).collect::<Vec<_>>().join(", ")
        );

        TriggerResponseBuilder::new()
            .text(info_text.trim())
            .build()
    }

    /// Handle parse error
    async fn handle_parse_error(
        &self,
        _message: &TriggerMessage,
        error: CommandError,
    ) -> TriggerResponse {
        let text = match error {
            CommandError::InvalidFormat(msg) => {
                format!("Invalid command format: {}\n\nUse `/help` for usage.", msg)
            }
            CommandError::UnknownCommand(cmd) => {
                format!("Unknown command: {}\n\nUse `/help` for available commands.", cmd)
            }
            CommandError::MissingArgument(arg) => {
                format!("Missing required argument: {}\n\nUse `/help` for usage.", arg)
            }
            CommandError::InvalidTarget(target) => {
                format!("Invalid target: {}\n\nValid targets: agent, task, fleet, flow", target)
            }
        };

        TriggerResponseBuilder::new().text(text).error().build()
    }

    /// Increment user task count
    fn increment_user_tasks(&self, user_id: &str) {
        self.user_tasks
            .entry(user_id.to_string())
            .and_modify(|count| *count += 1)
            .or_insert(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_handler_creation() {
        let orchestrator = Arc::new(RuntimeOrchestrator::new());
        let handler = TriggerHandler::new(orchestrator);

        assert_eq!(handler.platforms.len(), 0);
        assert!(handler.config.auto_ack);
    }
}
