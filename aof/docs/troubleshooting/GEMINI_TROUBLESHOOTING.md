# Gemini API Issues and Troubleshooting

## Common Issues with Gemini Integration

### Issue 1: Empty Response on Second Query

**Symptoms:**
- First query works and returns a response
- Second and subsequent queries return "Error: Empty response from agent"
- Works fine for single queries in non-interactive mode

**Root Cause:**
Two related bugs that were fixed in commit `293f752`:

1. **Gemini Empty Parts Array**: The Gemini API response `parts` array can be empty even when the candidate exists. The original code didn't validate this condition, leading to silent failures.

2. **Missing User Message in Multi-turn Conversations**: When conversation history was restored from memory, the condition `if context.messages.is_empty()` would be false, preventing the new user query from being added to the message history. This caused the model to generate a response without seeing the new question.

**Solution:**
Fixed in commit `293f752` with two changes:

#### 1. Gemini Response Validation (`google.rs:parse_response`)
```rust
// Validate that parts are present and not empty
let content_parts = candidate
    .content
    .as_ref()
    .map(|c| &c.parts)
    .ok_or_else(|| AofError::model("Missing parts in Gemini response content"))?;

if content_parts.is_empty() {
    tracing::warn!("[GOOGLE] Empty parts array in response candidate");
    return Err(AofError::model(
        "Empty response from Gemini - no parts in content. This may indicate a model processing issue or safety filter.",
    ));
}
```

#### 2. Multi-turn Conversation Fix (`agent_executor.rs:execute`)
```rust
// Always add the current user input to the message history for this execution
// This is critical for multi-turn conversations - even if we restored history,
// we need to add the NEW user query that triggered this execution
context.add_message(MessageRole::User, context.input.clone());
```

**Testing:**
```bash
# Test interactive mode with multiple queries
./target/release/aofctl run agent testframework/gemini-agent.yaml

# You should now be able to:
# 1. Enter first query → get response
# 2. Enter second query → get response (previously failed)
# 3. Continue conversation without errors
```

---

### Issue 2: Content Filter/Safety Filter Blocks Response

**Symptoms:**
- Response shows "Content blocked by safety filter"
- Error includes category and probability (e.g., "VIOLENCE=BLOCKED")

**Root Cause:**
Gemini API blocks content based on configured safety thresholds.

**Solution:**

1. **Check API Response Log:**
   ```
   Look for: "Content blocked by safety filter: CATEGORY=PROBABILITY"
   ```

2. **Adjust Safety Settings in Agent Config:**
   ```yaml
   name: my-agent
   model: google:gemini-2.5-flash
   instructions: |
     You are a helpful assistant...
   safety_settings:
     HARM_CATEGORY_HATE_SPEECH: "BLOCK_ONLY_HIGH"
     HARM_CATEGORY_DANGEROUS_CONTENT: "BLOCK_ONLY_HIGH"
     HARM_CATEGORY_HARASSMENT: "BLOCK_ONLY_HIGH"
     HARM_CATEGORY_SEXUALLY_EXPLICIT: "BLOCK_ONLY_HIGH"
   ```

3. **Verify API Key Permissions:**
   - Ensure GOOGLE_API_KEY has access to Gemini API
   - Check quota limits in Google Cloud Console

---

### Issue 3: No Candidates in Response

**Symptoms:**
- Error: "No candidates in Gemini response - possible API error or safety filter"
- Empty response from model

**Root Cause:**
Gemini API returned a response without any candidates, which can happen due to:
- API errors or rate limiting
- Invalid request format
- Safety filters rejecting the entire response

**Solution:**

1. **Check the Logs for Full Response:**
   ```bash
   # Enable debug logging
   RUST_LOG=debug ./target/release/aofctl run agent testframework/gemini-agent.yaml

   # Look for: [GOOGLE] model.generate() FAILED or prompt_feedback messages
   ```

2. **Verify Request Format:**
   - Check that messages are properly formatted
   - Ensure tools are in correct Gemini format
   - Validate that content is not empty

3. **Check API Quota:**
   ```bash
   # Visit Google Cloud Console
   # Navigate to APIs & Services > Quotas
   # Check Generative Language API quotas
   ```

4. **Rate Limiting:**
   If you're hitting rate limits, wait before retrying or implement exponential backoff in your application.

---

## Reference: Implementation Details

### Gemini Response Structure
```json
{
  "candidates": [
    {
      "content": {
        "parts": [
          { "text": "Response text" },
          { "functionCall": { "name": "tool_name", "args": {} } }
        ],
        "role": "model"
      },
      "finishReason": "STOP",
      "safetyRatings": []
    }
  ],
  "usageMetadata": {
    "promptTokenCount": 100,
    "candidatesTokenCount": 50,
    "totalTokenCount": 150
  }
}
```

### Tool Calling with Gemini
Gemini supports function declarations and returns function calls in the `parts` array:
```rust
// Tool call appears as:
GeminiPart::FunctionCall {
    function_call: GeminiFunctionCall {
        name: "kubectl",
        args: { "command": "get pods" }
    }
}
```

### Message History Restoration
For multi-turn conversations with persistent memory:
```rust
// Step 1: Restore previous messages
restore_conversation_history(context, memory).await?;

// Step 2: ALWAYS add new user query
context.add_message(MessageRole::User, context.input.clone());

// Step 3: Execute with full history
executor.execute(&mut context).await?;
```

---

## Related Issues and Commits

- **Commit**: `293f752` - Fix empty responses and multi-turn conversation bugs
- **Related Files**:
  - `crates/aof-llm/src/provider/google.rs` - Gemini API implementation
  - `crates/aof-runtime/src/executor/agent_executor.rs` - Agent execution loop
  - `crates/aofctl/src/commands/run.rs` - TUI interactive mode

---

## Testing Checklist

- [ ] Single query returns response
- [ ] Second query returns response (multi-turn conversation works)
- [ ] Tool calling with Gemini works
- [ ] Message history is properly maintained across queries
- [ ] Empty responses are properly detected and reported
- [ ] Safety filter blocks are properly reported

---

## Reference: Compared with Working Implementation

The kubeagentics project has a working Gemini integration in:
```
/packages/agentic-engine/src/providers/gemini.ts
```

Key patterns from their implementation:
- Uses official `@google/generative-ai` SDK
- Calls `.text()` and `.functionCalls()` on response
- Handles null/empty content gracefully: `content: text || null`
- Validates before streaming: `if (text) { yield { content: text, done: false }; }`

---

## Support and Debugging

### Enable Debug Logging
```bash
RUST_LOG=aof_llm=debug,aof_runtime=debug cargo run
```

### Check API Response
The error messages include details about what Gemini returned:
```
[GOOGLE] Response parsed successfully, candidates=1
[GOOGLE] model.generate() SUCCESS: stop_reason=EndTurn, content_len=245, tool_calls=0
```

### Validate Locally
```bash
# Test with curl
curl -X POST \
  "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key=$GOOGLE_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"contents":[{"parts":[{"text":"Hello"}]}]}'
```

