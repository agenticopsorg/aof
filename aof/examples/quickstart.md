# AOF Examples - 5-Minute Quickstart

Get your first AI agent running in less than 5 minutes! This guide walks you through setting up and running a simple support bot.

## Prerequisites

Before starting, ensure you have:

- ✅ Node.js 18+ installed
- ✅ AOF framework installed (`npm install -g aof`)
- ✅ API keys for your chosen model provider (Anthropic or OpenAI)

## Step 1: Choose Your Example (30 seconds)

For this quickstart, we'll use the **Slack Support Bot** - a simple, practical example.

```bash
# Navigate to AOF project
cd /path/to/aof

# Check examples directory
ls examples/agents/
```

You should see:
- `k8s-helper.yaml`
- `github-pr-reviewer.yaml`
- **`slack-support-bot.yaml`** ← We'll use this one
- `incident-responder.yaml`
- And more...

## Step 2: Set Up Environment Variables (1 minute)

The Slack Support Bot needs a few environment variables. Create a `.env` file:

```bash
# Create .env file in your project root
cat > .env << 'EOF'
# Model API Key (choose one)
ANTHROPIC_API_KEY=sk-ant-your-key-here
# OR
OPENAI_API_KEY=sk-your-key-here

# Slack credentials (get from https://api.slack.com/apps)
SLACK_BOT_TOKEN=xoxb-your-bot-token
SLACK_APP_TOKEN=xapp-your-app-token
EOF
```

### Getting Slack Tokens (optional - skip if testing without Slack)

1. Go to https://api.slack.com/apps
2. Click "Create New App" → "From scratch"
3. Name it "Support Bot" and choose your workspace
4. Under "OAuth & Permissions":
   - Add scopes: `chat:write`, `im:history`, `im:read`
   - Install to workspace
   - Copy the "Bot User OAuth Token" → `SLACK_BOT_TOKEN`
5. Under "Socket Mode":
   - Enable Socket Mode
   - Generate App Token → `SLACK_APP_TOKEN`

**Don't have Slack?** No problem! Skip to Step 5 for a test mode without Slack.

## Step 3: Install MCP Server (1 minute)

The support bot needs the Slack MCP server:

```bash
# Install Slack MCP server globally
npm install -g slack-mcp-server

# Verify installation
which slack-mcp-server
```

## Step 4: Review the Configuration (30 seconds)

Let's look at the agent configuration:

```bash
cat examples/agents/slack-support-bot.yaml
```

Key sections:

```yaml
name: slack-support-bot
model: claude-3-5-sonnet-20241022
provider: anthropic

instructions: |
  You are a helpful customer support specialist.
  Assist users with their inquiries professionally and efficiently.

tools:
  - slack
  - web_search

max_iterations: 10
temperature: 0.7
max_tokens: 2000
```

## Step 5: Run the Agent (1 minute)

### Option A: With Slack (full functionality)

Start the agent in server mode to listen for Slack messages:

```bash
# Load environment variables
export $(cat .env | xargs)

# Start the agent server
aof serve slack-support-bot --port 3000
```

You should see:
```
🚀 Starting agent: slack-support-bot
🔌 Connected to Slack workspace
👂 Listening for events...
📡 Server running on http://localhost:3000
```

Now, in Slack:
1. Open a DM with your bot
2. Send: "Hi, I need help with my account"
3. Watch the bot respond with helpful information!

### Option B: Test Mode (without Slack)

Don't have Slack set up yet? Test the agent locally:

```bash
# Run in test mode (simulates Slack messages)
aof run slack-support-bot "My dashboard isn't loading, can you help?"
```

The agent will respond as if in Slack:
```
I understand the dashboard isn't loading for you. Let me help!

Quick checks:
1. Try refreshing the page (Ctrl+R / Cmd+R)
2. Clear your browser cache
3. Try a different browser

Which browser are you using?
```

## Step 6: Try More Examples (2 minutes)

Now that you've run one agent, try others:

### Kubernetes Helper (No external services needed!)

```bash
# Test K8s troubleshooting (works without actual K8s cluster)
aof run k8s-helper "How do I check pod logs?"
```

### GitHub PR Reviewer

```bash
# Set GitHub token
export GITHUB_TOKEN=ghp_your_token_here

# Review a PR (replace with your PR number)
aof run github-pr-reviewer "Review PR #123 in myorg/myrepo"
```

### DevOps Assistant

```bash
# Ask for DevOps advice
aof run devops-assistant "How do I set up a CI/CD pipeline with GitHub Actions?"
```

## Common Commands Cheat Sheet

```bash
# Run agent interactively (chat mode)
aof run <agent-name>

# Run single query
aof run <agent-name> "your question here"

# Start webhook server
aof serve <agent-name> --port 3000

# List available agents
aof list

# Show agent configuration
aof show <agent-name>

# Validate agent config
aof validate <agent-name>

# Test MCP connections
aof mcp test <server-name>
```

## Next Steps

### 🎨 Customize Your Agent

Edit the YAML configuration:

```bash
# Copy example to your own configuration
cp examples/agents/slack-support-bot.yaml my-agents/custom-support-bot.yaml

# Edit the system prompt
nano my-agents/custom-support-bot.yaml
```

Change the instructions to match your business:

```yaml
instructions: |
  You are a customer support specialist for [YOUR COMPANY].

  Our products: [LIST YOUR PRODUCTS]
  Support hours: [YOUR HOURS]
  Escalation process: [YOUR PROCESS]

  Common issues:
  1. [ISSUE 1]: [SOLUTION]
  2. [ISSUE 2]: [SOLUTION]
```

### 🔧 Add More Tools

