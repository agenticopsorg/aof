# MCP (Model Context Protocol) Rust Client Implementation Research

**Research Date**: 2025-12-09
**Agent**: Researcher (AOF Hive Mind Swarm)
**Objective**: Research requirements for building a Rust MCP client library

---

## Executive Summary

The Model Context Protocol (MCP) is an open standard developed by Anthropic for connecting AI assistants to data sources and tools. This research provides comprehensive findings on implementing a Rust-based MCP client library, including protocol specifications, transport mechanisms, existing implementations, and recommended Rust crates.

**Key Findings**:
- MCP uses JSON-RPC 2.0 as its wire format over multiple transport types
- Three Rust implementations exist: mcpr, mcp-client-rs, and prism-mcp-rs
- Current specification (2025-03-26) defines stdio and StreamableHTTP transports (SSE deprecated)
- Recommended crates: tokio, reqwest, eventsource-client, serde_json

---

## 1. MCP Protocol Specification

### 1.1 Message Format (JSON-RPC 2.0)

MCP uses [JSON-RPC 2.0](https://modelcontextprotocol.io/specification/2025-03-26/basic) as its wire format. All messages MUST be UTF-8 encoded and follow the JSON-RPC 2.0 specification.

#### Message Types

**Requests**:
```json
{
  "jsonrpc": "2.0",
  "id": "string | number",
  "method": "string",
  "params": { /* optional parameters */ }
}
```
- MUST include a unique string or integer ID (NOT null)
- ID MUST NOT have been previously used in the same session
- Method specifies the operation to perform

**Responses**:
```json
{
  "jsonrpc": "2.0",
  "id": "string | number",
  "result": { /* success data */ },
  "error": {
    "code": "number",
    "message": "string",
    "data": "unknown"
  }
}
```
- MUST include the same ID as the corresponding request
- Either `result` OR `error` MUST be set (not both)
- Error codes MUST be integers

**Notifications**:
```json
{
  "jsonrpc": "2.0",
  "method": "string",
  "params": { /* optional parameters */ }
}
```
- One-way messages with no reply
- MUST NOT include an ID field

#### Batching Support

MCP implementations MAY support sending JSON-RPC batches (multiple requests/notifications in an array) but MUST support receiving JSON-RPC batches.

### 1.2 Protocol Lifecycle

The MCP connection lifecycle consists of three phases:

**1. Initialization Phase**:
```json
// Client → Server: initialize request
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {
    "protocolVersion": "2025-03-26",
    "capabilities": { /* client capabilities */ },
    "clientInfo": {
      "name": "client-name",
      "version": "1.0.0"
    }
  }
}

// Server → Client: initialize response
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "protocolVersion": "2025-03-26",
    "capabilities": { /* server capabilities */ },
    "serverInfo": {
      "name": "server-name",
      "version": "1.0.0"
    },
    "instructions": "optional usage instructions"
  }
}

// Client → Server: initialized notification
{
  "jsonrpc": "2.0",
  "method": "notifications/initialized"
}
```

**2. Operation Phase**:
- Normal protocol communication after initialization
- Tool discovery and invocation
- Resource access
- Prompt handling

**3. Shutdown Phase**:
- Graceful termination of connection
- Cleanup of resources

### 1.3 Tool Discovery and Invocation

#### tools/list

Clients discover available tools through the `tools/list` endpoint:

```json
// Request
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/list"
}

// Response
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "tools": [
      {
        "name": "get_weather",
        "description": "Get current weather for a location",
        "inputSchema": {
          "type": "object",
          "properties": {
            "location": {
              "type": "string",
              "description": "City name"
            }
          },
          "required": ["location"]
        }
      }
    ]
  }
}
```

**Tool Schema Structure**:
- `name`: Unique identifier (required)
- `description`: Human-readable description (optional)
- `inputSchema`: JSON Schema defining parameters (type must be "object")

#### tools/call

Tools are invoked using the `tools/call` endpoint:

```json
// Request
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "get_weather",
    "arguments": {
      "location": "New York"
    }
  }
}

// Response
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "Temperature: 72°F, Conditions: Sunny"
      }
    ]
  }
}
```

**Tool Results**:
- May contain multiple content items
- Content types: text, image, audio, resource links, embedded resources
- Tool execution errors use `isError: true` flag (not JSON-RPC errors)

### 1.4 Resource Handling

Resources are client-controlled data structures that provide context to the AI model:

```json
// resources/list request
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "resources/list"
}

// resources/read request
{
  "jsonrpc": "2.0",
  "id": 5,
  "method": "resources/read",
  "params": {
    "uri": "resource://example/data"
  }
}
```

**Key Differences**:
- **Tools**: Model-controlled, active operations (APIs, computations)
- **Resources**: Client-controlled, passive data (documents, configurations)

### 1.5 Prompts Support

Prompts are reusable templates exposed by servers:

```json
// prompts/list request
{
  "jsonrpc": "2.0",
  "id": 6,
  "method": "prompts/list"
}

// prompts/get request
{
  "jsonrpc": "2.0",
  "id": 7,
  "method": "prompts/get",
  "params": {
    "name": "code_review",
    "arguments": {
      "language": "rust"
    }
  }
}
```

---

## 2. Transport Types

### 2.1 stdio Transport

**Description**: Communication over standard input/output streams, ideal for local integrations.

**Specification** ([MCP Transports](https://modelcontextprotocol.io/specification/2025-03-26/basic/transports)):
- Client launches MCP server as subprocess
- Server reads JSON-RPC messages from stdin
- Server writes JSON-RPC messages to stdout
- Messages are newline-delimited and MUST NOT contain embedded newlines
- Server MAY write UTF-8 logging to stderr (clients may capture/forward/ignore)

**Use Cases**:
- Command-line tools
- Local development environments
- Minimal latency requirements
- Simple deployment scenarios

**Example Flow**:
```
Client                          Server (subprocess)
  |                                   |
  |--- spawn subprocess ------------->|
  |                                   |
  |--- initialize (via stdin) ------->|
  |<-- initialize response (stdout)---|
  |                                   |
  |--- tools/list (via stdin) ------->|
  |<-- tools response (stdout) -------|
  |                                   |
  |--- tools/call (via stdin) ------->|
  |<-- result (stdout) ----------------|
```

### 2.2 StreamableHTTP Transport (Current Standard)

**Description**: HTTP-based transport replacing HTTP+SSE, supports multiple client connections.

**Specification** ([MCP Transports](https://modelcontextprotocol.io/specification/2025-03-26/basic/transports)):
- Single HTTP endpoint supporting POST and GET methods
- Server operates as independent process (multi-client)
- Optional SSE for streaming multiple server messages
- Session management via `Mcp-Session-Id` header

**Endpoint Requirements**:
```
POST /mcp    - Client requests
GET /mcp     - Optional SSE stream for server-to-client messages
```

**Session Management**:
- Server MAY assign session ID during initialization
- Returned in `Mcp-Session-Id` response header
- Client MUST include in subsequent requests
- Session ID SHOULD be globally unique and cryptographically secure (UUID, JWT, hash)

**Use Cases**:
- Web applications
- Remote/cloud deployments
- Browser integrations
- Multi-client scenarios

**Example Flow**:
```
Client                          Server (HTTP)
  |                                   |
  |--- POST /mcp (initialize) ------->|
  |<-- 200 OK + Mcp-Session-Id -------|
  |                                   |
  |--- GET /mcp (SSE stream) --------->|
  |<-- SSE: event stream -------------|
  |                                   |
  |--- POST /mcp (tools/list) -------->|
  |    + Mcp-Session-Id header        |
  |<-- 200 OK + tools ----------------|
```

### 2.3 SSE Transport (Deprecated)

**Status**: Deprecated as of specification version 2025-03-26

**Reason for Deprecation**:
- Required two separate endpoints (POST for requests, GET for SSE stream)
- Complicated session management and error handling
- StreamableHTTP provides unified approach

**Backwards Compatibility**:
- Clients MAY support legacy servers
- Try POST first (new protocol)
- If 4xx error, try GET for SSE (old protocol)

---

## 3. Existing Rust MCP Implementations

### 3.1 mcpr (Most Comprehensive)

**Repository**: [conikeec/mcpr](https://github.com/conikeec/mcpr)
**Documentation**: [docs.rs/mcpr](https://docs.rs/mcpr)
**Version**: 0.2.3
**Downloads**: Most popular Rust MCP implementation

**Features**:
- Complete MCP schema implementation
- Multiple transport support (stdio, SSE)
- Easy-to-use client and server APIs
- Template generation for quick setup
- GitHub Tools example demonstrating full client-server architecture

**Example Usage**:
```rust
use mcpr::client::Client;
use mcpr::transport::StdioTransport;

// Create stdio transport
let transport = StdioTransport::new("mcp-server-command");
let client = Client::new(transport);

// Initialize connection
client.initialize().await?;

// List tools
let tools = client.list_tools().await?;

// Call tool
let result = client.call_tool("tool_name", args).await?;
```

**Strengths**:
- Most actively maintained
- Complete feature set
- Good documentation and examples
- Production-ready

### 3.2 mcp-client-rs

**Repository**: [tim-schultz/mcp-client-rs](https://github.com/tim-schultz/mcp-client-rs)

**Description**: Client implementation for Rust projects using MCP protocol.

**Status**: Simpler implementation, less feature-complete than mcpr

### 3.3 prism-mcp-rs (Enterprise-Grade)

**Repository**: [prismworks-ai/prism-mcp-rs](https://github.com/prismworks-ai/prism-mcp-rs)

**Features**:
- Production-grade implementation
- Enterprise reliability patterns
- Circuit breaker pattern for failure isolation
- Adaptive retry policies with smart backoff
- Zero-downtime operations
- Plugin ecosystem support

**Strengths**:
- Fault-tolerant by design
- Scalable architecture
- Advanced error handling
- Suitable for high-availability systems

### 3.4 agenterra-rmcp

**Repository**: [clafollett/agenterra-rmcp](https://github.com/clafollett/agenterra-rmcp)

**Description**: Based on official MCP Rust SDK concepts from Anthropic/MCP community.

---

## 4. Rust Crate Recommendations

### 4.1 Async Runtime

**tokio** (Required)

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
```

**Features**:
- Industry-standard async runtime
- [Process management](https://docs.rs/tokio/latest/tokio/process/) for stdio transport
- I/O utilities for stream handling
- Mature ecosystem with extensive documentation

**Key Modules**:
- `tokio::process` - Async subprocess spawning and management
- `tokio::io` - Async I/O primitives
- `tokio::sync` - Async synchronization primitives

### 4.2 HTTP Client

**reqwest** (Recommended)

```toml
[dependencies]
reqwest = { version = "0.12", features = ["json", "stream"] }
```

**Features**:
- Async HTTP client built on tokio and hyper
- 5.3+ million downloads
- JSON support for MCP messages
- Streaming support for SSE
- Header management for session IDs
- Connection pooling and keep-alive

**Repository**: [seanmonstar/reqwest](https://github.com/seanmonstar/reqwest)

### 4.3 SSE Client (For Legacy Support)

**eventsource-client** (Recommended)

```toml
[dependencies]
eventsource-client = "0.15"
```

**Features**:
- LaunchDarkly-maintained, production-grade
- 5.3+ million downloads
- Built on tokio
- Custom header support (authorization)
- Automatic reconnection with exponential backoff
- Retry logic for failed connections

**Alternative**: **sse-client**
```toml
[dependencies]
sse-client = "1.1"
```
- 17,862 downloads
- Handles redirections and retries
- Less actively maintained than eventsource-client

**Repository**: [launchdarkly/rust-eventsource-client](https://github.com/launchdarkly/rust-eventsource-client)

### 4.4 JSON Serialization

**serde** + **serde_json** (Required)

```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

**Features**:
- De facto standard for Rust serialization
- JSON-RPC 2.0 message serialization
- Type-safe schema definitions
- Extensive ecosystem support

### 4.5 Process Management

**tokio::process** (Built-in)

```rust
use tokio::process::Command;
use std::process::Stdio;

let mut child = Command::new("mcp-server")
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()?;

let stdin = child.stdin.take().unwrap();
let stdout = child.stdout.take().unwrap();
```

**Features** ([tokio::process documentation](https://docs.rs/tokio/latest/tokio/process/)):
- Async subprocess spawning
- Piped stdin/stdout/stderr
- Async read/write to child process
- Child process lifecycle management
- Optional kill-on-drop behavior

### 4.6 Error Handling

**thiserror** (Recommended)

```toml
[dependencies]
thiserror = "1"
```

**Features**:
- Derive macros for custom error types
- Idiomatic Rust error handling
- Source chain support for error context

**anyhow** (Alternative for applications)

```toml
[dependencies]
anyhow = "1"
```

### 4.7 Additional Utilities

**url** - URL parsing
```toml
[dependencies]
url = "2"
```

**tracing** - Structured logging
```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = "0.3"
```

**async-trait** - Async trait definitions
```toml
[dependencies]
async-trait = "0.1"
```

---

## 5. Connection Management Best Practices

### 5.1 Timeout Management

**Specification** ([MCP Lifecycle](https://modelcontextprotocol.io/specification/2025-03-26/basic/lifecycle)):

- Implementations SHOULD establish timeouts for all sent requests
- Prevents hung connections and resource exhaustion
- Sender SHOULD issue cancellation notification on timeout
- SDKs SHOULD allow per-request timeout configuration
- MAY reset timeout on progress notifications
- SHOULD enforce maximum timeout regardless of progress

**Implementation Pattern**:
```rust
use tokio::time::{timeout, Duration};

async fn call_with_timeout<T>(
    future: impl Future<Output = T>,
    duration: Duration,
) -> Result<T, TimeoutError> {
    timeout(duration, future)
        .await
        .map_err(|_| TimeoutError::RequestTimeout)
}

// Usage
let result = call_with_timeout(
    client.call_tool("expensive_operation", args),
    Duration::from_secs(30),
).await?;
```

### 5.2 Retry Patterns

**Exponential Backoff with Jitter** ([Error Handling Guide](https://mcpcat.io/guides/error-handling-custom-mcp-servers/)):

```rust
use tokio::time::{sleep, Duration};
use rand::Rng;

async fn retry_with_backoff<T, E, F>(
    mut operation: F,
    max_retries: u32,
    base_delay: Duration,
) -> Result<T, E>
where
    F: FnMut() -> Pin<Box<dyn Future<Output = Result<T, E>>>>,
{
    let mut retries = 0;

    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(err) if retries >= max_retries => return Err(err),
            Err(_) => {
                let delay = base_delay * 2u32.pow(retries);
                let jitter = rand::thread_rng().gen_range(0..1000);
                let delay_with_jitter = delay + Duration::from_millis(jitter);

                sleep(delay_with_jitter).await;
                retries += 1;
            }
        }
    }
}
```

**Key Principles**:
- Incremental backoff (exponential growth)
- Maximum retry limits (prevent infinite loops)
- Randomized jitter (prevent thundering herd)
- Distinguishable error types (transient vs permanent)

### 5.3 Circuit Breaker Pattern

**Purpose**: Prevent cascading failures by failing fast when downstream services are unhealthy.

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
enum CircuitState {
    Closed,        // Normal operation
    Open(Instant), // Failing, reject requests
    HalfOpen,      // Testing recovery
}

struct CircuitBreaker {
    state: Arc<RwLock<CircuitState>>,
    failure_threshold: u32,
    success_threshold: u32,
    timeout: Duration,
    failures: Arc<RwLock<u32>>,
    successes: Arc<RwLock<u32>>,
}

impl CircuitBreaker {
    async fn call<T, E, F>(&self, operation: F) -> Result<T, E>
    where
        F: FnOnce() -> Pin<Box<dyn Future<Output = Result<T, E>>>>,
    {
        let state = self.state.read().await.clone();

        match state {
            CircuitState::Open(opened_at) => {
                if opened_at.elapsed() >= self.timeout {
                    *self.state.write().await = CircuitState::HalfOpen;
                    self.try_operation(operation).await
                } else {
                    Err(/* circuit open error */)
                }
            }
            CircuitState::HalfOpen | CircuitState::Closed => {
                self.try_operation(operation).await
            }
        }
    }

    async fn try_operation<T, E, F>(&self, operation: F) -> Result<T, E> {
        match operation().await {
            Ok(result) => {
                self.on_success().await;
                Ok(result)
            }
            Err(err) => {
                self.on_failure().await;
                Err(err)
            }
        }
    }

    async fn on_success(&self) {
        let mut successes = self.successes.write().await;
        *successes += 1;

        if *successes >= self.success_threshold {
            *self.state.write().await = CircuitState::Closed;
            *successes = 0;
            *self.failures.write().await = 0;
        }
    }

    async fn on_failure(&self) {
        let mut failures = self.failures.write().await;
        *failures += 1;

        if *failures >= self.failure_threshold {
            *self.state.write().await = CircuitState::Open(Instant::now());
            *failures = 0;
        }
    }
}
```

### 5.4 Error Handling Tiers

**Three-Tier Error Model** ([Error Handling in MCP](https://mcpcat.io/guides/error-handling-custom-mcp-servers/)):

**1. Transport-Level Errors**:
- Network timeouts
- Broken pipes
- Authentication failures
- Handled before MCP protocol engagement

**2. Protocol-Level Errors**:
- JSON-RPC 2.0 violations
- Malformed JSON
- Non-existent methods
- Invalid parameters
- Return standard JSON-RPC error responses

**3. Tool Execution Errors**:
- Application-level failures
- Use `isError: true` flag in response content
- NOT JSON-RPC errors
- Allows model to reason about errors and retry

**Error Response Pattern**:
```rust
// Protocol error (JSON-RPC)
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32601,  // Method not found
    "message": "Method 'unknown_method' not found"
  }
}

// Tool execution error (NOT JSON-RPC error)
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "Failed to connect to database",
        "isError": true
      }
    ]
  }
}
```

### 5.5 Graceful Degradation

**Principles**:
- Return partial results when some data unavailable
- Provide fallback responses for non-critical failures
- Communicate degraded state to caller

**Example**:
```rust
async fn get_weather_data(location: &str) -> Result<WeatherData> {
    let mut data = WeatherData::default();

    // Critical: current temperature
    data.temperature = fetch_temperature(location).await?;

    // Non-critical: forecast (fallback on error)
    data.forecast = fetch_forecast(location).await
        .unwrap_or_else(|_| "Forecast unavailable".to_string());

    // Non-critical: historical data (optional)
    data.historical = fetch_historical(location).await.ok();

    Ok(data)
}
```

### 5.6 Health Checks

**Proactive Monitoring**:
```rust
struct McpClient {
    health_check_interval: Duration,
}

impl McpClient {
    async fn start_health_monitoring(&self) {
        let mut interval = tokio::time::interval(self.health_check_interval);

        loop {
            interval.tick().await;

            match self.ping().await {
                Ok(_) => log::debug!("Server healthy"),
                Err(err) => {
                    log::warn!("Server health check failed: {}", err);
                    // Trigger reconnection logic
                    self.reconnect().await;
                }
            }
        }
    }
}
```

---

## 6. Sample Code Patterns

### 6.1 stdio Transport Client

```rust
use tokio::process::{Command, ChildStdin, ChildStdout};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use serde_json::{json, Value};
use std::process::Stdio;

struct StdioClient {
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    request_id: u64,
}

impl StdioClient {
    async fn new(command: &str, args: &[&str]) -> Result<Self> {
        let mut child = Command::new(command)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;

        let stdin = child.stdin.take().ok_or("Failed to get stdin")?;
        let stdout = BufReader::new(
            child.stdout.take().ok_or("Failed to get stdout")?
        );

        Ok(Self {
            stdin,
            stdout,
            request_id: 0,
        })
    }

    async fn send_request(&mut self, method: &str, params: Value) -> Result<Value> {
        self.request_id += 1;
        let request = json!({
            "jsonrpc": "2.0",
            "id": self.request_id,
            "method": method,
            "params": params,
        });

        // Write request with newline delimiter
        let request_str = serde_json::to_string(&request)?;
        self.stdin.write_all(request_str.as_bytes()).await?;
        self.stdin.write_all(b"\n").await?;
        self.stdin.flush().await?;

        // Read response line
        let mut response_line = String::new();
        self.stdout.read_line(&mut response_line).await?;

        let response: Value = serde_json::from_str(&response_line)?;

        if let Some(error) = response.get("error") {
            return Err(/* parse error */);
        }

        Ok(response["result"].clone())
    }

    async fn initialize(&mut self) -> Result<Value> {
        self.send_request("initialize", json!({
            "protocolVersion": "2025-03-26",
            "capabilities": {},
            "clientInfo": {
                "name": "rust-mcp-client",
                "version": "0.1.0"
            }
        })).await?;

        // Send initialized notification
        let notification = json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized",
        });
        let notification_str = serde_json::to_string(&notification)?;
        self.stdin.write_all(notification_str.as_bytes()).await?;
        self.stdin.write_all(b"\n").await?;
        self.stdin.flush().await?;

        Ok(Value::Null)
    }

    async fn list_tools(&mut self) -> Result<Vec<Tool>> {
        let result = self.send_request("tools/list", json!({})).await?;
        let tools: Vec<Tool> = serde_json::from_value(result["tools"].clone())?;
        Ok(tools)
    }

    async fn call_tool(&mut self, name: &str, arguments: Value) -> Result<Value> {
        self.send_request("tools/call", json!({
            "name": name,
            "arguments": arguments,
        })).await
    }
}
```

### 6.2 StreamableHTTP Transport Client

```rust
use reqwest::{Client as HttpClient, header};
use serde_json::{json, Value};
use std::time::Duration;

struct StreamableHttpClient {
    http_client: HttpClient,
    endpoint: String,
    session_id: Option<String>,
    request_id: u64,
}

impl StreamableHttpClient {
    fn new(endpoint: String) -> Self {
        let http_client = HttpClient::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap();

        Self {
            http_client,
            endpoint,
            session_id: None,
            request_id: 0,
        }
    }

    async fn send_request(&mut self, method: &str, params: Value) -> Result<Value> {
        self.request_id += 1;
        let request = json!({
            "jsonrpc": "2.0",
            "id": self.request_id,
            "method": method,
            "params": params,
        });

        let mut req = self.http_client
            .post(&self.endpoint)
            .json(&request);

        // Add session ID header if available
        if let Some(session_id) = &self.session_id {
            req = req.header("Mcp-Session-Id", session_id);
        }

        let response = req.send().await?;

        // Extract session ID from response headers
        if let Some(session_id) = response.headers().get("Mcp-Session-Id") {
            self.session_id = Some(session_id.to_str()?.to_string());
        }

        let response_json: Value = response.json().await?;

        if let Some(error) = response_json.get("error") {
            return Err(/* parse error */);
        }

        Ok(response_json["result"].clone())
    }

    async fn initialize(&mut self) -> Result<Value> {
        let result = self.send_request("initialize", json!({
            "protocolVersion": "2025-03-26",
            "capabilities": {},
            "clientInfo": {
                "name": "rust-mcp-client",
                "version": "0.1.0"
            }
        })).await?;

        // Send initialized notification (no response expected)
        self.http_client
            .post(&self.endpoint)
            .header("Mcp-Session-Id", self.session_id.as_ref().unwrap())
            .json(&json!({
                "jsonrpc": "2.0",
                "method": "notifications/initialized",
            }))
            .send()
            .await?;

        Ok(result)
    }

    async fn list_tools(&mut self) -> Result<Vec<Tool>> {
        let result = self.send_request("tools/list", json!({})).await?;
        let tools: Vec<Tool> = serde_json::from_value(result["tools"].clone())?;
        Ok(tools)
    }

    async fn call_tool(&mut self, name: &str, arguments: Value) -> Result<Value> {
        self.send_request("tools/call", json!({
            "name": name,
            "arguments": arguments,
        })).await
    }
}
```

### 6.3 SSE Stream Handling (Legacy Support)

```rust
use eventsource_client as es;
use futures::StreamExt;

async fn handle_sse_stream(url: &str, session_id: &str) -> Result<()> {
    let client = es::ClientBuilder::for_url(url)?
        .header("Mcp-Session-Id", session_id)?
        .build();

    let mut stream = client.stream();

    while let Some(event) = stream.next().await {
        match event {
            Ok(es::SSE::Event(event)) => {
                match event.event_type.as_str() {
                    "message" => {
                        let data: Value = serde_json::from_str(&event.data)?;
                        handle_server_message(data).await?;
                    }
                    "endpoint" => {
                        // Legacy SSE: endpoint event with request URL
                        let endpoint = event.data;
                        println!("Server endpoint: {}", endpoint);
                    }
                    _ => {
                        log::debug!("Unknown event type: {}", event.event_type);
                    }
                }
            }
            Ok(es::SSE::Comment(comment)) => {
                log::trace!("SSE comment: {}", comment);
            }
            Err(err) => {
                log::error!("SSE error: {}", err);
                // Implement reconnection logic
                break;
            }
        }
    }

    Ok(())
}
```

### 6.4 Unified Client Interface

```rust
use async_trait::async_trait;

#[async_trait]
trait McpTransport: Send + Sync {
    async fn initialize(&mut self) -> Result<InitializeResult>;
    async fn list_tools(&mut self) -> Result<Vec<Tool>>;
    async fn call_tool(&mut self, name: &str, arguments: Value) -> Result<ToolResult>;
    async fn list_resources(&mut self) -> Result<Vec<Resource>>;
    async fn read_resource(&mut self, uri: &str) -> Result<ResourceContent>;
}

struct McpClient<T: McpTransport> {
    transport: T,
    circuit_breaker: CircuitBreaker,
}

impl<T: McpTransport> McpClient<T> {
    fn new(transport: T) -> Self {
        Self {
            transport,
            circuit_breaker: CircuitBreaker::new(),
        }
    }

    async fn call_tool_with_resilience(
        &mut self,
        name: &str,
        arguments: Value,
    ) -> Result<ToolResult> {
        retry_with_backoff(
            || {
                Box::pin(self.circuit_breaker.call(|| {
                    Box::pin(self.transport.call_tool(name, arguments.clone()))
                }))
            },
            3, // max retries
            Duration::from_millis(100), // base delay
        ).await
    }
}

// Usage
let stdio_transport = StdioClient::new("mcp-server", &[]).await?;
let mut client = McpClient::new(stdio_transport);
client.transport.initialize().await?;

let http_transport = StreamableHttpClient::new("https://example.com/mcp".to_string());
let mut http_client = McpClient::new(http_transport);
http_client.transport.initialize().await?;
```

---

## 7. Popular MCP Servers for DevOps

### 7.1 Kubernetes MCP Servers

**1. Azure mcp-kubernetes**
- **Repository**: [Azure/mcp-kubernetes](https://github.com/Azure/mcp-kubernetes)
- **Description**: Bridge between AI tools (Claude, Cursor, GitHub Copilot) and Kubernetes
- **Features**: Direct Kubernetes API interaction, multiple kubeconfig source support

**2. Flux159/mcp-server-kubernetes**
- **Repository**: [Flux159/mcp-server-kubernetes](https://github.com/Flux159/mcp-server-kubernetes)
- **Description**: Kubernetes cluster management via MCP
- **Features**: Kubeconfig loading, cluster operations

**3. containers/kubernetes-mcp-server**
- **Repository**: [containers/kubernetes-mcp-server](https://github.com/containers/kubernetes-mcp-server)
- **Description**: Go-based native Kubernetes API implementation (NOT kubectl wrapper)
- **Features**: No external dependencies, direct API server interaction

**4. rohitg00/kubectl-mcp-server**
- **Description**: Natural language interaction with Kubernetes clusters
- **Features**: AI assistant integration (Claude, Cursor)

### 7.2 Prometheus MCP Servers

**1. pab1it0/prometheus-mcp-server**
- **Repository**: [pab1it0/prometheus-mcp-server](https://github.com/pab1it0/prometheus-mcp-server)
- **Description**: Prometheus metrics access via standardized MCP interfaces
- **Features**: PromQL query execution, metrics analysis

**2. idanfishman/prometheus-mcp**
- **Repository**: [idanfishman/prometheus-mcp](https://github.com/idanfishman/prometheus-mcp)
- **Description**: AI assistant integration with Prometheus monitoring
- **Features**: Natural language metric queries

**3. loglmhq/mcp-server-prometheus**
- **Repository**: [loglmhq/mcp-server-prometheus](https://github.com/loglmhq/mcp-server-prometheus)
- **Description**: MCP server for Prometheus interactions

**Use Cases**:
- Quick metrics access without dashboard switching
- Incident response acceleration
- Natural language observability queries

### 7.3 GitHub MCP Server

**Official GitHub MCP Server**
- **Description**: AI agent interaction with code repositories
- **Features**:
  - Repository operations
  - Issue creation and commenting
  - Pull request management (create, comment, merge)
  - Project metadata retrieval
- **Use Cases**: Automated code review, issue tracking, CI/CD integration

### 7.4 Filesystem MCP Server

**Description**: Secure file and directory access for AI assistants
**Features**:
- File management within chat interface
- Log file inspection
- Configuration updates
- Code analysis
- Bulk file operations
**Use Cases**: Configuration management, log analysis, local development

### 7.5 PostgreSQL MCP Server

**Description**: Database interaction via MCP protocol
**Features**:
- Query execution
- Schema inspection
- Data analysis
**Use Cases**: Database administration, data analysis, schema evolution

### 7.6 Additional DevOps MCP Servers

**Curated List**: [rohitg00/awesome-devops-mcp-servers](https://github.com/rohitg00/awesome-devops-mcp-servers)

**Emerging Servers**:
- Docker MCP (container management)
- Linear MCP (issue tracking)
- Playwright MCP (continuous testing)
- Cloud-native infrastructure utilities

**Configuration**: Most servers are straightforward to configure in MCP-compatible tools like:
- Claude Code
- GitHub Copilot
- Cursor
- Windsurf

---

## 8. Integration Architecture

### 8.1 Recommended Client Architecture

```
┌─────────────────────────────────────────────────────────┐
│                   Application Layer                      │
│  (Business logic, AI model integration)                 │
└────────────────────┬────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────┐
│               McpClient (Resilience)                     │
│  • Circuit Breaker                                      │
│  • Retry Logic                                          │
│  • Timeout Management                                   │
│  • Health Checks                                        │
└────────────────────┬────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────┐
│            Transport Abstraction Layer                   │
│  (Unified interface for all transport types)           │
└──┬──────────────────┬──────────────────┬────────────────┘
   │                  │                  │
┌──▼─────────┐  ┌─────▼──────────┐  ┌───▼──────────────┐
│  stdio     │  │ StreamableHTTP │  │  SSE (Legacy)    │
│ Transport  │  │   Transport    │  │   Transport      │
└──┬─────────┘  └─────┬──────────┘  └───┬──────────────┘
   │                  │                  │
┌──▼─────────┐  ┌─────▼──────────┐  ┌───▼──────────────┐
│  tokio     │  │    reqwest     │  │ eventsource-     │
│  process   │  │  (HTTP client) │  │    client        │
└────────────┘  └────────────────┘  └──────────────────┘
```

### 8.2 Connection Lifecycle Management

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, PartialEq)]
enum ConnectionState {
    Disconnected,
    Connecting,
    Initialized,
    Ready,
    Error(String),
}

struct ManagedConnection<T: McpTransport> {
    transport: Arc<RwLock<T>>,
    state: Arc<RwLock<ConnectionState>>,
    config: ConnectionConfig,
}

struct ConnectionConfig {
    auto_reconnect: bool,
    max_reconnect_attempts: u32,
    health_check_interval: Duration,
}

impl<T: McpTransport> ManagedConnection<T> {
    async fn connect(&self) -> Result<()> {
        let mut state = self.state.write().await;
        *state = ConnectionState::Connecting;
        drop(state);

        let mut transport = self.transport.write().await;

        match transport.initialize().await {
            Ok(_) => {
                let mut state = self.state.write().await;
                *state = ConnectionState::Ready;
                Ok(())
            }
            Err(err) => {
                let mut state = self.state.write().await;
                *state = ConnectionState::Error(err.to_string());
                Err(err)
            }
        }
    }

    async fn ensure_connected(&self) -> Result<()> {
        let state = self.state.read().await;

        match *state {
            ConnectionState::Ready => Ok(()),
            ConnectionState::Connecting => {
                // Wait for connection to complete
                drop(state);
                self.wait_for_ready().await
            }
            ConnectionState::Disconnected | ConnectionState::Error(_) => {
                drop(state);
                if self.config.auto_reconnect {
                    self.reconnect().await
                } else {
                    Err(/* connection error */)
                }
            }
            _ => Ok(()),
        }
    }

    async fn reconnect(&self) -> Result<()> {
        for attempt in 1..=self.config.max_reconnect_attempts {
            log::info!("Reconnection attempt {}/{}", attempt, self.config.max_reconnect_attempts);

            match self.connect().await {
                Ok(_) => return Ok(()),
                Err(err) if attempt >= self.config.max_reconnect_attempts => {
                    return Err(err);
                }
                Err(_) => {
                    let delay = Duration::from_secs(2u64.pow(attempt));
                    tokio::time::sleep(delay).await;
                }
            }
        }

        Err(/* max attempts exceeded */)
    }

    async fn start_health_monitoring(&self) {
        let mut interval = tokio::time::interval(self.config.health_check_interval);

        loop {
            interval.tick().await;

            if let Err(err) = self.health_check().await {
                log::warn!("Health check failed: {}", err);
                let mut state = self.state.write().await;
                *state = ConnectionState::Error(err.to_string());
                drop(state);

                if self.config.auto_reconnect {
                    let _ = self.reconnect().await;
                }
            }
        }
    }

    async fn health_check(&self) -> Result<()> {
        let transport = self.transport.read().await;

        // Implement ping/health check logic
        // For example, list tools with timeout
        tokio::time::timeout(
            Duration::from_secs(5),
            transport.list_tools()
        ).await??;

        Ok(())
    }
}
```

### 8.3 Multi-Server Connection Pool

```rust
use std::collections::HashMap;

struct McpConnectionPool {
    connections: HashMap<String, Arc<ManagedConnection<Box<dyn McpTransport>>>>,
}

impl McpConnectionPool {
    fn new() -> Self {
        Self {
            connections: HashMap::new(),
        }
    }

    async fn add_stdio_server(
        &mut self,
        name: String,
        command: String,
        args: Vec<String>,
    ) -> Result<()> {
        let transport = StdioClient::new(&command, &args).await?;
        let connection = ManagedConnection::new(
            Box::new(transport),
            ConnectionConfig {
                auto_reconnect: true,
                max_reconnect_attempts: 3,
                health_check_interval: Duration::from_secs(30),
            }
        );

        connection.connect().await?;
        self.connections.insert(name, Arc::new(connection));

        Ok(())
    }

    async fn add_http_server(
        &mut self,
        name: String,
        endpoint: String,
    ) -> Result<()> {
        let transport = StreamableHttpClient::new(endpoint);
        let connection = ManagedConnection::new(
            Box::new(transport),
            ConnectionConfig {
                auto_reconnect: true,
                max_reconnect_attempts: 5,
                health_check_interval: Duration::from_secs(60),
            }
        );

        connection.connect().await?;
        self.connections.insert(name, Arc::new(connection));

        Ok(())
    }

    async fn call_tool(
        &self,
        server_name: &str,
        tool_name: &str,
        arguments: Value,
    ) -> Result<ToolResult> {
        let connection = self.connections
            .get(server_name)
            .ok_or("Server not found")?;

        connection.ensure_connected().await?;

        let transport = connection.transport.read().await;
        transport.call_tool(tool_name, arguments).await
    }
}

// Usage example
async fn example_multi_server() -> Result<()> {
    let mut pool = McpConnectionPool::new();

    // Add Kubernetes server
    pool.add_http_server(
        "kubernetes".to_string(),
        "https://k8s-mcp.example.com/mcp".to_string(),
    ).await?;

    // Add Prometheus server
    pool.add_stdio_server(
        "prometheus".to_string(),
        "npx".to_string(),
        vec!["-y", "@pab1it0/prometheus-mcp-server"],
    ).await?;

    // Add GitHub server
    pool.add_http_server(
        "github".to_string(),
        "https://api.github.com/mcp".to_string(),
    ).await?;

    // Use different servers
    let pods = pool.call_tool("kubernetes", "list_pods", json!({
        "namespace": "default"
    })).await?;

    let metrics = pool.call_tool("prometheus", "query", json!({
        "query": "up"
    })).await?;

    let issues = pool.call_tool("github", "list_issues", json!({
        "repo": "owner/repo"
    })).await?;

    Ok(())
}
```

---

## 9. Testing Strategy

### 9.1 Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_json_rpc_request_serialization() {
        let request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list",
            "params": {}
        });

        let serialized = serde_json::to_string(&request).unwrap();
        assert!(serialized.contains("\"jsonrpc\":\"2.0\""));
        assert!(serialized.contains("\"id\":1"));
    }

    #[tokio::test]
    async fn test_circuit_breaker_opens_after_failures() {
        let breaker = CircuitBreaker::new();

        for _ in 0..5 {
            let _ = breaker.call(|| Box::pin(async {
                Err::<(), _>("failure")
            })).await;
        }

        // Circuit should now be open
        let state = breaker.state.read().await;
        assert!(matches!(*state, CircuitState::Open(_)));
    }
}
```

### 9.2 Integration Tests

```rust
#[tokio::test]
#[ignore] // Requires running MCP server
async fn test_stdio_server_integration() {
    let mut client = StdioClient::new("test-mcp-server", &[]).await.unwrap();

    // Initialize
    client.initialize().await.unwrap();

    // List tools
    let tools = client.list_tools().await.unwrap();
    assert!(!tools.is_empty());

    // Call tool
    let result = client.call_tool("echo", json!({"message": "hello"}))
        .await
        .unwrap();

    assert!(result["content"][0]["text"].as_str().unwrap().contains("hello"));
}
```

### 9.3 Mock Server for Testing

```rust
use tokio::net::TcpListener;
use axum::{Router, routing::post, Json};

async fn mock_mcp_server() -> Result<()> {
    let app = Router::new()
        .route("/mcp", post(handle_mcp_request));

    let listener = TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn handle_mcp_request(Json(request): Json<Value>) -> Json<Value> {
    let method = request["method"].as_str().unwrap();

    match method {
        "initialize" => Json(json!({
            "jsonrpc": "2.0",
            "id": request["id"],
            "result": {
                "protocolVersion": "2025-03-26",
                "capabilities": {},
                "serverInfo": {
                    "name": "mock-server",
                    "version": "0.1.0"
                }
            }
        })),
        "tools/list" => Json(json!({
            "jsonrpc": "2.0",
            "id": request["id"],
            "result": {
                "tools": [
                    {
                        "name": "echo",
                        "description": "Echo a message",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "message": {"type": "string"}
                            }
                        }
                    }
                ]
            }
        })),
        _ => Json(json!({
            "jsonrpc": "2.0",
            "id": request["id"],
            "error": {
                "code": -32601,
                "message": "Method not found"
            }
        }))
    }
}
```

---

## 10. Security Considerations

### 10.1 stdio Security

**Risks**:
- Arbitrary command execution
- Process privilege escalation
- Untrusted server executables

**Mitigations**:
- Validate server command paths
- Use allowlists for server executables
- Run servers with minimal privileges
- Sandbox server processes (containers, VMs)

### 10.2 HTTP Security

**Risks**:
- Man-in-the-middle attacks
- Session hijacking
- Credential exposure

**Mitigations**:
- Always use HTTPS in production
- Validate TLS certificates
- Secure session ID storage (encrypted, short-lived)
- Implement authentication (OAuth, JWT)
- Rate limiting and DDoS protection

### 10.3 Input Validation

**Risks**:
- Injection attacks
- Malicious tool arguments
- Resource exhaustion

**Mitigations**:
- Validate all tool arguments against schemas
- Sanitize inputs before passing to tools
- Implement request size limits
- Timeout long-running operations

---

## 11. Performance Optimization

### 11.1 Connection Pooling

- Reuse HTTP connections (reqwest handles this)
- Maintain persistent stdio processes
- Implement connection limits to prevent resource exhaustion

### 11.2 Concurrent Requests

```rust
use futures::future::join_all;

async fn parallel_tool_calls(
    client: &mut McpClient<impl McpTransport>,
    calls: Vec<(&str, Value)>,
) -> Vec<Result<ToolResult>> {
    let futures = calls.into_iter().map(|(name, args)| {
        client.call_tool(name, args)
    });

    join_all(futures).await
}
```

### 11.3 Caching

```rust
use std::time::Instant;
use std::collections::HashMap;

struct CachedResult {
    value: Value,
    timestamp: Instant,
}

struct CachingClient<T: McpTransport> {
    transport: T,
    cache: HashMap<String, CachedResult>,
    ttl: Duration,
}

impl<T: McpTransport> CachingClient<T> {
    async fn call_tool_cached(
        &mut self,
        name: &str,
        arguments: Value,
    ) -> Result<Value> {
        let cache_key = format!("{}:{}", name, arguments);

        if let Some(cached) = self.cache.get(&cache_key) {
            if cached.timestamp.elapsed() < self.ttl {
                return Ok(cached.value.clone());
            }
        }

        let result = self.transport.call_tool(name, arguments).await?;

        self.cache.insert(cache_key, CachedResult {
            value: result.clone(),
            timestamp: Instant::now(),
        });

        Ok(result)
    }
}
```

---

## 12. Deployment Patterns

### 12.1 Single Client, Multiple Servers

```yaml
# Configuration file
mcp_servers:
  - name: kubernetes
    type: http
    endpoint: https://k8s-mcp.example.com/mcp
    auth:
      type: bearer
      token_env: K8S_MCP_TOKEN

  - name: prometheus
    type: stdio
    command: npx
    args:
      - "-y"
      - "@pab1it0/prometheus-mcp-server"

  - name: github
    type: http
    endpoint: https://api.github.com/mcp
    auth:
      type: oauth
      client_id_env: GITHUB_CLIENT_ID
      client_secret_env: GITHUB_CLIENT_SECRET
```

### 12.2 Sidecar Pattern (Kubernetes)

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: app-with-mcp
spec:
  containers:
  - name: app
    image: my-app:latest
    env:
    - name: MCP_SERVER_ENDPOINT
      value: "http://localhost:8080/mcp"

  - name: mcp-server
    image: prometheus-mcp-server:latest
    ports:
    - containerPort: 8080
```

### 12.3 Gateway Pattern

```
┌──────────────┐
│  AI Client   │
└──────┬───────┘
       │
┌──────▼─────────────┐
│   MCP Gateway      │
│ (Load balancing,   │
│  authentication,   │
│  rate limiting)    │
└────┬───┬───┬───────┘
     │   │   │
┌────▼┐ ┌▼───▼┐ ┌────▼┐
│ K8s │ │Prom │ │GitHub│
│ MCP │ │ MCP │ │ MCP  │
└─────┘ └─────┘ └──────┘
```

---

## 13. Recommendations Summary

### 13.1 For Immediate Implementation

1. **Start with stdio transport**: Simpler to implement and test
2. **Use mcpr as reference**: Most complete Rust implementation
3. **Implement core protocol first**: Initialize, tools/list, tools/call
4. **Add resilience patterns**: Retry with backoff, circuit breaker
5. **Focus on DevOps servers**: Kubernetes, Prometheus, GitHub

### 13.2 Rust Crates (Priority Order)

1. **tokio** (1.x) - Async runtime (required)
2. **serde** + **serde_json** (1.x) - JSON serialization (required)
3. **reqwest** (0.12) - HTTP client for StreamableHTTP
4. **eventsource-client** (0.15) - SSE client for legacy support
5. **thiserror** (1.x) - Error handling
6. **tracing** + **tracing-subscriber** - Logging
7. **async-trait** (0.1) - Async trait abstraction

### 13.3 Architecture Priorities

1. Transport abstraction layer (support stdio, StreamableHTTP)
2. Connection lifecycle management (health checks, reconnection)
3. Error handling (three-tier model)
4. Resilience patterns (retry, circuit breaker, timeout)
5. Multi-server connection pool

### 13.4 Testing Priorities

1. JSON-RPC serialization/deserialization
2. Mock stdio server for unit tests
3. Mock HTTP server for integration tests
4. Circuit breaker behavior
5. Retry logic with backoff
6. End-to-end tests with real MCP servers

---

## 14. Next Steps

1. **Prototype stdio client**: Implement basic stdio transport with initialize and tools/list
2. **Add StreamableHTTP**: Extend to HTTP transport with session management
3. **Implement resilience**: Add retry, circuit breaker, and timeout patterns
4. **Test with real servers**: Kubernetes, Prometheus MCP servers
5. **Create connection pool**: Multi-server management
6. **Documentation**: API docs, usage examples, integration guides
7. **Publish crate**: Share with Rust community on crates.io

---

## 15. Sources

### Official MCP Documentation
- [MCP Overview](https://modelcontextprotocol.io/specification/2025-03-26/basic)
- [MCP Transports](https://modelcontextprotocol.io/specification/2025-03-26/basic/transports)
- [MCP Lifecycle](https://modelcontextprotocol.io/specification/2025-03-26/basic/lifecycle)
- [MCP Tools Specification](https://modelcontextprotocol.io/specification/2025-06-18/server/tools)
- [Base Protocol Specification](https://spec.modelcontextprotocol.io/specification/2024-11-05/basic/)

### MCP Resources
- [MCP Transport Comparison](https://mcpcat.io/guides/comparing-stdio-sse-streamablehttp/)
- [MCP Server Development Guide](https://github.com/cyanheads/model-context-protocol-resources/blob/main/guides/mcp-server-development-guide.md)
- [How MCP Works](https://howmcpworks.com/spec/)
- [MCP Concepts: Tools](https://modelcontextprotocol.info/docs/concepts/tools/)

### Rust Implementations
- [conikeec/mcpr](https://github.com/conikeec/mcpr) - Most complete implementation
- [tim-schultz/mcp-client-rs](https://github.com/tim-schultz/mcp-client-rs) - Simple client
- [prismworks-ai/prism-mcp-rs](https://github.com/prismworks-ai/prism-mcp-rs) - Enterprise-grade
- [clafollett/agenterra-rmcp](https://github.com/clafollett/agenterra-rmcp) - Community SDK

### Rust Crates
- [tokio Documentation](https://docs.rs/tokio/latest/tokio/)
- [tokio::process](https://docs.rs/tokio/latest/tokio/process/)
- [reqwest](https://github.com/seanmonstar/reqwest)
- [eventsource-client](https://github.com/launchdarkly/rust-eventsource-client)
- [docs.rs/mcpr](https://docs.rs/mcpr)

### Error Handling & Resilience
- [MCP Error Handling Guide](https://mcpcat.io/guides/error-handling-custom-mcp-servers/)
- [MCP Error Codes](https://www.mcpevals.io/blog/mcp-error-codes)
- [Resilient AI Agents](https://octopus.com/blog/mcp-timeout-retry)
- [MCP Best Practices](https://modelcontextprotocol.info/docs/best-practices/)

### DevOps MCP Servers
- [10 MCP Servers for DevOps](https://www.infoworld.com/article/4096223/10-mcp-servers-for-devops.html)
- [Top 10 DevOps MCP Servers](https://blog.globalping.io/top-10-mcp-servers-devops-developers/amp/)
- [Awesome DevOps MCP Servers](https://github.com/rohitg00/awesome-devops-mcp-servers)
- [Azure mcp-kubernetes](https://github.com/Azure/mcp-kubernetes)
- [Flux159 Kubernetes MCP](https://github.com/Flux159/mcp-server-kubernetes)
- [pab1it0 Prometheus MCP](https://github.com/pab1it0/prometheus-mcp-server)
- [idanfishman Prometheus MCP](https://github.com/idanfishman/prometheus-mcp)

---

## Appendix A: JSON-RPC 2.0 Error Codes

Standard JSON-RPC 2.0 error codes used in MCP:

| Code   | Message              | Meaning                                    |
|--------|----------------------|--------------------------------------------|
| -32700 | Parse error          | Invalid JSON received                      |
| -32600 | Invalid Request      | JSON-RPC request object invalid            |
| -32601 | Method not found     | Method does not exist or unavailable       |
| -32602 | Invalid params       | Invalid method parameters                  |
| -32603 | Internal error       | Internal JSON-RPC error                    |
| -32001 | Request timeout      | Request took too long to complete          |
| -32002 | Resource not found   | Requested resource does not exist          |
| -32003 | Tool not found       | Requested tool does not exist              |

Custom error codes should be in range -32000 to -32099.

---

## Appendix B: Tool Input Schema Examples

### Simple String Parameter
```json
{
  "type": "object",
  "properties": {
    "location": {
      "type": "string",
      "description": "City name or coordinates"
    }
  },
  "required": ["location"]
}
```

### Multiple Parameter Types
```json
{
  "type": "object",
  "properties": {
    "query": {
      "type": "string",
      "description": "PromQL query"
    },
    "start": {
      "type": "string",
      "format": "date-time",
      "description": "Start time (RFC3339)"
    },
    "end": {
      "type": "string",
      "format": "date-time",
      "description": "End time (RFC3339)"
    },
    "step": {
      "type": "integer",
      "description": "Query resolution step (seconds)",
      "default": 60
    }
  },
  "required": ["query"]
}
```

### Enum Parameters
```json
{
  "type": "object",
  "properties": {
    "namespace": {
      "type": "string",
      "description": "Kubernetes namespace"
    },
    "resource_type": {
      "type": "string",
      "enum": ["pods", "services", "deployments", "configmaps"],
      "description": "Resource type to list"
    }
  },
  "required": ["namespace", "resource_type"]
}
```

### Nested Objects
```json
{
  "type": "object",
  "properties": {
    "repository": {
      "type": "object",
      "properties": {
        "owner": {"type": "string"},
        "name": {"type": "string"}
      },
      "required": ["owner", "name"]
    },
    "filters": {
      "type": "object",
      "properties": {
        "state": {
          "type": "string",
          "enum": ["open", "closed", "all"]
        },
        "labels": {
          "type": "array",
          "items": {"type": "string"}
        }
      }
    }
  },
  "required": ["repository"]
}
```

---

## Appendix C: Example Server Configuration

### Kubernetes MCP Server (stdio)
```toml
[[servers.kubernetes]]
command = "npx"
args = ["-y", "mcp-server-kubernetes"]
env = { KUBECONFIG = "/path/to/kubeconfig" }
```

### Prometheus MCP Server (stdio)
```toml
[[servers.prometheus]]
command = "npx"
args = ["-y", "@pab1it0/prometheus-mcp-server"]
env = { PROMETHEUS_URL = "http://prometheus:9090" }
```

### GitHub MCP Server (HTTP)
```toml
[[servers.github]]
url = "https://api.github.com/mcp"
transport = "http"
headers = { Authorization = "Bearer ${GITHUB_TOKEN}" }
```

### Custom Server Configuration
```rust
#[derive(Deserialize)]
struct ServerConfig {
    name: String,
    transport: TransportType,
    #[serde(flatten)]
    transport_config: TransportConfig,
    #[serde(default)]
    env: HashMap<String, String>,
    #[serde(default)]
    headers: HashMap<String, String>,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum TransportConfig {
    Stdio { command: String, args: Vec<String> },
    Http { url: String },
}
```

---

**End of Research Report**

This comprehensive research provides all necessary information to implement a production-ready Rust MCP client library with support for multiple transport types, resilience patterns, and integration with popular DevOps MCP servers.