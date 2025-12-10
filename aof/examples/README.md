# AOF Example Agent Configurations

This directory contains comprehensive, production-ready example agent configurations demonstrating various use cases and integration patterns for the Agentic Operating Framework (AOF).

## 📁 Directory Structure

```
examples/
├── agents/                    # Agent configuration examples
│   ├── k8s-helper.yaml       # Kubernetes troubleshooting
│   ├── github-pr-reviewer.yaml # Automated code review
│   ├── slack-support-bot.yaml # Customer support
│   ├── incident-responder.yaml # On-call automation
│   ├── devops-assistant.yaml # General DevOps helper
│   ├── whatsapp-sales-bot.yaml # Sales automation
│   ├── discord-community-mod.yaml # Community management
│   └── telegram-analytics.yaml # Data analysis
├── README.md                 # This file
└── quickstart.md             # 5-minute getting started guide
```

## 🚀 Quick Start

New to AOF? Start with our [5-minute quickstart guide](./quickstart.md)!

## 📚 Example Agents Overview

### 1. Kubernetes Helper (`k8s-helper.yaml`)

**Purpose**: Expert Kubernetes troubleshooting assistant

**Model**: Claude 3.5 Sonnet (precise technical responses)

**MCP Tools**:
- `kubectl` - Kubernetes CLI access

**Use Cases**:
- Diagnose pod crashes and CrashLoopBackOff
- Debug networking issues (Services, Ingress, CNI)
- Analyze resource constraints (CPU, memory)
- Troubleshoot RBAC and security policies
- Optimize cluster performance

**Example Usage**:
```bash
aof run k8s-helper "Why is my nginx pod in CrashLoopBackOff?"
aof run k8s-helper "Check cluster health and resource usage"
aof run k8s-helper "Debug service not routing to backend pods"
```

**Key Features**:
- Low temperature (0.3) for deterministic responses
- Conversation memory for context retention
- Step-by-step investigation commands
- Root cause analysis with prevention strategies

---

### 2. GitHub PR Reviewer (`github-pr-reviewer.yaml`)

**Purpose**: Automated code review with best practices

**Model**: GPT-4 Turbo (comprehensive analysis)

**MCP Tools**:
- `github` - GitHub API integration

**Triggers**:
- `pull_request.opened`
- `pull_request.synchronize` (new commits)

**Review Checklist**:
- ✅ Code quality and maintainability
- ✅ Security vulnerabilities
- ✅ Test coverage (>80%)
- ✅ Performance optimization
- ✅ Documentation completeness

**Example Workflow**:
```bash
# Webhook triggered automatically when PR is opened
# Agent reviews code and posts comments on GitHub

# Manual review:
aof run github-pr-reviewer --trigger github-webhook --data '{
  "pull_request": {
    "number": 123,
    "html_url": "https://github.com/org/repo/pull/123"
  }
}'
```

**Key Features**:
- Automated GitHub comments
- Security scanning
- Request changes for critical issues
- Constructive feedback format

---

### 3. Slack Support Bot (`slack-support-bot.yaml`)

**Purpose**: Fast, intelligent customer support

**Model**: Claude 3 Haiku (fast, cost-effective)

**MCP Tools**:
- `slack` - Slack workspace integration

**Capabilities**:
- Answer common questions
- Escalate to human agents
- Create support tickets
- Track resolution status

**Interactive Components**:
```yaml
Buttons:
  - "Talk to Human Agent"
  - "Issue Resolved ✓"
  - "Request Callback"

Select Menu:
  - Technical Issue
  - Billing Question
  - Feature Request
  - Account Access
```

**Example Interaction**:
```
User: @support-bot My dashboard won't load
Bot: I understand the dashboard isn't loading. Let me help!

     Quick checks:
     1. Refresh the page (Ctrl+R)
     2. Clear browser cache
     3. Try incognito mode

     Which browser are you using?
     [Chrome] [Firefox] [Safari] [Other]
```

**Key Features**:
- 12-hour conversation memory
- After-hours auto-responses
- Response templates
- Escalation workflows

---

### 4. Incident Responder (`incident-responder.yaml`)

**Purpose**: Production incident automation

**Model**: Claude 3.5 Sonnet (precise SRE decisions)

**MCP Tools**:
- `kubectl` - Kubernetes management
- `aws-cli` - Cloud infrastructure
- `slack` - Team notifications

**Incident Framework (OODA Loop)**:
1. **Observe**: Gather metrics, logs, events
2. **Orient**: Classify severity, assess impact
3. **Decide**: Choose mitigation strategy
4. **Act**: Execute remediation, notify team

