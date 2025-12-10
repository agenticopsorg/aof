# AOF Testing Guide

## Quick Start

```bash
# Run all tests
cargo test --workspace

# Run tests with output
cargo test --workspace -- --nocapture

# Run specific crate tests
cargo test -p aof-core
cargo test -p aof-runtime

# Run integration tests only
cargo test --test '*'

# Run a specific test
cargo test test_agent_executor_simple
```

## Test Structure

### Unit Tests

Unit tests are located in each crate under `crates/<crate-name>/tests/`:

```
crates/
├── aof-core/
│   └── tests/
│       ├── memory_tests.rs
│       └── tool_tests.rs
├── aof-llm/
│   └── tests/
│       └── provider_tests.rs
├── aof-mcp/
│   └── tests/
│       └── transport_tests.rs
├── aof-runtime/
│   └── tests/
│       └── executor_tests.rs
└── aof-memory/
    └── tests/
        └── backend_tests.rs
```

### Integration Tests

Integration tests are in the workspace root `tests/` directory:

```
tests/
├── end_to_end_agent_test.rs
├── streaming_response_test.rs
├── platform_flow_test.rs
├── gui_command_test.rs
└── multi_tool_parallel_test.rs
```

## Writing Tests

### Unit Test Example

```rust
use aof_core::{ToolInput, ToolResult};

#[tokio::test]
async fn test_tool_execution() {
    let tool = MockTool::new("test", false);
    let input = ToolInput::new(serde_json::json!({"arg": "value"}));

    let result = tool.execute(input).await.unwrap();

    assert!(result.success);
    assert_eq!(result.data["output"], "expected");
}
```

### Integration Test Example

```rust
use aof_runtime::AgentExecutor;

#[tokio::test]
async fn test_end_to_end_execution() {
    let config = AgentConfig { /* ... */ };
    let model = Box::new(MockModel::new(/* ... */));
    let executor = AgentExecutor::new(config, model, None, None);

    let mut context = AgentContext::new("Test input");
    let response = executor.execute(&mut context).await.unwrap();

    assert!(!response.is_empty());
    assert!(context.metadata.execution_time_ms > 0);
}
```

## Mocking Patterns

### Mock Model

```rust
struct MockModel {
    responses: Vec<ModelResponse>,
    current: Mutex<usize>,
    config: ModelConfig,
}

#[async_trait]
impl Model for MockModel {
    async fn generate(&self, request: &ModelRequest) -> AofResult<ModelResponse> {
        let mut current = self.current.lock().unwrap();
        let idx = *current;
        *current += 1;

        Ok(self.responses[idx].clone())
    }

    // ... other trait methods
}
```

### Mock Tool Executor

```rust
struct MockToolExecutor {
    should_fail: bool,
}

#[async_trait]
impl ToolExecutor for MockToolExecutor {
    async fn execute_tool(&self, name: &str, input: ToolInput) -> AofResult<ToolResult> {
        if self.should_fail {
            return Ok(ToolResult::error("Tool failed"));
        }

        Ok(ToolResult::success(serde_json::json!({
            "tool": name,
            "result": "success"
        })))
    }

    // ... other trait methods
}
```

## Test Coverage

### Checking Coverage

```bash
# Install tarpaulin (one-time)
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --workspace --out Html

# Open coverage report
open tarpaulin-report.html
```

### Current Coverage Status

- **aof-core**: 90%+ (traits, types, error handling)
- **aof-llm**: 80%+ (provider factory, implementations)
- **aof-mcp**: 80%+ (transport protocols)
- **aof-runtime**: 85%+ (executor, orchestrator)
- **aof-memory**: 90%+ (backends, operations)
- **Critical paths**: 100%

## Testing Best Practices

### 1. Async Testing

```rust
#[tokio::test]
async fn test_async_function() {
    let result = async_function().await;
    assert!(result.is_ok());
}
```

### 2. Error Testing

