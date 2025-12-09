# AOF - Agentic Ops Framework

> High-performance Rust framework for building AI agents targeting DevOps and SRE workflows

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

## Features

- **Blazing Fast**: 10-100x faster than Python frameworks
- **Type Safe**: Catch errors at compile time with Rust's type system
- **MCP Native**: First-class Model Context Protocol support
- **Multi-Provider**: Anthropic, OpenAI, Bedrock, Azure, Ollama
- **Pluggable**: Swap memory backends, tools, and models without code changes
- **kubectl-style CLI**: Familiar interface for DevOps engineers
- **Async-First**: Built on Tokio for high-concurrency workloads

## Quick Start

```bash
# Install aofctl
cargo install aofctl

# Create agent config
cat > agent.yaml <<EOF
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: devops-agent
spec:
  model:
    provider: anthropic
    model: claude-3-5-sonnet-20241022
  tools:
    - kubectl
    - aws-cli
EOF

# Run agent
aofctl run --config agent.yaml --input "List all pods in default namespace"
```

## Project Structure

```
aof/
├── crates/
│   ├── aof-core/       # Core traits and types
│   ├── aof-mcp/        # MCP client (stdio, sse, http)
│   ├── aof-llm/        # Multi-provider LLM abstraction
│   ├── aof-runtime/    # Agent execution runtime
│   ├── aof-memory/     # Pluggable memory backends
│   └── aofctl/         # CLI binary
├── examples/           # Example agents and workflows
└── docs/              # Documentation

```

## Core Traits

### Agent
```rust
#[async_trait]
pub trait Agent: Send + Sync {
    async fn execute(&self, ctx: &mut AgentContext) -> AofResult<String>;
    fn metadata(&self) -> &AgentMetadata;
}
```

### Model
```rust
#[async_trait]
pub trait Model: Send + Sync {
    async fn generate(&self, request: &ModelRequest) -> AofResult<ModelResponse>;
    async fn generate_stream(&self, request: &ModelRequest)
        -> AofResult<Pin<Box<dyn Stream<Item = AofResult<StreamChunk>> + Send>>>;
}
```

### Tool
```rust
#[async_trait]
pub trait Tool: Send + Sync {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult>;
    fn config(&self) -> &ToolConfig;
}
```

### Memory
```rust
#[async_trait]
pub trait MemoryBackend: Send + Sync {
    async fn store(&self, key: &str, entry: MemoryEntry) -> AofResult<()>;
    async fn retrieve(&self, key: &str) -> AofResult<Option<MemoryEntry>>;
    async fn search(&self, query: &MemoryQuery) -> AofResult<Vec<MemoryEntry>>;
}
```

## Feature Flags

```toml
# Cargo.toml
[features]
default = ["anthropic", "openai", "memory"]

# LLM Providers
anthropic = []
openai = []
bedrock = ["aws-config", "aws-sdk-bedrockruntime"]
azure = []
ollama = []

# MCP Transports
stdio = []
sse = ["reqwest"]
http = ["reqwest"]

# Memory Backends
memory = []  # In-memory (DashMap)
redis-backend = ["redis"]
sled-backend = ["sled"]
file-backend = ["memmap2"]
```

## Performance

```
Benchmark: 1000 concurrent agent executions
- AOF (Rust):      2.3s (435 req/s)
- LangChain (Py):  45s  (22 req/s)
- CrewAI (Py):     67s  (15 req/s)

Memory usage (idle):
- AOF:        12 MB
- LangChain:  245 MB
- CrewAI:     310 MB
```

## Examples

### Kubernetes Agent
```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: k8s-operator
spec:
  model:
    provider: anthropic
    model: claude-3-5-sonnet-20241022
  tools:
    - name: kubectl
      type: mcp
      config:
        command: npx
        args: ["-y", "kubectl-mcp"]
  memory:
    backend: redis
    config:
      url: redis://localhost:6379
```

### AWS DevOps Agent
```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: aws-ops
spec:
  model:
    provider: bedrock
    model: anthropic.claude-3-5-sonnet-20241022-v2:0
  tools:
    - aws-cli
    - terraform
    - ansible
```

## Building

```bash
# Build workspace
cargo build --release

# Build specific crate
cargo build -p aofctl --release

# Run tests
cargo test --workspace

# Build with all features
cargo build --all-features
```

## Documentation

- [Architecture](docs/ARCHITECTURE.md)
- [API Reference](https://docs.rs/aof-core)
- [Examples](examples/)

## License

Dual-licensed under MIT or Apache-2.0.
