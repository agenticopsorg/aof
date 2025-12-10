# Memory Integration in AOF Runtime

## Overview

This document describes the memory integration in the AOF runtime execution flow, addressing the critical issue where memory was created but never used.

## Implementation Details

### 1. Conversation History Restoration

**Location**: `agent_executor.rs:59-65`

Before execution starts, the runtime now:
- Attempts to restore conversation history from memory using key `agent:{name}:conversation`
- Prunes history if it exceeds the context window (100 messages)
- Preserves system messages during pruning
- Falls back gracefully if no history exists

```rust
async fn restore_conversation_history(
    &self,
    context: &mut AgentContext,
    memory: &Arc<SimpleMemory>,
) -> AofResult<()>
```

### 2. Conversation Turn Storage

**Location**: `agent_executor.rs:126-129`

After each agent turn (assistant response), the runtime:
- Stores full conversation history with key `agent:{name}:conversation`
- Stores individual turn metadata with key `agent:{name}:turn:{iteration}`
- Tracks metrics: iteration, tokens, tool calls, timestamp
- Enables semantic search and analytics

```rust
async fn store_conversation_turn(
    &self,
    context: &AgentContext,
    memory: &Arc<SimpleMemory>,
    iteration: usize,
) -> AofResult<()>
```

### 3. History Pruning

**Location**: `agent_executor.rs:339-371`

Implements intelligent conversation pruning:
- **Max messages**: 100 (configurable)
- **Strategy**: Keep all system messages + most recent messages
- **Logging**: Warns when pruning occurs
- **Future**: Token-based pruning, importance scoring

```rust
fn prune_conversation_history(&self, history: Vec<Message>) -> Vec<Message>
```

### 4. TTL Enforcement & Cleanup

**Location**: `agent_executor.rs:374-396`

Provides manual cleanup of expired entries:
- Lists all keys for the agent with prefix `agent:{name}:`
- Leverages lazy cleanup (auto-delete on retrieve)
- Can be called periodically or on-demand

```rust
pub async fn cleanup_expired_memory(&self) -> AofResult<()>
```

### 5. Semantic Search

**Location**: `agent_executor.rs:398-429`

Enables searching memory for relevant context:
- Searches all turn entries with prefix `agent:{name}:turn:`
- Returns `MemoryEntry` with metadata
- **Future**: Embedding-based semantic similarity

```rust
pub async fn search_memory(&self, query: &str) -> AofResult<Vec<MemoryEntry>>
```

## Memory Key Structure

```
agent:{agent_name}:conversation       # Full conversation history (pruned)
agent:{agent_name}:turn:{iteration}   # Individual turn metadata
```

### Turn Metadata Schema

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

## Memory Lifecycle

```
┌─────────────────────────────────────────────────────────┐
│                    Agent Execution                      │
└─────────────────────────────────────────────────────────┘
                            │
                            ▼
              ┌──────────────────────────┐
              │  Restore Conversation    │
              │  from Memory (if exists) │
              └──────────────────────────┘
                            │
                            ▼
              ┌──────────────────────────┐
              │   Execute Agent Turn      │
              │  (Model + Tools)          │
              └──────────────────────────┘
                            │
                            ▼
              ┌──────────────────────────┐
              │  Store Turn in Memory    │
              │  (History + Metadata)    │
              └──────────────────────────┘
                            │
                            ▼
              ┌──────────────────────────┐
              │   Prune if Needed        │
              │   (Max 100 messages)     │
              └──────────────────────────┘
                            │
                            ▼
              ┌──────────────────────────┐
              │   Continue or Complete   │
              └──────────────────────────┘
```

## Performance Considerations

### Lock-Free Concurrent Access

The memory backend uses `DashMap` for:
- **Zero-copy reads**: Multiple concurrent retrievals
- **Optimistic writes**: Lock-free inserts
- **Atomic operations**: Safe concurrent updates

### Lazy TTL Cleanup

- **On-retrieve**: Expired entries auto-deleted
- **Manual cleanup**: `cleanup_expired_memory()` for batch cleanup
- **No background threads**: Reduces overhead

### Memory Pruning

- **Trigger**: When history exceeds 100 messages
- **Strategy**: FIFO with system message preservation
- **Cost**: O(n) where n = history length
- **Future**: Token-based pruning for precise context windows

## Future Enhancements

### 1. Embedding-Based Semantic Search

```rust
// Future implementation
async fn search_memory_semantic(
    &self,
    query: &str,
    top_k: usize,
) -> AofResult<Vec<(MemoryEntry, f32)>> {
    // 1. Generate query embedding
    // 2. Search vector store
    // 3. Return top-k similar entries with scores
}
```

### 2. Context Window Management

```rust
// Token-based pruning
fn prune_by_tokens(&self, history: Vec<Message>, max_tokens: usize) -> Vec<Message> {
    // 1. Count tokens for each message
    // 2. Keep messages fitting in max_tokens
    // 3. Preserve important context (system, recent, tool results)
}
```

### 3. Memory Persistence

```rust
// Periodic snapshots to disk
async fn snapshot_to_disk(&self, path: &str) -> AofResult<()> {
    // 1. Serialize all agent memory
    // 2. Write to disk (JSONL, Parquet, etc.)
    // 3. Enable resume after restart
}
```

### 4. Memory Analytics

```rust
// Track memory usage patterns
pub struct MemoryStats {
    total_entries: usize,
    total_bytes: usize,
    avg_turn_tokens: f32,
    cache_hit_rate: f32,
}

pub async fn get_memory_stats(&self) -> MemoryStats {
    // Analyze memory usage
}
```

## Testing

The implementation includes:
- ✅ Unit tests for memory store/retrieve
- ✅ TTL expiry tests
- ✅ Concurrent access tests
- ✅ Pruning logic tests

### Example Test

```rust
#[tokio::test]
async fn test_conversation_persistence() {
    let executor = create_test_executor_with_memory();
    let mut context = AgentContext::new("Hello");

    // Execute first turn
    executor.execute(&mut context).await.unwrap();

    // Create new context, should restore history
    let mut new_context = AgentContext::new("Continue");
    executor.execute(&mut new_context).await.unwrap();

    // Verify history restored
    assert!(new_context.messages.len() > 1);
}
```

## Configuration

Memory behavior can be configured via:

```yaml
# agent.yaml
memory:
  backend: "in-memory"  # or "redis", "postgres", etc.
  ttl: 3600             # 1 hour default TTL
  max_messages: 100     # Prune threshold
  enable_search: true   # Enable semantic search
```

## Coordination with Claude-Flow

The memory integration works seamlessly with claude-flow hooks:

```bash
# Pre-task: Restore context
npx claude-flow@alpha hooks pre-task --description "Load conversation history"

# Post-edit: Store updates
npx claude-flow@alpha hooks post-edit --file "conversation.json" --memory-key "agent:coder:conversation"

# Session-end: Persist state
npx claude-flow@alpha hooks session-end --export-metrics true
```

## Summary

✅ **Memory is now fully integrated** into the AOF runtime execution flow:
- Conversation history restored before execution
- Turns persisted after each response
- Intelligent pruning for context window management
- TTL enforcement with lazy cleanup
- Semantic search foundation for future RAG

**Key Achievement**: Memory is no longer created and forgotten—it's actively used throughout the agent lifecycle.
