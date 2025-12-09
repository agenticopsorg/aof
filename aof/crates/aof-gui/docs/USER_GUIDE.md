# AOF Desktop - User Guide

## Overview

AOF Desktop is the graphical interface for the Agent Operating Framework (AOF). It allows operations users to configure, run, and monitor AI agents without using command-line tools.

---

## Getting Started

### Installation

1. Download the AOF Desktop installer for your platform:
   - **macOS**: `AOF Desktop.dmg`
   - **Windows**: `AOF Desktop Setup.exe`
   - **Linux**: `aof-desktop.AppImage`

2. Run the installer and follow the prompts

3. Launch AOF Desktop from your applications menu

### First Launch

On first launch, AOF Desktop will:
- Create a configuration directory at `~/.aof/`
- Initialize default settings
- Display the main dashboard

---

## Main Interface

The application has three main tabs:

| Tab | Purpose |
|-----|---------|
| **Agents** | Run and monitor AI agents |
| **Configuration** | Create and manage agent configs |
| **MCP Tools** | Connect to MCP servers and test tools |

---

## Agents Tab

### Running an Agent

1. Navigate to the **Agents** tab
2. In the **Agent Configuration** text area, enter or paste your agent YAML configuration
3. Click **Run Agent**
4. The agent will appear in the **Running Agents** list on the left

### Agent Configuration Format

```yaml
name: my-agent
description: A helpful AI assistant
model:
  provider: anthropic
  model: claude-sonnet-4-20250514
  api_key: ${ANTHROPIC_API_KEY}
system_prompt: |
  You are a helpful assistant that answers questions accurately.
tools:
  - name: search
    description: Search the web
    endpoint: http://localhost:8080/search
```

### Required Fields

| Field | Description |
|-------|-------------|
| `name` | Unique identifier for the agent |
| `model.provider` | LLM provider (anthropic, openai) |
| `model.model` | Model name |
| `system_prompt` | Instructions for the agent |

### Optional Fields

| Field | Description |
|-------|-------------|
| `description` | Human-readable description |
| `model.api_key` | API key (can use env vars with `${VAR}`) |
| `tools` | List of tools the agent can use |
| `memory` | Memory configuration |

### Monitoring Agent Output

- Select an agent from the **Running Agents** list
- Real-time output appears in the **Output** panel
- Output includes:
  - Agent thinking/reasoning
  - Tool calls and results
  - Final responses
  - Error messages

### Stopping an Agent

1. Select the agent in the **Running Agents** list
2. Click **Stop Agent**
3. The agent status changes to "stopped"

### Agent Status Indicators

| Status | Meaning |
|--------|---------|
| `running` | Agent is actively processing |
| `idle` | Agent is waiting for input |
| `stopped` | Agent has been stopped |
| `error` | Agent encountered an error |
| `completed` | Agent finished successfully |

### Clearing Completed Agents

Click **Clear Completed** to remove all stopped/completed agents from the list.

---

## Configuration Tab

### Creating a New Configuration

1. Navigate to the **Configuration** tab
2. Enter your agent YAML in the editor
3. Click **Validate** to check for errors
4. If valid, click **Save Config**
5. Enter a name when prompted

### Validation Feedback

The validation panel shows:
- **Valid**: Green checkmark, configuration is correct
- **Invalid**: Red X with error details and line numbers

### Common Validation Errors

| Error | Solution |
|-------|----------|
| "missing field 'name'" | Add a `name:` field at the top level |
| "missing field 'model'" | Add the required `model:` section |
| "invalid provider" | Use `anthropic` or `openai` |
| "invalid YAML syntax" | Check indentation and formatting |

### Loading Saved Configurations

1. Click **Load Config**
2. Select from the list of saved configurations
3. The YAML appears in the editor

### Deleting Configurations

1. Click **Delete Config**
2. Select the configuration to remove
3. Confirm deletion

### Example Configuration

Click **Generate Example** to load a sample configuration that you can modify.

---

## MCP Tools Tab

MCP (Model Context Protocol) allows agents to use external tools. This tab manages MCP server connections.

### Connecting to an MCP Server

1. Navigate to the **MCP Tools** tab
2. Enter the server endpoint URL (e.g., `http://localhost:3000`)
3. Select the transport type:
   - **HTTP**: Standard HTTP requests
   - **SSE**: Server-Sent Events for streaming
   - **Stdio**: Standard I/O for local processes
4. Click **Connect**

### Connection Status