**Severity Levels**:
- **P0** (Critical): Complete outage, immediate escalation
- **P1** (High): Major feature down, escalate to senior engineer
- **P2** (Medium): Minor degradation, fix within 4 hours
- **P3** (Low): Cosmetic issue, schedule for sprint

**Example Webhook Payload**:
```json
{
  "alert_name": "HighErrorRate",
  "severity": "P1",
  "service": "api-gateway",
  "metric": "error_rate",
  "current_value": "15.3%",
  "threshold": "5%"
}
```

**Automated Workflows**:
- High CPU → Check deployments, analyze metrics, scale if needed
- Database errors → Verify connections, restart stale pools
- OOM → Identify leak, increase limits, create heap dump

**Safety Guardrails**:
- Confirmation required for destructive actions
- Blocked actions: `drop database`, `rm -rf /`
- Max 10 actions per incident

---

### 5. DevOps Assistant (`devops-assistant.yaml`)

**Purpose**: Comprehensive DevOps helper

**Model**: GPT-4 (balanced expertise)

**MCP Tools**:
- `kubectl` - Kubernetes
- `aws` - AWS cloud services
- `github` - Repository management
- `terraform` - Infrastructure as Code
- `postgres` - Database operations
- `slack` - Team notifications

**Expertise Areas**:
- 🏗️ Infrastructure as Code (Terraform, CloudFormation)
- 🐳 Containerization (Docker, Kubernetes, Helm)
- ☁️ Cloud platforms (AWS, GCP, Azure)
- 🔄 CI/CD pipelines (GitHub Actions, GitLab CI)
- 📊 Monitoring (Prometheus, Grafana, DataDog)
- 🔒 Security & compliance

**Example Commands**:
```bash
aof run devops-assistant "Deploy API v2.1.0 to production"
aof run devops-assistant "Create VPC with public/private subnets"
aof run devops-assistant "Why is RDS connection timing out?"
aof run devops-assistant "Optimize AWS costs"
aof run devops-assistant "Audit IAM permissions"
```

**Predefined Workflows**:
- **deploy_application**: Build → Push → Update manifests → Deploy → Verify
- **provision_infrastructure**: Validate → Plan → Approve → Apply → Monitor
- **disaster_recovery**: Assess → Restore → Verify → Failover → Report

**Environment Configs**:
```yaml
dev:
  auto_approve: true
  require_confirmation: false

production:
  auto_approve: false
  require_approval_count: 2
  notifications: slack-production-alerts
```

---

### 6. WhatsApp Sales Bot (`whatsapp-sales-bot.yaml`)

**Purpose**: Automated sales with product recommendations

**Model**: Claude 3 Haiku (fast, conversational)

**MCP Tools**:
- `whatsapp` - WhatsApp Business API
- `shopify` - Product catalog integration

**Sales Methodology (SPIN)**:
1. **Situation**: Understand customer context
2. **Problem**: Identify pain points
3. **Implication**: Explore consequences
4. **Need-Payoff**: Show value proposition

**Interactive Components**:
```yaml
Lists:
  - Electronics (Smartphones, Laptops, Accessories)
  - Fashion (Men's, Women's, Footwear)

Buttons:
  - View Cart 🛒
  - Track Order 📦
  - Chat with Agent 💬
```

**Customer Segmentation**:
- **New**: 15% off first order
- **Returning**: Free shipping on $50+
- **VIP** (>$1000 LTV): Exclusive early access
- **Cart Abandoners**: 10% off to complete order

**Automated Sequences**:
```yaml
Welcome (0h):
  "Welcome! What brings you here today?"

Cart Abandonment (24h):
  "You left items in your cart. Still interested?"

Cart Expiry (48h):
  "Use code SAVE10 for 10% off! Expires in 24h 🎁"
```

**Example Conversation**:
```
Customer: Hi
Bot: Hey there! 👋 Welcome to TechStore!
     What are you looking for today?
     🔹 Smartphones
     🔹 Laptops
     🔹 Accessories

Customer: Smartphone
Bot: Great choice! 📱 What's your budget?
     💰 Under $500
     💰 $500-$1000
     💰 Premium ($1000+)

Customer: $500-$1000
Bot: Perfect! Here are our top picks:

     1️⃣ iPhone 13 - $699
        ⭐⭐⭐⭐⭐ (4.8/5)

     2️⃣ Samsung Galaxy S23 - $799
        ⭐⭐⭐⭐⭐ (4.7/5)

     Tap a number to learn more! 👆
```

---

### 7. Discord Community Mod (`discord-community-mod.yaml`)

**Purpose**: Automated community moderation

**Model**: GPT-3.5 Turbo (cost-effective for high volume)

**MCP Tools**:
- `discord` - Discord server integration

