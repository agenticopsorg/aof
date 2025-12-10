# Memory Integration Patch for agent_executor.rs

## Overview

Due to active file modifications (formatter/linter), the memory integration code has been prepared in `/src/memory_integration.rs`. This document describes how to apply the changes to `agent_executor.rs`.

## Changes Required

### 1. Update execute() method header (around line 63-70)

**Replace:**
```rust
/// Execute the agent with the given context
///
/// This runs the main execution loop:
/// 1. Build model request from context
/// 2. Call model.generate()
/// 3. Handle response (execute tools if needed)
/// 4. Repeat until done or max iterations
pub async fn execute(&self, context: &mut AgentContext) -> AofResult<String> {
    info!("Starting agent execution: {}", self.config.name);
    let execution_start = Instant::now();

    // Add user message if not already in history
    if context.messages.is_empty() {
        context.add_message(MessageRole::User, context.input.clone());
    }
```

**With:**
```rust
/// Execute the agent with the given context
///
/// This runs the main execution loop:
/// 1. Restore conversation history from memory (if available)
/// 2. Build model request from context
/// 3. Call model.generate()
/// 4. Store conversation turn in memory
/// 5. Handle response (execute tools if needed)
/// 6. Repeat until done or max iterations
pub async fn execute(&self, context: &mut AgentContext) -> AofResult<String> {
    info!("Starting agent execution: {}", self.config.name);
    let execution_start = Instant::now();

    // Restore conversation history from memory if available
    if let Some(memory) = &self.memory {
        self.restore_conversation_history(context, memory).await?;
    }

    // Add user message if not already in history
    if context.messages.is_empty() {
        context.add_message(MessageRole::User, context.input.clone());
    }
```

### 2. Add memory storage after assistant message (around line 134)

**Replace:**
```rust
context.messages.push(assistant_msg);

// Handle stop reason
match response.stop_reason {
```

**With:**
```rust
context.messages.push(assistant_msg);

// Store conversation turn in memory after each response
if let Some(memory) = &self.memory {
    self.store_conversation_turn(context, memory, iteration).await?;
}

// Handle stop reason
match response.stop_reason {
```

### 3. Add memory integration methods (after the memory() method, around line 383)

Add these methods to the `impl AgentExecutor` block:

```rust
/// Restore conversation history from memory
async fn restore_conversation_history(
    &self,
    context: &mut AgentContext,
    memory: &Arc<SimpleMemory>,
) -> AofResult<()> {
    let conversation_key = format!("agent:{}:conversation", self.config.name);

    if let Some(history) = memory.retrieve::<Vec<aof_core::Message>>(&conversation_key).await? {
        debug!(
            "Restored {} messages from memory for agent: {}",
            history.len(),
            self.config.name
        );

        // Prune history if it exceeds context window
        let pruned_history = self.prune_conversation_history(history);
        context.messages = pruned_history;
    } else {
        debug!("No conversation history found for agent: {}", self.config.name);
    }

    Ok(())
}

/// Store conversation turn in memory
async fn store_conversation_turn(
    &self,
    context: &AgentContext,
    memory: &Arc<SimpleMemory>,
    iteration: usize,
) -> AofResult<()> {
    let conversation_key = format!("agent:{}:conversation", self.config.name);
    let turn_key = format!("agent:{}:turn:{}", self.config.name, iteration);

    // Store full conversation history
    let conversation_value = serde_json::to_value(&context.messages)
        .map_err(|e| AofError::memory(format!("Failed to serialize messages: {}", e)))?;

    memory.store(&conversation_key, conversation_value).await?;

    // Store individual turn with metadata for semantic search
    let turn_value = serde_json::json!({
        "iteration": iteration,
        "message_count": context.messages.len(),
        "input_tokens": context.metadata.input_tokens,
        "output_tokens": context.metadata.output_tokens,
        "tool_calls": context.metadata.tool_calls,
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    });

    memory.store(&turn_key, turn_value).await?;

    debug!(
        "Stored conversation turn {} for agent: {}",
        iteration, self.config.name
    );

    Ok(())
}

/// Prune conversation history to fit context window
fn prune_conversation_history(&self, mut history: Vec<aof_core::Message>) -> Vec<aof_core::Message> {
    const MAX_MESSAGES: usize = 100;

    if history.len() > MAX_MESSAGES {
        warn!(
            "Pruning conversation history from {} to {} messages for agent: {}",
            history.len(),
            MAX_MESSAGES,
            self.config.name
        );

        // Keep system messages and most recent messages
        let system_messages: Vec<_> = history
            .iter()
            .filter(|m| m.role == MessageRole::System)
            .cloned()
            .collect();

        // Take most recent non-system messages
        let recent_messages: Vec<_> = history
            .into_iter()
            .filter(|m| m.role != MessageRole::System)
            .rev()
            .take(MAX_MESSAGES - system_messages.len())
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();

        // Combine system + recent
        history = system_messages;
        history.extend(recent_messages);
    }

    history
}

/// Cleanup expired memory entries
pub async fn cleanup_expired_memory(&self) -> AofResult<()> {
    if let Some(memory) = &self.memory {
        let prefix = format!("agent:{}:", self.config.name);

        // List all keys for this agent
        let keys = memory.list_keys().await?;
        let agent_keys: Vec<_> = keys.iter()
            .filter(|k| k.starts_with(&prefix))
            .collect();

        debug!(
            "Checking {} memory entries for expiry for agent: {}",
            agent_keys.len(),
            self.config.name
        );

        // Memory backend handles lazy cleanup on retrieve
        for key in agent_keys {
            let _: Option<serde_json::Value> = memory.retrieve(key).await?;
        }
    }

    Ok(())
}

/// Search memory for relevant context
pub async fn search_memory(
    &self,
    query: &str,
) -> AofResult<Vec<aof_core::MemoryEntry>> {
    if let Some(memory) = &self.memory {
        let prefix = format!("agent:{}:turn:", self.config.name);

        let keys = memory.list_keys().await?;
        let mut entries = Vec::new();

        for key in keys {
            if key.starts_with(&prefix) {
                if let Some(value) = memory.retrieve::<serde_json::Value>(&key).await? {
                    entries.push(aof_core::MemoryEntry::new(key, value));
                }
            }
        }

        debug!(
            "Found {} memory entries matching query '{}' for agent: {}",
            entries.len(),
            query,
            self.config.name
        );

        Ok(entries)
    } else {
        Ok(Vec::new())
    }
}
```

## Memory Key Structure

```
agent:{agent_name}:conversation       # Full conversation history (pruned)
agent:{agent_name}:turn:{iteration}   # Individual turn metadata
```

## Turn Metadata Schema

```json
{
  "iteration": 1,
  "message_count": 5,
  "input_tokens": 1234,
  "output_tokens": 567,
  "tool_calls": 2,
  "timestamp": 1702345678
}
```

## Testing

After applying the patch, run:

```bash
cd /Users/gshah/work/agentic/my-framework/aof
cargo test --package aof-runtime --lib executor::agent_executor
```

## Verification

Check that:
1. Memory is restored before execution
2. Conversation turns are stored after each response
3. History is pruned when exceeding 100 messages
4. Expired entries are cleaned up on retrieval
5. Search returns relevant context

## Manual Integration Steps

Since the file is being actively modified:

1. Wait for linter/formatter to finish
2. Open `crates/aof-runtime/src/executor/agent_executor.rs`
3. Apply the three changes described above
4. Run `cargo fmt` and `cargo clippy`
5. Run tests to verify