| Status | Meaning |
|--------|---------|
| `connected` | Server is reachable |
| `disconnected` | Not connected |
| `error` | Connection failed |

### Browsing Available Tools

Once connected:
1. The **Tools** panel shows all available tools
2. Click a tool to view its details:
   - Description
   - Input parameters (JSON schema)
   - Required vs optional parameters

### Testing a Tool

1. Select a tool from the list
2. Enter test input in JSON format:
   ```json
   {
     "query": "test search",
     "limit": 10
   }
   ```
3. Click **Call Tool**
4. Results appear in the output panel

### Disconnecting

Click **Disconnect** to close the MCP server connection.

---

## Environment Variables

AOF Desktop supports environment variable substitution in configurations.

### Syntax

Use `${VARIABLE_NAME}` to reference environment variables:

```yaml
model:
  api_key: ${ANTHROPIC_API_KEY}
```

### Setting Environment Variables

**macOS/Linux:**
```bash
export ANTHROPIC_API_KEY="sk-ant-..."
```

**Windows:**
```cmd
set ANTHROPIC_API_KEY=sk-ant-...
```

### Common Variables

| Variable | Purpose |
|----------|---------|
| `ANTHROPIC_API_KEY` | Anthropic Claude API key |
| `OPENAI_API_KEY` | OpenAI API key |
| `AOF_CONFIG_DIR` | Custom config directory |

---

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Cmd/Ctrl + 1` | Switch to Agents tab |
| `Cmd/Ctrl + 2` | Switch to Configuration tab |
| `Cmd/Ctrl + 3` | Switch to MCP Tools tab |
| `Cmd/Ctrl + Enter` | Run agent / Validate config |
| `Cmd/Ctrl + S` | Save configuration |
| `Cmd/Ctrl + O` | Load configuration |

---

## Troubleshooting

### Agent Won't Start

**Symptoms**: Clicking "Run Agent" has no effect or shows error

**Solutions**:
1. Validate the configuration first (Configuration tab)
2. Check that API keys are set correctly
3. Verify network connectivity for external providers
4. Check the output panel for error messages

### MCP Connection Failed

**Symptoms**: "Connection error" when connecting to MCP server

**Solutions**:
1. Verify the server URL is correct
2. Check that the MCP server is running
3. Try a different transport type
4. Check firewall settings

### Configuration Validation Errors

**Symptoms**: Validation shows errors

**Solutions**:
1. Check YAML indentation (use 2 spaces, no tabs)
2. Ensure all required fields are present
3. Verify field names match the schema exactly
4. Check for typos in provider/model names

### Application Crashes

**Symptoms**: App closes unexpectedly

**Solutions**:
1. Check logs at `~/.aof/logs/`
2. Ensure sufficient system memory
3. Update to the latest version
4. Report the issue with logs attached

---

## Configuration Storage

### Locations

| Platform | Config Directory |
|----------|-----------------|
| macOS | `~/Library/Application Support/io.aof.desktop/` |
| Windows | `%APPDATA%\io.aof.desktop\` |
| Linux | `~/.config/io.aof.desktop/` |

### Files

| File | Purpose |
|------|---------|
| `settings.json` | Application preferences |
| `configs/` | Saved agent configurations |
| `logs/` | Application logs |

---

## Best Practices

### Security

- Never hardcode API keys in configurations
- Use environment variables for sensitive values
- Store configurations in secure locations
- Regularly rotate API keys

### Performance

- Stop agents when not in use
- Clear completed agents periodically
- Use appropriate model sizes for tasks
- Monitor memory usage for long-running agents

### Configuration Management

- Use descriptive agent names
- Add descriptions to configurations
- Version control your configurations
- Test configurations before production use

---

## Support

For issues or feature requests:

1. Check the troubleshooting section above
2. Review logs at `~/.aof/logs/`
3. Contact your system administrator
4. File an issue in the project repository

---

## Appendix: Full Configuration Schema

```yaml
# Required
name: string                    # Unique agent identifier
model:
  provider: anthropic | openai  # LLM provider
  model: string                 # Model name/ID

# Optional
description: string             # Human-readable description
model:
  api_key: string              # API key (supports env vars)
  temperature: number          # 0.0 - 1.0, default 0.7
  max_tokens: number           # Max response length
system_prompt: string          # Agent instructions
tools:                         # List of available tools
  - name: string
    description: string
    endpoint: string
    parameters: object         # JSON Schema
memory:
  type: ephemeral | persistent
  capacity: number             # Max memory items
```
