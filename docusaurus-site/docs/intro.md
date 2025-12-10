---
sidebar_position: 1
---

# Introduction to AOF

Welcome to the **Agentic Ops Framework (AOF)** documentation!

AOF is an AI-powered automation framework designed for **DevOps engineers**, **SREs**, and **Platform Engineers** who want to build intelligent automation for their infrastructure.

## What is AOF?

AOF lets you build **AI agents** that can:
- Execute kubectl commands and manage Kubernetes clusters
- Respond to incidents and automate remediation
- Integrate with Slack, GitHub, PagerDuty, and other tools
- Run workflows with scheduling, webhooks, and conditional logic
- Maintain context and memory across interactions

## Quick Example

Here's a simple AI agent that helps with Kubernetes:

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: k8s-helper
spec:
  model: openai:gpt-4
  instructions: |
    You are a Kubernetes expert assistant.
    Help users manage their clusters safely.
  tools:
    - type: Shell
      config:
        allowed_commands: [kubectl, helm]
```

## Key Features

### 🚀 Quick to Deploy
Install with a single command and run your first agent in minutes. YAML-based configuration makes it easy to version control and share.

### 🔧 Flexible Tools
Integrate with kubectl, shell commands, HTTP APIs, Slack, GitHub, PagerDuty, and custom MCP tools.

### 🤖 Multi-Provider AI
Use OpenAI, Anthropic, Ollama, or Groq models. Switch providers easily.

### 🔒 Safe & Controlled
Human-in-the-loop approvals, allowed command lists, and audit logging keep your infrastructure safe.

### 📊 Memory & Context
Persistent memory and RAG integration help agents learn and maintain context.

### ⚡ Production Ready
Built with Rust for performance and reliability. Supports fleets, workflows, scheduling, and webhooks.

## Getting Started

Ready to build your first agent?

1. **[Install AOF](./getting-started)** - Get up and running in 5 minutes
2. **[Learn Core Concepts](./concepts)** - Understand the fundamentals
3. **[Follow a Tutorial](./tutorials/first-agent)** - Build a real agent step-by-step
4. **[Explore Examples](./examples/)** - Copy production-ready configurations

## Use Cases

### Incident Response
Automate diagnostics, remediation, and post-incident analysis. Integrate with your monitoring and alerting stack.

→ [Learn more](./tutorials/incident-response)

### Kubernetes Management
Build intelligent K8s assistants that understand your cluster and execute safe operations.

→ [View example](./examples/)

### Workflow Automation
Create multi-step workflows with AI agents that handle complex decision-making.

→ [View spec](./reference/agentflow-spec)

## Need Help?

- **Documentation**: Browse the full docs using the sidebar
- **Examples**: Check out [production-ready examples](./examples/)
- **GitHub**: [Report issues or contribute](https://github.com/gshah/my-framework)
