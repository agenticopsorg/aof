# AOF Architecture

## Overview

AOF (Agentic Ops Framework) is a high-performance Rust framework designed for DevOps and SRE engineers to build agentic AI workflows with minimal overhead and maximum speed.

## Design Principles

1. **Zero-Cost Abstractions**: Traits compile to static dispatch where possible
2. **Async-First**: Built on Tokio for high-concurrency workloads
3. **Pluggable Backends**: Swap memory, LLM providers, and tools without code changes
4. **Type Safety**: Leverage Rust's type system to catch errors at compile time
5. **Performance**: Beat Python frameworks (LangChain, CrewAI) in speed and memory

## Crate Structure

### aof-core
Foundation types and traits. Zero dependencies on specific implementations.

- `Agent` trait: Core agent abstraction
- `Model` trait: LLM provider interface
- `Tool` trait: Executable capabilities
- `Memory` trait: State persistence
- Error types and result handling

### aof-mcp
Model Context Protocol client with multiple transport support.

- Stdio transport (default)
- SSE transport (server-sent events)
- HTTP transport (REST API)
- Tool discovery and execution
- Connection pooling and retries

### aof-llm
Multi-provider LLM abstraction layer.

- Anthropic Claude (default)
- OpenAI GPT
- AWS Bedrock (optional)
- Azure OpenAI (optional)
- Ollama (optional)
- Streaming responses
- Token counting and optimization

### aof-runtime
Agent execution runtime with task orchestration.

- Agent lifecycle management
- Tool execution pipeline
- Memory coordination
- Parallel task execution
- Error recovery and retries

### aof-memory
Pluggable memory backends for agent state.

- In-memory (default, DashMap)
- Redis (optional)
- Sled (optional, embedded DB)
- File-based (optional, mmap)
- TTL support
- Search and filtering

### aofctl
kubectl-style CLI for agent orchestration.

```bash
# Run agent with config
aofctl run --config agent.yaml --input "Deploy nginx to k8s"

# List available tools
aofctl tools --server npx --args claude-flow mcp start

# Apply agent configuration
aofctl apply --file workflow.yaml

# Get agent status
aofctl get agent my-devops-agent
```

## Performance Optimizations

1. **Zero-Copy Parsing**: Use `bytes::Bytes` for network I/O
2. **Connection Pooling**: Reuse HTTP connections
3. **Parallel Execution**: Tokio tasks for concurrent tool calls
4. **Smart Caching**: DashMap for lock-free concurrent access
5. **Minimal Allocations**: Use `&str` and borrowing where possible
6. **Compile-Time Dispatch**: Trait objects only where necessary

## YAML Configuration

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: devops-agent
spec:
  model:
    provider: anthropic
    model: claude-3-5-sonnet-20241022
    temperature: 0.7
  tools:
    - kubectl
    - aws-cli
    - github
  memory:
    backend: redis
    ttl: 3600
  maxIterations: 10
```

## Comparison to Python Frameworks

| Feature | AOF (Rust) | LangChain | CrewAI |
|---------|------------|-----------|--------|
| Startup time | <50ms | ~2s | ~3s |
| Memory overhead | ~10MB | ~200MB | ~300MB |
| Concurrent requests | 10k+ | ~100 | ~50 |
| Token efficiency | High | Medium | Low |
| Type safety | Compile-time | Runtime | Runtime |

## Future Roadmap

- [ ] Vector search integration (Qdrant, Pinecone)
- [ ] Distributed tracing (OpenTelemetry)
- [ ] Kubernetes operator
- [ ] WebAssembly runtime for sandboxed tools
- [ ] GraphQL API for agent management
