# LLM Provider Abstraction Layer

A unified, high-performance Rust interface for multiple Large Language Model providers.

## Features

- 🔄 **Unified Interface** - One trait for all providers (OpenAI, Anthropic, Google, Ollama)
- 🎯 **Model String Format** - Easy switching with `provider:model_id`
- ⚡ **High Performance** - Connection pooling, retry logic, circuit breakers
- 🔧 **Full Features** - Streaming, tools, vision, cost tracking
- 🧪 **Well Tested** - Comprehensive unit and integration tests
- 📚 **Documented** - Complete documentation and examples

## Quick Start

```rust
use my_framework::llm::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Create provider
    let config = config::OpenAiConfig::from_env().unwrap();
    let provider = OpenAiProvider::new(config, Some("gpt-4o".to_string()))?;

    // Send request
    let response = provider.chat(
        ChatRequest::new(vec![
            Message::user("What is 2+2?")
        ])
    ).await?;

    println!("{:?}", response.choices[0].message.content);
    Ok(())
}
```

## Supported Providers

| Provider | Models | Features | Cost |
|----------|--------|----------|------|
| **OpenAI** | GPT-4o, o1, o3 | Tools, Vision, Streaming | $2.50-$15/1M |
| **Anthropic** | Claude 3.5 | Tools, Vision, Streaming | $0.25-$15/1M |
| **Google** | Gemini 2.0 | Tools, Vision, Streaming | FREE-$5/1M |
| **Ollama** | llama3, mistral | Local, Free | FREE |

## Model String Format

```rust
// Explicit provider:model
"openai:gpt-4o"
"anthropic:claude-3-5-sonnet-20241022"
"google:gemini-2.0-flash-exp"
"ollama:llama3"

// Auto-detection
"gpt-4o"        // -> openai
"claude-opus"   // -> anthropic
"gemini-flash"  // -> google
```

## Usage Examples

### Using Registry (Recommended)

```rust
let config = LlmConfig::default();
let registry = create_default_registry(config).await?;

// Easy provider switching
let provider = registry.get_provider_for_model("openai:gpt-4o")?;
let response = provider.chat(request).await?;
```

### Tool Calling

```rust
let tool = Tool {
    tool_type: "function".to_string(),
    function: FunctionDefinition {
        name: "search".to_string(),
        description: "Search the web".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {"query": {"type": "string"}}
        }),
    },
};

let response = provider.chat(
    ChatRequest::new(messages).with_tools(vec![tool])
).await?;
```

### Streaming

```rust
use futures::StreamExt;

let mut stream = provider.chat_stream(
    ChatRequest::new(messages).with_stream(true)
).await?;

while let Some(chunk) = stream.next().await {
    print!("{}", chunk?.delta.content.unwrap_or_default());
}
```

### Vision

```rust
let message = Message {
    role: Role::User,
    content: MessageContent::Parts(vec![
        ContentPart::Text { text: "What's this?".to_string() },
        ContentPart::ImageUrl {
            image_url: ImageUrl {
                url: "https://...".to_string(),
                detail: Some("high".to_string())
            }
        },
    ]),
    // ...
};
```

### Cost Tracking

```rust
let response = provider.chat(request).await?;
let cost = response.usage.calculate_cost(provider.model_info());
println!("Cost: ${:.6}, Tokens: {}", cost, response.usage.total_tokens);
```

## Module Structure

```
src/llm/
├── mod.rs              # Public exports
├── core.rs             # Core traits and types
├── error.rs            # Error handling
├── config.rs           # Configuration
├── registry.rs         # Provider registry
├── tokens.rs           # Token counting
└── providers/
    ├── openai.rs       # OpenAI implementation
    ├── anthropic.rs    # Anthropic implementation
    ├── google.rs       # Google implementation
    └── ollama.rs       # Ollama implementation
```

## Environment Variables

```bash
# OpenAI
export OPENAI_API_KEY="sk-..."

# Anthropic
export ANTHROPIC_API_KEY="sk-ant-..."

# Google
export GOOGLE_API_KEY="AIza..."

# Ollama (no key needed, just run)
ollama serve
```

## Configuration

```rust
// Global configuration
let config = LlmConfig {
    default_provider: "openai".to_string(),
    timeout: Duration::from_secs(60),
    retry_enabled: true,
    max_retries: 3,
    circuit_breaker_enabled: true,
    providers: ProviderConfigs {
        openai: Some(OpenAiConfig::from_env()),
        anthropic: Some(AnthropicConfig::from_env()),
        // ...
    },
};
```

## Error Handling

```rust
match provider.chat(request).await {
    Ok(response) => { /* success */ },
    Err(LlmError::AuthenticationError(_)) => {
        eprintln!("Check API key!");
    },
    Err(LlmError::RateLimitError(_)) => {
        // Implement backoff
    },
    Err(e) if e.is_retryable() => {
        // Retry logic
    },
    Err(e) => eprintln!("Error: {}", e),
}
```

## Testing

```bash
# Unit tests
cargo test --lib llm

# Integration tests (requires API keys)
export OPENAI_API_KEY="sk-..."
cargo test --test llm_integration_test -- --ignored

# Run examples
cargo run --example llm_usage
```

## Documentation

- **[Quick Start Guide](../../docs/LLM_QUICKSTART.md)** - Get started in 5 minutes
- **[Provider Design](../../docs/LLM_PROVIDER_DESIGN.md)** - Architecture details
- **[Complete Summary](../../docs/LLM_SUMMARY.md)** - Full reference
- **[Documentation Index](../../docs/LLM_INDEX.md)** - All documentation

## Performance

- Connection pooling for reduced latency
- Automatic retry with exponential backoff
- Circuit breaker for failing providers
- Streaming for better UX
- Token counting for cost estimation

## Dependencies

```toml
tokio = { version = "1.35", features = ["full"] }
async-trait = "0.1"
reqwest = { version = "0.11", features = ["json", "stream"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
futures = "0.3"
thiserror = "1.0"
```

## Contributing

To add a new provider:

1. Create `providers/yourprovider.rs`
2. Implement `LlmProvider` trait
3. Add config to `config.rs`
4. Add detection to `registry.rs`
5. Write tests
6. Update documentation

See [LLM_SUMMARY.md](../../docs/LLM_SUMMARY.md#contributing) for details.

## Examples

See [examples/llm_usage.rs](../../examples/llm_usage.rs) for:
- Basic chat
- Registry usage
- Tool calling
- Vision/multimodal
- Streaming
- Cost tracking

## License

Same as parent project.

## Support

- Documentation: `/docs/LLM_*.md`
- Examples: `/examples/llm_*.rs`
- Tests: `/tests/llm_*.rs`
- Issues: GitHub Issues
