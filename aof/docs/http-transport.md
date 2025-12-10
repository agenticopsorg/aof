# HTTP Transport for AOF-MCP

The HTTP transport implementation for the Model Context Protocol (MCP) provides a robust, production-ready way to communicate with MCP servers over HTTP/HTTPS using JSON-RPC 2.0.

## Features

- ✅ **Full McpTransport trait implementation**
- ✅ **HTTP endpoint validation** (http/https schemes only)
- ✅ **Connection pooling** for high performance
- ✅ **Configurable timeouts and keep-alive**
- ✅ **Comprehensive HTTP error handling** (400-504 status codes)
- ✅ **JSON-RPC 2.0 error detection**
- ✅ **HTTP/2 support** (optional)
- ✅ **Detailed logging** with tracing
- ✅ **Graceful shutdown** with connection cleanup
- ✅ **Extensive unit tests**

## Usage

### Basic Example

```rust
use aof_mcp::transport::{http::HttpTransport, McpRequest, McpTransport};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create transport
    let mut transport = HttpTransport::new("http://localhost:8080/mcp");

    // Initialize (validates endpoint and creates HTTP client)
    transport.init().await?;

    // Send request
    let request = McpRequest::new("tools/list", json!({}));
    let response = transport.request(&request).await?;

    println!("Response: {:?}", response);

    // Cleanup
    transport.shutdown().await?;

    Ok(())
}
```

### Custom Configuration

```rust
use aof_mcp::transport::http::{HttpTransport, HttpConfig};
use std::time::Duration;

let config = HttpConfig {
    timeout: Duration::from_secs(60),
    pool_max_idle_per_host: 20,
    pool_idle_timeout: Duration::from_secs(120),
    http2_prior_knowledge: true,
};

let mut transport = HttpTransport::with_config(
    "https://api.example.com/mcp",
    config
);

transport.init().await?;
```

## Configuration Options

### HttpConfig

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `timeout` | `Duration` | 30s | Request timeout |
| `pool_max_idle_per_host` | `usize` | 10 | Max idle connections per host |
| `pool_idle_timeout` | `Duration` | 90s | How long idle connections stay open |
| `http2_prior_knowledge` | `bool` | false | Use HTTP/2 without negotiation |

## Connection Pooling

The HTTP transport uses `reqwest`'s built-in connection pooling for optimal performance:

- **Reuses connections** for multiple requests to the same host
- **Configurable pool size** via `pool_max_idle_per_host`
- **Automatic cleanup** of idle connections
- **TCP keep-alive** enabled (60s intervals)
- **HTTP/2 keep-alive** when enabled (30s intervals)

### Performance Benefits

- Eliminates TCP handshake overhead for subsequent requests
- Reduces TLS negotiation for HTTPS endpoints
- Lower latency for sequential operations
- Better resource utilization

## Error Handling

### HTTP Status Code Mapping

The transport maps HTTP errors to descriptive `AofError` messages:

| Status Code | Error Message |
|-------------|---------------|
| 400 | Bad request |
| 401 | Unauthorized |
| 403 | Forbidden |
| 404 | Not found |
| 408 | Request timeout |
| 429 | Rate limited |
| 500 | Server error |
| 502 | Bad gateway |
| 503 | Service unavailable |
| 504 | Gateway timeout |

### Connection Errors

```rust
// Timeout error
if e.is_timeout() {
    AofError::mcp("Request timeout after 30s")
}

// Connection failure
if e.is_connect() {
    AofError::mcp("Connection failed: ...")
}
```

### JSON-RPC Errors

The transport automatically detects JSON-RPC error responses:

```rust
if let Some(error) = &response.error {
    return Err(AofError::mcp(format!(
        "MCP error {}: {}",
        error.code,
        error.message
    )));
}
```

## Endpoint Validation

The transport validates endpoints during initialization:

```rust
// ✓ Valid endpoints
HttpTransport::new("http://localhost:8080/mcp")
HttpTransport::new("https://api.example.com/v1/mcp")

// ✗ Invalid endpoints (will error on init)
HttpTransport::new("ftp://example.com")      // Invalid scheme
HttpTransport::new("not a url")              // Malformed URL
```

