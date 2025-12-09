use async_trait::async_trait;
use aof_core::{AofError, AofResult};

use super::{McpRequest, McpResponse, McpTransport, TransportType};

/// HTTP transport for MCP
pub struct HttpTransport {
    endpoint: String,
}

impl HttpTransport {
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
        }
    }
}

#[async_trait]
impl McpTransport for HttpTransport {
    async fn init(&mut self) -> AofResult<()> {
        // TODO: Implement HTTP transport initialization
        Ok(())
    }

    async fn request(&self, _request: &McpRequest) -> AofResult<McpResponse> {
        // TODO: Implement HTTP request/response
        Err(AofError::mcp("HTTP transport not yet implemented"))
    }

    fn transport_type(&self) -> TransportType {
        TransportType::Http
    }

    async fn shutdown(&mut self) -> AofResult<()> {
        Ok(())
    }
}
