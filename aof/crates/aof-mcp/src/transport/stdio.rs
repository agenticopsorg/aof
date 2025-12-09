use async_trait::async_trait;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tracing::{debug, error};

use super::{McpRequest, McpResponse, McpTransport, TransportType};
use aof_core::{AofError, AofResult};

/// Stdio transport for MCP
pub struct StdioTransport {
    process: Option<Child>,
    stdin: Option<ChildStdin>,
    stdout: Option<BufReader<ChildStdout>>,
    command: String,
    args: Vec<String>,
}

impl StdioTransport {
    pub fn new(command: impl Into<String>, args: Vec<String>) -> Self {
        Self {
            process: None,
            stdin: None,
            stdout: None,
            command: command.into(),
            args,
        }
    }
}

#[async_trait]
impl McpTransport for StdioTransport {
    async fn init(&mut self) -> AofResult<()> {
        debug!("Initializing stdio transport: {} {:?}", self.command, self.args);

        let mut child = Command::new(&self.command)
            .args(&self.args)
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

        self.stdin = Some(stdin);
        self.stdout = Some(BufReader::new(stdout));
        self.process = Some(child);

        Ok(())
    }

    async fn request(&self, request: &McpRequest) -> AofResult<McpResponse> {
        let stdin = self
            .stdin
            .as_ref()
            .ok_or_else(|| AofError::mcp("Transport not initialized"))?;

        let stdout = self
            .stdout
            .as_ref()
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
        if let Some(mut process) = self.process.take() {
            debug!("Shutting down stdio transport");
            process
                .kill()
                .await
                .map_err(|e| AofError::mcp(format!("Failed to kill process: {}", e)))?;
        }
        Ok(())
    }
}