## Implementation Details

### Request Flow

1. **Validate** endpoint URL (on init)
2. **Build** HTTP client with pooling (on init)
3. **Serialize** request to JSON
4. **Send** POST request with headers:
   - `Content-Type: application/json`
   - `Accept: application/json`
5. **Check** HTTP status code
6. **Parse** JSON-RPC response
7. **Validate** no JSON-RPC errors
8. **Return** response

### Thread Safety

- Uses `Arc<Client>` for shared ownership
- Client is thread-safe and cloneable
- Safe to use from multiple async tasks
- Implements `Send + Sync` via McpTransport trait

### Shutdown Behavior

```rust
transport.shutdown().await?;
```

- Drops the HTTP client
- Closes all pooled connections
- Releases system resources
- Safe to call multiple times

## Testing

The implementation includes comprehensive unit tests:

```bash
# Run HTTP transport tests
cargo test -p aof-mcp --features http

# Run specific test
cargo test -p aof-mcp --features http test_init_success
```

### Test Coverage

- ✅ Endpoint validation (valid/invalid schemes and URLs)
- ✅ Configuration (default and custom)
- ✅ Initialization success
- ✅ Uninitialized transport error
- ✅ Shutdown behavior
- ✅ Request/response flow (via integration tests)

## Comparison with Stdio Transport

| Feature | Stdio | HTTP |
|---------|-------|------|
| Process management | ✓ | - |
| Connection pooling | - | ✓ |
| Remote endpoints | - | ✓ |
| Local processes | ✓ | - |
| Keep-alive | - | ✓ |
| HTTP/2 support | - | ✓ |

## Feature Flags

Enable HTTP transport in `Cargo.toml`:

```toml
[dependencies]
aof-mcp = { version = "0.1", features = ["http"] }

# Or enable all transports
aof-mcp = { version = "0.1", features = ["all-transports"] }
```

## Dependencies

- `reqwest` - HTTP client with connection pooling
- `url` - URL parsing and validation
- `serde_json` - JSON serialization
- `tracing` - Structured logging

## Example Application

See `examples/http_transport_example.rs` for a complete example:

```bash
cargo run --example http_transport_example --features http
```

## Production Considerations

### Timeouts

- Set appropriate timeouts based on expected response times
- Consider network latency for remote endpoints
- Use shorter timeouts for health checks

### Connection Pool

- Increase `pool_max_idle_per_host` for high-throughput scenarios
- Adjust `pool_idle_timeout` based on server keep-alive settings
- Monitor connection pool utilization

### Error Handling

- Implement retry logic for transient errors (503, 504)
- Use exponential backoff for rate limiting (429)
- Log errors with context for debugging

### Security

- Always use HTTPS for production endpoints
- Validate SSL certificates (enabled by default)
- Consider authentication headers if required

## Troubleshooting

### "Transport not initialized"

Ensure you call `init()` before `request()`:

```rust
transport.init().await?;
let response = transport.request(&request).await?;
```

### Connection Refused

- Verify the server is running
- Check firewall rules
- Ensure correct host and port

### Timeout Errors

- Increase timeout in `HttpConfig`
- Check network connectivity
- Verify server responsiveness

### SSL/TLS Errors

- Ensure HTTPS endpoint has valid certificate
- For self-signed certs, consider using custom reqwest configuration

## Future Enhancements

Potential improvements for future versions:

- [ ] Authentication middleware (Bearer tokens, API keys)
- [ ] Request/response interceptors
- [ ] Retry policies with exponential backoff
- [ ] Circuit breaker pattern
- [ ] Metrics and telemetry integration
- [ ] Streaming support for large payloads
- [ ] WebSocket upgrade option
- [ ] Custom TLS configuration

## References

- [JSON-RPC 2.0 Specification](https://www.jsonrpc.org/specification)
- [Model Context Protocol](https://modelcontextprotocol.io)
- [reqwest Documentation](https://docs.rs/reqwest)
- [HTTP Status Codes](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status)
