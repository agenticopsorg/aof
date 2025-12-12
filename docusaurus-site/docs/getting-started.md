# Getting Started with AOF

Get up and running with your first AI agent in 5 minutes.

## Prerequisites

### Required
- **API Key**: Get one from:
  - [Google Gemini](https://aistudio.google.com/app/apikey) (Free tier available)
  - [OpenAI](https://platform.openai.com/api-keys)
  - [Anthropic](https://console.anthropic.com/)
  - Or use [Ollama](https://ollama.ai/) locally (free, runs on your machine)
- **Terminal**: Any Unix shell (bash, zsh, fish)

### Optional
- **Rust**: Only needed if building from source
- **kubectl**: For Kubernetes-related agents
- **Docker**: For containerized deployments

## Installation

### Step 1: Install aofctl

Choose your preferred method:

#### Option A: Binary Download (Recommended)
```bash
# Automatically detects your OS and architecture, downloads and installs
curl -sSL https://aof.sh/install.sh | bash

# Verify installation
aofctl --version
```

#### Option B: Cargo Install
```bash
cargo install --git https://github.com/agenticopsorg/aof aofctl

# Verify installation
aofctl --version
```

#### Option C: Build from Source
```bash
git clone https://github.com/agenticopsorg/aof.git
cd aof
cargo build --release --package aofctl
sudo cp target/release/aofctl /usr/local/bin/

# Verify installation
aofctl --version
```

### Step 2: Configure API Keys

Set your LLM provider API key:

```bash
# Google Gemini (recommended for free tier)
export GOOGLE_API_KEY=your-api-key-here

# OR OpenAI
export OPENAI_API_KEY=sk-...

# OR Anthropic
export ANTHROPIC_API_KEY=sk-ant-...

# OR Groq
export GROQ_API_KEY=your-api-key-here

# OR Ollama (runs locally on your machine, no key needed)
# Just install: brew install ollama && ollama serve
```

**💡 Tip**: Add these to your `~/.zshrc` or `~/.bashrc` to persist across sessions.

**Getting Free API Keys:**
- **Google Gemini**: https://aistudio.google.com/app/apikey (Free tier: 60 requests/minute)
- **OpenAI**: https://platform.openai.com/api-keys (Paid, starts with free credits)
- **Anthropic**: https://console.anthropic.com/ (Paid)
- **Groq**: https://console.groq.com (Fast inference, free tier available)
- **Ollama**: https://ollama.ai (Free, runs locally)

## Create Your First Agent

### Step 3: Create an Agent YAML

Create a file called `hello-agent.yaml`:

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: hello-assistant
spec:
  model: google:gemini-2.0-flash  # Using Google Gemini (free tier)
  instructions: |
    You are a friendly assistant that helps DevOps engineers.
    Keep responses concise and practical.
```

**Available Models:**
```yaml
# Google Gemini (Free & Paid)
model: google:gemini-2.0-flash
model: google:gemini-1.5-pro

# OpenAI (Paid)
model: openai:gpt-4o
model: openai:gpt-4-turbo
model: openai:gpt-3.5-turbo

# Anthropic (Paid)
model: anthropic:claude-3-5-sonnet-20241022
model: anthropic:claude-3-5-haiku-20241022

# Groq (Free & Fast)
model: groq:llama-3.1-70b-versatile
model: groq:mixtral-8x7b-32768

# Ollama (Free, runs locally)
model: ollama:llama2
model: ollama:mistral
```

### Step 4: Run Your Agent

```bash
# Run agent with a query
aofctl run agent hello-agent.yaml --input "What's the difference between a Deployment and a StatefulSet?"
```

The agent will process your input and respond:
```
Agent: hello-assistant
Result: A Deployment manages stateless applications with replicas. StatefulSet manages
stateful applications where each pod has a stable identity and persistent storage.
```

You can also run without explicit input (uses a default message):
```bash
aofctl run agent hello-agent.yaml
```

**Note:** Interactive REPL mode (reading from stdin) is planned for a future release. Currently, use `--input` flag for programmatic interaction or pipe input via stdin.

### Step 5: Verify It Works

Your agent should respond with a clear explanation. If you see a response, congratulations! 🎉

## Add Some Tools

Let's make the agent more useful by adding shell access:

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: k8s-helper
spec:
  model: google:gemini-2.0-flash
  instructions: |
    You are a Kubernetes expert assistant. Help users run kubectl commands
    and troubleshoot their clusters. Always explain what commands do before running them.

  tools:
    - type: Shell
      config:
        allowed_commands:
          - kubectl
          - helm
        working_directory: /tmp
```

Save this as `k8s-agent.yaml` and run:

```bash
aofctl run agent k8s-agent.yaml --input "How do I check deployment status?"
```

Now try:
```bash
aofctl run agent k8s-agent.yaml --input "Show me all pods in the default namespace"
```

The agent will explain what it's doing and run `kubectl get pods -n default` to fetch the information.

## Next Steps

You now have a working AI agent! Here's where to go next:

### Learn Core Concepts
- **[Core Concepts](concepts.md)** - Understand Agents, Fleets, and Flows

### Follow Tutorials
- **[Build Your First Agent](tutorials/first-agent.md)** - Deeper dive into Agent specs
- **[Create a Slack Bot](tutorials/slack-bot.md)** - Build a production bot
- **[Incident Response Flow](tutorials/incident-response.md)** - Auto-remediation workflow

### Explore Examples
- **[Copy-paste Examples](examples/)** - Ready-to-use agent configurations

### Read Reference Docs
- **[Agent Spec](reference/agent-spec.md)** - Complete YAML reference
- **[aofctl CLI](reference/aofctl.md)** - All CLI commands

## Common Issues

### "API key not found"
```bash
# Make sure you've exported your key (example for Gemini)
echo $GOOGLE_API_KEY

# If empty, set it:
export GOOGLE_API_KEY=your-key-here

# Or for your provider:
export OPENAI_API_KEY=sk-...      # OpenAI
export ANTHROPIC_API_KEY=sk-ant-... # Anthropic
export GROQ_API_KEY=...            # Groq
```

### "Command not found: kubectl"
The agent can't use tools you don't have installed. Either:
1. Install the tool: `brew install kubectl`
2. Remove it from `allowed_commands`

### "Model not supported"
Check your provider:model format:
- ✅ `google:gemini-2.0-flash`
- ✅ `openai:gpt-4`
- ✅ `anthropic:claude-3-5-sonnet-20241022`
- ✅ `groq:llama-3.1-70b-versatile`
- ✅ `ollama:llama3`
- ❌ `gpt-4` (missing provider)
- ❌ `gemini` (incomplete - needs provider prefix)

## Getting Help

- **Documentation**: Full docs at [https://aof.sh](https://aof.sh)
- **Examples**: Check [docs/examples/](examples/) for copy-paste configs
- **Issues**: Report bugs at [GitHub Issues](https://github.com/agenticopsorg/aof/issues)
- **Discussions**: Ask questions in [GitHub Discussions](https://github.com/agenticopsorg/aof/discussions)

---

**Ready to build something real?** → [Build Your First Agent Tutorial](tutorials/first-agent.md)