```rust
#[tokio::test]
async fn test_error_handling() {
    let result = function_that_fails().await;
    assert!(result.is_err());

    match result {
        Err(AofError::Tool(msg)) => {
            assert!(msg.contains("expected error"));
        }
        _ => panic!("Wrong error type"),
    }
}
```

### 3. Concurrent Testing

```rust
#[tokio::test]
async fn test_concurrent_access() {
    let backend = Arc::new(InMemoryBackend::new());
    let mut handles = vec![];

    for i in 0..10 {
        let backend_clone = backend.clone();
        let handle = tokio::spawn(async move {
            let entry = MemoryEntry::new(format!("key_{}", i), json!(i));
            backend_clone.store(&format!("key_{}", i), entry).await.unwrap();
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let keys = backend.list_keys(None).await.unwrap();
    assert_eq!(keys.len(), 10);
}
```

### 4. Serialization Testing

```rust
#[test]
fn test_serialization_round_trip() {
    let original = MyStruct { /* ... */ };

    let json = serde_json::to_string(&original).unwrap();
    let deserialized: MyStruct = serde_json::from_str(&json).unwrap();

    assert_eq!(original, deserialized);
}
```

## Continuous Integration

### GitHub Actions Example

```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run tests
        run: cargo test --workspace
      - name: Check coverage
        run: |
          cargo install cargo-tarpaulin
          cargo tarpaulin --workspace --out Xml
      - name: Upload coverage
        uses: codecov/codecov-action@v3
```

## Debugging Tests

### Run single test with output

```bash
cargo test test_name -- --nocapture --test-threads=1
```

### Debug with rust-lldb

```bash
rust-lldb -- cargo test test_name
```

### Add debug logging

```rust
#[tokio::test]
async fn test_with_logging() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // Your test code...
}
```

## Performance Testing

### Benchmark Example

```rust
#[tokio::test]
async fn test_parallel_performance() {
    let start = Instant::now();

    // Execute parallel operations
    executor.execute_tools(&tool_calls).await.unwrap();

    let duration = start.elapsed();

    // Parallel should be faster than sequential
    assert!(duration.as_millis() < 250,
        "Parallel execution took {}ms, expected < 250ms",
        duration.as_millis()
    );
}
```

## Common Issues

### Test Timeouts

```rust
// Increase timeout for slow tests
#[tokio::test]
#[timeout(10000)] // 10 seconds
async fn test_slow_operation() {
    // ...
}
```

### Flaky Tests

```rust
// Use deterministic values
let entry = MemoryEntry {
    timestamp: 1234567890, // Fixed timestamp
    // ...
};

// Avoid time-based assertions
// ❌ Bad
assert_eq!(entry.timestamp, now());

// ✅ Good
assert!(entry.timestamp > 0);
```

### Resource Cleanup

```rust
#[tokio::test]
async fn test_with_cleanup() {
    let backend = InMemoryBackend::new();

    // Test operations...

    // Cleanup
    backend.clear().await.unwrap();
}
```

## Test Maintenance

### Regular Tasks

1. **Run tests before commit**: `cargo test --workspace`
2. **Check coverage monthly**: `cargo tarpaulin --workspace`
3. **Update mocks when traits change**
4. **Add tests for new features**
5. **Remove tests for deprecated features**

### Code Review Checklist

- [ ] All new code has tests
- [ ] Tests cover success and error paths
- [ ] Tests are independent (no shared state)
- [ ] Tests have descriptive names
- [ ] Mock objects are used appropriately
- [ ] Async tests use `#[tokio::test]`
- [ ] Tests clean up resources

## Resources

- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Tokio Testing](https://tokio.rs/tokio/topics/testing)
- [cargo-tarpaulin](https://github.com/xd009642/tarpaulin)
- [proptest](https://github.com/proptest-rs/proptest)
- [criterion](https://github.com/bheisler/criterion.rs)