**Primary Duties**:
- 👥 Welcome new members
- 🛡️ Enforce community guidelines
- ⚠️ Warn/timeout/ban violators
- ❓ Answer common questions
- 🎯 Guide users to appropriate channels

**Auto-Moderation Rules**:
```yaml
Spam Detection:
  - Same message 3x in 10s → Delete + 5min timeout
  - 10 messages in 60s → Delete + timeout

Invite Links:
  - Block discord.gg/ (except #partnerships)
  - Delete + warn user

Mass Mentions:
  - >5 mentions → Delete + 1h timeout
```

**Role Permissions**:
```yaml
Admin:
  - ban, kick, mute_server, bulk_delete

Moderator:
  - timeout, warn, delete_message

Helper:
  - answer_questions, welcome_members

Public:
  - help, faq, rules, report
```

**Commands**:
```
/help - Show available commands
/rules - Display server rules
/report @user [reason] - Report violation
/warn @user [reason] - Warn user (moderator)
/timeout @user [duration] - Timeout user (moderator)
/ban @user [reason] - Ban user (admin)
```

**Example Scenarios**:

**New Member**:
```
Bot: Welcome to [Server], @NewUser! 🎉

     📜 Read rules in #rules
     🎭 Pick roles in #roles
     👋 Introduce yourself in #introductions

     Have fun and be respectful! 😊
```

**Spam Detected**:
```
User: BUY CRYPTO NOW!!! (×3 in 5 seconds)
Bot: [Deletes messages]
     [Timeouts user for 5 minutes]
     #mod-log: "⚠️ User123 timed out for spam"
```

---

### 8. Telegram Analytics (`telegram-analytics.yaml`)

**Purpose**: Interactive data analysis with charts

**Model**: Claude 3.5 Sonnet (precise analysis)

**MCP Tools**:
- `telegram` - Chat interface
- `postgres` - Data queries
- `quickchart` - Chart generation

**Core Competencies**:
- 📊 Statistical analysis (descriptive, inferential, predictive)
- 📈 Data visualization (line, bar, pie, scatter, heatmap)
- 🔍 Pattern recognition (trends, anomalies, correlations)
- 💡 Insight generation (actionable recommendations)

**Commands**:
```bash
/stats revenue today
/query SELECT COUNT(*) FROM orders
/chart line revenue 7d
/dashboard sales
/trend users 30d
/compare revenue this_month vs last_month
/alert revenue < 10000
```

**Predefined Dashboards**:

**Sales Dashboard**:
- Total Revenue (24h, trend)
- Orders Today (vs yesterday)
- Average Order Value (30d)
- Conversion Rate (7d)
- Revenue Trend chart (30d)
- Top Products chart

**Engagement Dashboard**:
- Active Users (24h)
- Daily Active Users
- Monthly Active Users
- Avg Session Duration
- DAU/MAU Ratio chart

**Alerts**:
```yaml
Low Revenue:
  - Threshold: <$10,000 daily
  - Action: Notify with investigation tips

High Error Rate:
  - Threshold: >5% in 1 hour
  - Action: Alert to check logs

Signup Spike:
  - Threshold: >100 in 1 hour
  - Action: Celebrate potential viral growth
```

**Example Interaction**:
```
User: /stats revenue today
Bot: 📊 Revenue Statistics - Today

     Total: $45,234
     Orders: 127
     Avg: $356

     📈 vs Yesterday: +12.3% ⬆️
     📅 vs Last Week: +5.7% ⬆️

     🎯 On track to exceed monthly goal!

User: /chart line revenue 7d
Bot: 📈 Revenue Trend - Last 7 Days
     [Sends line chart image]

     💡 Key Insights:
     • Peak: $52K on Dec 8 (Friday)
     • Avg: $43K/day
     • Trend: +8% WoW
     • Weekends: -23% lower

     Recommendation:
     Run weekend promotions to boost Sat/Sun sales
```

---

## 🛠️ Configuration Patterns

### Model Selection Guide

| Use Case | Model | Rationale |
|----------|-------|-----------|
| Technical troubleshooting | Claude 3.5 Sonnet | Precise, methodical reasoning |
| Code review | GPT-4 Turbo | Comprehensive analysis, large context |
| Customer support | Claude 3 Haiku | Fast responses, cost-effective |
| High-volume moderation | GPT-3.5 Turbo | Balanced cost/performance |
| Data analysis | Claude 3.5 Sonnet | Statistical precision |
| Sales conversations | Claude 3 Haiku | Quick, friendly, conversational |

### Temperature Settings

- **0.1-0.3**: Deterministic (incident response, K8s troubleshooting)
- **0.4-0.6**: Balanced (DevOps, code review)
- **0.7-0.9**: Creative (sales, support, community engagement)

### Memory Configuration

