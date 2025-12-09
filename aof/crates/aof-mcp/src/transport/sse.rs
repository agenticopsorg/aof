use async_trait::async_trait;
use aof_core::{AofError, AofResult};

use super::{McpRequest, McpResponse, McpTransport, TransportType};

/// SSE transport for MCP (Server-Sent Events)
pub struct SseTransport {
    endpoint: String,
}

impl SseTransport {
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
        }
    }
}

#[async_trait]
impl McpTransport for SseTransport {
    async fn init(&mut self) -> AofResult<()> {
        // TODO: Implement SSE transport initialization
        Ok(())
    }

    async fn request(&self, _request: &McpRequest) -> AofResult<McpResponse> {
        // TODO: Implement SSE request/response
        Err(AofError::mcp("SSE transport not yet implemented"))
    }

    fn transport_type(&self) -> TransportType {
        TransportType::Sse
    }

    async fn shutdown(&mut self) -> AofResult<()> {
        Ok(())
    }
}
