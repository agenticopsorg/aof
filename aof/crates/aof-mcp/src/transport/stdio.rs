use async_trait::async_trait;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::Mutex;
use tracing::debug;

use super::{McpRequest, McpResponse, McpTransport, TransportType};
use aof_core::{AofError, AofResult};

/// Stdio transport for MCP
pub struct StdioTransport {
    process: Arc<Mutex<Option<Child>>>,
    stdin: Arc<Mutex<Option<ChildStdin>>>,
    stdout: Arc<Mutex<Option<BufReader<ChildStdout>>>>,
    command: String,
    args: Vec<String>,
}

impl StdioTransport {
    pub fn new(command: impl Into<String>, args: Vec<String>) -> Self {
        Self {
            process: Arc::new(Mutex::new(None)),
            stdin: Arc::new(Mutex::new(None)),
            stdout: Arc::new(Mutex::new(None)),
            command: command.into(),
            args,
        }
    }
}

#[async_trait]
impl McpTransport for StdioTransport {
    async fn init(&mut self) -> AofResult<()> {
        debug!("Initializing stdio transport: {} {:?}", self.command, self.args);

        // Build the full command with args
        let full_command = if self.args.is_empty() {
            self.command.clone()
        } else {
            format!("{} {}", self.command, self.args.join(" "))
        };

        // Use shell to resolve PATH and execute command
        let mut child = Command::new("sh")
            .arg("-c")
            .arg(&full_command)
            .env("PATH", std::env::var("PATH").unwrap_or_else(|_|
                "/usr/local/bin:/usr/bin:/bin:/opt/homebrew/bin".to_string()
            ))
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| AofError::mcp(format!("Failed to spawn process: {}", e)))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| AofError::mcp("Failed to get stdin"))?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| AofError::mcp("Failed to get stdout"))?;

        *self.stdin.lock().await = Some(stdin);
        *self.stdout.lock().await = Some(BufReader::new(stdout));
        *self.process.lock().await = Some(child);

        Ok(())
    }

    async fn request(&self, request: &McpRequest) -> AofResult<McpResponse> {
        let mut stdin_guard = self.stdin.lock().await;
        let stdin = stdin_guard
            .as_mut()
            .ok_or_else(|| AofError::mcp("Transport not initialized"))?;

        let mut stdout_guard = self.stdout.lock().await;
        let stdout = stdout_guard
            .as_mut()
            .ok_or_else(|| AofError::mcp("Transport not initialized"))?;

        // Send request
        let request_json = serde_json::to_string(request)?;
        debug!("Sending MCP request: {}", request_json);

        stdin
            .write_all(request_json.as_bytes())
            .await
            .map_err(|e| AofError::mcp(format!("Failed to write request: {}", e)))?;

        stdin
            .write_all(b"\n")
            .await
            .map_err(|e| AofError::mcp(format!("Failed to write newline: {}", e)))?;

        // Read response
        let mut response_line = String::new();
        stdout
            .read_line(&mut response_line)
            .await
            .map_err(|e| AofError::mcp(format!("Failed to read response: {}", e)))?;

        debug!("Received MCP response: {}", response_line);

        let response: McpResponse = serde_json::from_str(&response_line)?;

        if let Some(error) = &response.error {
            return Err(AofError::mcp(format!(
                "MCP error {}: {}",
                error.code, error.message
            )));
        }

        Ok(response)
    }

    fn transport_type(&self) -> TransportType {
        TransportType::Stdio
    }

    async fn shutdown(&mut self) -> AofResult<()> {
        let mut process_guard = self.process.lock().await;
        if let Some(mut process) = process_guard.take() {
            debug!("Shutting down stdio transport");
            process
                .kill()
                .await
                .map_err(|e| AofError::mcp(format!("Failed to kill process: {}", e)))?;
        }
        Ok(())
    }
}