```yaml
# Short-term (chat sessions)
memory:
  enabled: true
  type: short_term
  max_messages: 20-50
  context_window: 12h-48h

# Long-term (persistent context)
memory:
  enabled: true
  type: long_term
  max_messages: 100-200
  persistence: true
  namespace: incidents
```

### Trigger Patterns

**Webhooks** (external systems):
```yaml
triggers:
  - type: webhook
    path: /incident/alert
    methods: [POST]
    authentication:
      type: bearer
      token: ${WEBHOOK_TOKEN}
```

**Chat platforms** (user interactions):
```yaml
triggers:
  - type: slack
    events: [message.im, app_mention]
    channels: [support, help]
```

**GitHub events** (automation):
```yaml
triggers:
  - type: github
    events: [pull_request.opened, pull_request.synchronize]
    filters:
      branches: [main, develop]
```

---

## 🚦 Getting Started

### 1. Choose Your Use Case

Pick an example agent that matches your needs:
- **DevOps/SRE**: k8s-helper, incident-responder, devops-assistant
- **Development**: github-pr-reviewer
- **Customer-facing**: slack-support-bot, whatsapp-sales-bot
- **Community**: discord-community-mod
- **Analytics**: telegram-analytics

### 2. Configure Environment Variables

Each agent requires specific environment variables. Check the YAML file and set:

```bash
# Example for k8s-helper
export KUBECONFIG=/path/to/kubeconfig

# Example for GitHub PR reviewer
export GITHUB_TOKEN=ghp_your_token_here

# Example for Slack support bot
export SLACK_BOT_TOKEN=xoxb-your-token
export SLACK_APP_TOKEN=xapp-your-token
```

### 3. Install MCP Servers

```bash
# Install required MCP servers
npm install -g kubectl-mcp-server
npm install -g @modelcontextprotocol/server-github
npm install -g slack-mcp-server
# ... etc
```

### 4. Run the Agent

```bash
# Interactive mode
aof run k8s-helper

# Single query
aof run k8s-helper "Why is my pod crashing?"

# Webhook mode (for automated triggers)
aof serve github-pr-reviewer --port 3000
```

---

## 📖 Advanced Topics

### Combining Multiple Agents

Create complex workflows by chaining agents:

```yaml
# pipeline.yaml
name: full-deployment-pipeline
agents:
  - devops-assistant  # Provision infrastructure
  - github-pr-reviewer  # Validate code
  - k8s-helper  # Deploy to cluster
  - slack-support-bot  # Notify team
```

### Custom System Prompts

Tailor agents to your domain:

```yaml
system_prompt: |
  You are a ${COMPANY_NAME} DevOps engineer specializing in:
  - Our tech stack: ${TECH_STACK}
  - Our deployment process: ${DEPLOYMENT_PROCESS}
  - Our escalation policy: ${ESCALATION_POLICY}

  Always follow our runbooks at: ${RUNBOOK_URL}
```

### Multi-MCP Integration

Combine multiple MCP servers for powerful workflows:

```yaml
mcp_servers:
  - name: kubectl
  - name: aws
  - name: datadog
  - name: pagerduty
  - name: slack
```

### Security Best Practices

1. **Never hardcode tokens** - Use environment variables
2. **Use webhook authentication** - Bearer tokens, signatures
3. **Limit MCP server permissions** - Least privilege
4. **Enable audit logging** - Track all agent actions
5. **Set safety guardrails** - Block destructive commands

---

## 🤝 Contributing

Have a great agent example? Contribute it!

1. Create agent YAML in `examples/agents/`
2. Add comprehensive comments
3. Include example usage
4. Document all MCP dependencies
5. Submit a PR

---

## 📚 Additional Resources

- [AOF Documentation](../docs/)
- [MCP Server Registry](https://github.com/modelcontextprotocol/servers)
- [Agent Configuration Reference](../docs/config-reference.md)
- [5-Minute Quickstart](./quickstart.md)

---

## 🐛 Troubleshooting

### Common Issues

**Agent won't start**:
```bash
# Check MCP server installation
aof mcp list

# Verify environment variables
aof config check

# Test MCP connectivity
aof mcp test kubectl
```

**Webhook not triggering**:
```bash
# Verify webhook endpoint is accessible
curl -X POST http://localhost:3000/incident/alert

# Check authentication token
aof webhook test --token $WEBHOOK_TOKEN
```

**Memory not persisting**:
```yaml
# Ensure persistence is enabled
memory:
  enabled: true
  type: long_term
  persistence: true  # Add this line
```

---

## 📄 License

All example configurations are provided as-is under the MIT License. Adapt them freely for your use cases!

---

**Ready to get started?** Check out our [5-minute quickstart guide](./quickstart.md)!