Expand capabilities with additional tools:

```yaml
name: github-assistant
model: claude-3-5-sonnet-20241022
provider: anthropic

instructions: |
  You are a GitHub assistant. Help users with repository operations,
  code reviews, and pull request management.

tools:
  - github
  - web_search
  - slack

max_iterations: 10
temperature: 0.7
```

### 📊 Configure Agent Behavior

Fine-tune agent behavior for your use case:

```yaml
name: memory-enabled-agent
model: claude-3-5-sonnet-20241022
provider: anthropic

instructions: |
  You are a helpful assistant with access to conversation context.

tools:
  - web_search

max_iterations: 5
temperature: 0.5
max_tokens: 1500
```

### 🎯 Different Models for Different Tasks

Choose the right model for your use case:

```yaml
# Fast and cost-effective
name: haiku-support-bot
model: claude-3-5-haiku-20241022
provider: anthropic
temperature: 0.5

# Advanced reasoning and code
name: sonnet-code-reviewer
model: claude-3-5-sonnet-20241022
provider: anthropic
temperature: 0.2

# Google Gemini with tools
name: k8s-helper
model: google:gemini-2.5-flash
instructions: |
  You are a Kubernetes expert assistant.
tools:
  - shell
  - kubectl
```

## Troubleshooting

### "MCP server not found"

```bash
# Reinstall MCP server globally
npm install -g slack-mcp-server

# Check it's in PATH
which slack-mcp-server
```

### "Authentication failed"

```bash
# Verify environment variables are set
echo $SLACK_BOT_TOKEN
echo $ANTHROPIC_API_KEY

# Re-export if needed
export $(cat .env | xargs)
```

### "Agent not responding"

```bash
# Check agent logs
aof logs slack-support-bot

# Validate configuration
aof validate slack-support-bot

# Test MCP connection
aof mcp test slack
```

### "Rate limit exceeded"

```yaml
# Use a cheaper/faster model for testing
name: budget-agent
model: claude-3-5-haiku-20241022  # Faster, cheaper
provider: anthropic
temperature: 0.5

instructions: "You are a helpful assistant"
tools:
  - web_search
max_iterations: 5
```

## Example Workflows

### Workflow 1: Customer Support Flow

```yaml
# Create a multi-step support workflow

1. User: "I can't login"
   Bot: Diagnoses issue, checks status page

2. Bot asks: "Are you seeing an error message?"
   User: "Yes, 'Invalid credentials'"

3. Bot: Guides through password reset
   [Sends password reset link button]

4. User clicks: "Issue Resolved ✓"
   Bot: Logs resolution, asks for feedback
```

### Workflow 2: Incident Response

```yaml
# Automated incident detection and response

1. Monitoring alert triggers webhook
   POST /incident/alert

2. incident-responder agent:
   - Classifies severity
   - Gathers diagnostics (logs, metrics)
   - Posts to #incidents Slack channel

3. Suggests mitigation:
   "Recommend rollback to v1.2.3"

4. On approval:
   - Executes rollback
   - Monitors recovery
   - Updates status page
```

### Workflow 3: Sales Automation

```yaml
# WhatsApp sales flow

1. Customer: "Hi"
   Bot: Warm greeting, shows categories

2. Customer selects: "Smartphones"
   Bot: Asks budget range

3. Customer: "$500-$1000"
   Bot: Shows top 3 products with images/reviews

4. Customer: Adds iPhone to cart
   Bot: Shows cart, applies first-order discount

5. Customer: Confirms order
   Bot: Processes payment, sends tracking
```

## Performance Tips

### 1. Choose the Right Model

| Use Case | Model | Reason |
|----------|-------|--------|
| High-volume support | claude-3-5-haiku-20241022 | Fast, cheap |
| Code review | claude-3-5-sonnet-20241022 | Quality analysis |
| Simple Q&A | claude-3-5-haiku-20241022 | Cost-effective |
| Complex reasoning | claude-3-5-sonnet-20241022 | Best quality |
| Kubernetes/CLI | google:gemini-2.5-flash | Tool execution |

### 2. Optimize Temperature

```yaml
# Deterministic (0.1-0.3): Technical tasks
temperature: 0.2

# Balanced (0.4-0.6): General support
temperature: 0.5

# Creative (0.7-0.9): Sales, marketing
temperature: 0.8
```

### 3. Use Memory Wisely

```yaml
# Short conversations (support)
max_messages: 20
context_window: 2h

# Long-running context (incidents)
max_messages: 100
context_window: 24h
persistence: true
```

### 4. Batch Operations

When possible, batch MCP operations:

```yaml
# Good: Single MCP call for multiple queries
mcp.query([query1, query2, query3])

# Bad: Separate calls
mcp.query(query1)
mcp.query(query2)
mcp.query(query3)
```

## What's Next?

- 📖 Read the [full examples README](./README.md) for all 8 agents
- 🛠️ Check the [AOF documentation](../docs/) for advanced features
- 🔍 Explore [MCP server registry](https://github.com/modelcontextprotocol/servers)
- 💬 Join the community (Discord/Slack) for help

## Quick Reference Card

**Run agent interactively**:
```bash
aof run <agent>
```

**Single query**:
```bash
aof run <agent> "your question"
```

**Start webhook server**:
```bash
aof serve <agent> --port 3000
```

**Test without external services**:
```bash
aof run <agent> --test-mode
```

**View logs**:
```bash
aof logs <agent>
```

**Validate config**:
```bash
aof validate <agent>
```

---

**🎉 Congratulations!** You've successfully run your first AOF agent in under 5 minutes!

Now explore the other 7 examples and build your own custom agents. Happy automating! 🚀
