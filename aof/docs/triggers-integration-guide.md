# AOF Triggers Integration Guide

This guide provides comprehensive documentation for integrating AOF (Agentic Ops Framework) with messaging platforms like WhatsApp, Telegram, Slack, and Discord.

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Quick Start](#quick-start)
4. [Platform Configuration](#platform-configuration)
5. [Command Reference](#command-reference)
6. [API Reference](#api-reference)
7. [Deployment Guide](#deployment-guide)
8. [Security Considerations](#security-considerations)
9. [Troubleshooting](#troubleshooting)

---

## Overview

The `aof-triggers` crate enables users to interact with AOF agents, tasks, fleets, and flows through popular messaging platforms. Users can:

- **Run agents** with natural language commands
- **Create and manage tasks** asynchronously
- **Monitor task status** in real-time
- **Control agent fleets** and workflows
- **Receive notifications** about task completion

### Supported Platforms

| Platform | Status | Features |
|----------|--------|----------|
| Telegram | ✅ Full | Text messages, inline keyboards, callback queries |
| Slack | ✅ Full | Events API, Block Kit, interactive messages |
| Discord | ✅ Full | Slash commands, embeds, components |
| WhatsApp | ✅ Full | Cloud API, interactive buttons, lists |

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Messaging Platforms                          │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐        │
│  │ Telegram │  │  Slack   │  │ Discord  │  │ WhatsApp │        │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘        │
└───────┼─────────────┼─────────────┼─────────────┼───────────────┘
        │             │             │             │
        └─────────────┴──────┬──────┴─────────────┘
                             │
                    ┌────────▼────────┐
                    │  Webhook Server │  (axum HTTP server)
                    │   POST /webhook │
                    │    /:platform   │
                    └────────┬────────┘
                             │
                    ┌────────▼────────┐
                    │ TriggerHandler  │  (Central coordinator)
                    │ - Parse message │
                    │ - Route command │
                    │ - Execute task  │
                    └────────┬────────┘
                             │
                    ┌────────▼────────┐
                    │ CommandParser   │  (Parse /commands)
                    │ - TriggerTarget │
                    │ - CommandType   │
                    │ - Arguments     │
                    └────────┬────────┘
                             │
                    ┌────────▼────────┐
                    │RuntimeOrchestra-│  (AOF Runtime)
                    │      tor        │
                    │ - Submit tasks  │
                    │ - Execute agents│
                    │ - Track status  │
                    └─────────────────┘
```

### Core Components

1. **TriggerPlatform** (`platforms/`) - Platform-specific adapters
2. **TriggerHandler** (`handler/`) - Central message routing and execution
3. **CommandParser** (`command/`) - Parse commands into structured types
4. **TriggerServer** (`server/`) - HTTP webhook server
5. **ResponseFormatter** (`response/`) - Platform-specific formatting

---

## Quick Start

### 1. Add Dependency

```toml
# Cargo.toml
[dependencies]
aof-triggers = { path = "../aof-triggers" }
aof-runtime = { path = "../aof-runtime" }
```

### 2. Create Handler

```rust
use std::sync::Arc;
use aof_triggers::{
    TriggerServer, TriggerHandler, TriggerHandlerConfig,
    TelegramPlatform, TelegramConfig,
    SlackPlatform, SlackConfig,
};
use aof_runtime::RuntimeOrchestrator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize runtime
    let orchestrator = Arc::new(RuntimeOrchestrator::new());

    // Create handler
    let mut handler = TriggerHandler::new(Arc::clone(&orchestrator));

    // Register Telegram
    let telegram = TelegramPlatform::new(TelegramConfig {
        bot_token: std::env::var("TELEGRAM_BOT_TOKEN")?,
        webhook_secret: Some(std::env::var("TELEGRAM_WEBHOOK_SECRET")?),
        bot_name: "aofbot".to_string(),
        ..Default::default()
    })?;
    handler.register_platform(Arc::new(telegram));

    // Register Slack
    let slack = SlackPlatform::new(SlackConfig {
        bot_token: std::env::var("SLACK_BOT_TOKEN")?,
        signing_secret: std::env::var("SLACK_SIGNING_SECRET")?,
        app_id: std::env::var("SLACK_APP_ID")?,
        bot_user_id: std::env::var("SLACK_BOT_USER_ID")?,
        ..Default::default()
    })?;
    handler.register_platform(Arc::new(slack));

    // Start server
    let server = TriggerServer::new(Arc::new(handler));
    server.serve().await?;

    Ok(())
}
```

### 3. Set Up Webhooks

Configure your messaging platform to send webhooks to:
```
https://your-domain.com/webhook/telegram
https://your-domain.com/webhook/slack
https://your-domain.com/webhook/discord
https://your-domain.com/webhook/whatsapp
```

---

## Platform Configuration

### Telegram

1. **Create Bot**: Message @BotFather on Telegram
2. **Get Token**: `/newbot` → copy token
3. **Set Webhook**:
   ```bash
   curl -X POST "https://api.telegram.org/bot<TOKEN>/setWebhook" \
     -H "Content-Type: application/json" \
     -d '{"url": "https://your-domain.com/webhook/telegram", "secret_token": "your-secret"}'
   ```

```rust
TelegramConfig {
    bot_token: "123456:ABC-DEF1234...".to_string(),
    webhook_url: Some("https://your-domain.com/webhook/telegram".to_string()),
    webhook_secret: Some("your-secret-token".to_string()),
    bot_name: "aofbot".to_string(),
    allowed_users: Some(vec![123456789]), // Optional whitelist
    allowed_groups: Some(vec![-100123456789]),
}
```

### Slack

1. **Create App**: https://api.slack.com/apps
2. **Enable Events API**: Subscribe to `message.im`, `app_mention`
3. **Install to Workspace**: Get OAuth token

```rust
SlackConfig {
    bot_token: "xoxb-...".to_string(),
    signing_secret: "abc123...".to_string(),
    app_id: "A0123456789".to_string(),
    bot_user_id: "U0123456789".to_string(),
    bot_name: "aofbot".to_string(),
    allowed_workspaces: None, // Allow all
    allowed_channels: None,
}
```

### Discord

1. **Create Application**: https://discord.com/developers/applications
2. **Create Bot**: Add bot to application
3. **Get Credentials**: Token, Application ID, Public Key

```rust
DiscordConfig {
    bot_token: "MTIzNDU2Nzg5...".to_string(),
    application_id: "123456789012345678".to_string(),
    public_key: "abc123...".to_string(),
    guild_ids: Some(vec!["123456789012345678".to_string()]), // Guild-specific commands
    allowed_roles: None,
}
```

### WhatsApp

1. **Create Meta Business Account**: https://business.facebook.com
2. **Set Up WhatsApp Business**: Get Phone Number ID
3. **Generate Access Token**: Permanent token from System Users

```rust
WhatsAppConfig {
    phone_number_id: "123456789012345".to_string(),
    access_token: "EAABcD...".to_string(),
    verify_token: "your-verify-token".to_string(),
    app_secret: "abc123...".to_string(),
    business_account_id: Some("987654321".to_string()),
    allowed_numbers: None, // Allow all
    api_version: "v18.0".to_string(),
}
```

---

## Command Reference

### Agent Commands

| Command | Description | Example |
|---------|-------------|---------|
| `/run agent <name> <input>` | Run an agent | `/run agent monitor Check server health` |
| `/status agent <id>` | Get agent status | `/status agent agent-123` |
| `/list agents` | List available agents | `/list agents` |

### Task Commands

| Command | Description | Example |
|---------|-------------|---------|
| `/create task <description>` | Create new task | `/create task Deploy to production` |
| `/status task <id>` | Check task status | `/status task task-abc123` |
| `/cancel task <id>` | Cancel running task | `/cancel task task-abc123` |
| `/list tasks` | List all tasks | `/list tasks` |

### Fleet Commands

| Command | Description | Example |
|---------|-------------|---------|
| `/create fleet <name>` | Create agent fleet | `/create fleet production-fleet` |
| `/status fleet <name>` | Check fleet status | `/status fleet production-fleet` |
| `/list fleets` | List all fleets | `/list fleets` |

### Flow Commands

| Command | Description | Example |
|---------|-------------|---------|
| `/run flow <name>` | Execute workflow | `/run flow deploy-pipeline` |
| `/status flow <id>` | Check flow status | `/status flow flow-xyz` |
| `/cancel flow <id>` | Cancel workflow | `/cancel flow flow-xyz` |

### Utility Commands

| Command | Description |
|---------|-------------|
| `/help` | Show help message |
| `/info` | Show system information |

---

## API Reference

### TriggerPlatform Trait

```rust
#[async_trait]
pub trait TriggerPlatform: Send + Sync {
    /// Parse incoming webhook payload
    async fn parse_message(
        &self,
        raw: &[u8],
        headers: &HashMap<String, String>,
    ) -> Result<TriggerMessage, PlatformError>;

    /// Send response to platform
    async fn send_response(
        &self,
        channel: &str,
        response: TriggerResponse,
    ) -> Result<(), PlatformError>;

    /// Get platform name
    fn platform_name(&self) -> &'static str;

    /// Verify webhook signature
    async fn verify_signature(&self, payload: &[u8], signature: &str) -> bool;

    /// Get bot name for mentions
    fn bot_name(&self) -> &str;

    /// Platform capabilities
    fn supports_threading(&self) -> bool;
    fn supports_interactive(&self) -> bool;
    fn supports_files(&self) -> bool;
}
```

### TriggerMessage

```rust
pub struct TriggerMessage {
    pub id: String,           // Unique message ID
    pub platform: String,     // Platform name
    pub channel_id: String,   // Channel/chat ID
    pub user: TriggerUser,    // User info
    pub text: String,         // Message text
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, Value>,
    pub thread_id: Option<String>,
    pub reply_to: Option<String>,
}
```

### TriggerResponse

```rust
pub struct TriggerResponse {
    pub text: String,
    pub format: ResponseFormat,   // Text, Markdown, Html, Rich
    pub status: ResponseStatus,   // Success, Error, Warning, Info
    pub attachments: Vec<Attachment>,
    pub actions: Vec<Action>,     // Interactive buttons
    pub thread_id: Option<String>,
    pub reply_to: Option<String>,
}
```

---

## Deployment Guide

### Docker

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release -p aof-triggers-server

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/aof-triggers-server /usr/local/bin/
EXPOSE 8080
CMD ["aof-triggers-server"]
```

### Environment Variables

```bash
# Server
BIND_ADDR=0.0.0.0:8080
LOG_LEVEL=info

# Telegram
TELEGRAM_BOT_TOKEN=123456:ABC...
TELEGRAM_WEBHOOK_SECRET=your-secret

# Slack
SLACK_BOT_TOKEN=xoxb-...
SLACK_SIGNING_SECRET=abc123...
SLACK_APP_ID=A0123456789
SLACK_BOT_USER_ID=U0123456789

# Discord
DISCORD_BOT_TOKEN=MTIzNDU2...
DISCORD_APPLICATION_ID=123456789012345678
DISCORD_PUBLIC_KEY=abc123...

# WhatsApp
WHATSAPP_PHONE_NUMBER_ID=123456789012345
WHATSAPP_ACCESS_TOKEN=EAABcD...
WHATSAPP_VERIFY_TOKEN=your-verify-token
WHATSAPP_APP_SECRET=abc123...
```

### Kubernetes

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: aof-triggers
spec:
  replicas: 2
  selector:
    matchLabels:
      app: aof-triggers
  template:
    metadata:
      labels:
        app: aof-triggers
    spec:
      containers:
      - name: triggers
        image: your-registry/aof-triggers:latest
        ports:
        - containerPort: 8080
        envFrom:
        - secretRef:
            name: aof-triggers-secrets
        resources:
          requests:
            memory: "256Mi"
            cpu: "100m"
          limits:
            memory: "512Mi"
            cpu: "500m"
---
apiVersion: v1
kind: Service
metadata:
  name: aof-triggers
spec:
  selector:
    app: aof-triggers
  ports:
  - port: 80
    targetPort: 8080
```

---

## Security Considerations

### Webhook Verification

All platforms support signature verification:

- **Telegram**: Secret token header (`X-Telegram-Bot-Api-Secret-Token`)
- **Slack**: HMAC-SHA256 (`X-Slack-Signature`)
- **Discord**: Ed25519 (`X-Signature-Ed25519`, `X-Signature-Timestamp`)
- **WhatsApp**: HMAC-SHA256 (`X-Hub-Signature-256`)

**Always verify signatures in production!**

### Access Control

Configure user/channel whitelists:

```rust
// Telegram: Allow specific users
allowed_users: Some(vec![123456789, 987654321]),

// Slack: Limit to specific workspaces
allowed_workspaces: Some(vec!["T0123456789".to_string()]),

// Discord: Role-based access
allowed_roles: Some(vec!["admin".to_string(), "operator".to_string()]),

// WhatsApp: Phone number whitelist
allowed_numbers: Some(vec!["+1234567890".to_string()]),
```

### Rate Limiting

The handler supports per-user task limits:

```rust
TriggerHandlerConfig {
    max_tasks_per_user: 3,  // Max concurrent tasks per user
    command_timeout_secs: 300,  // 5 minute timeout
    ..Default::default()
}
```

### Sensitive Data

- Never log full API tokens
- Redact sensitive info in responses
- Use environment variables for secrets
- Enable TLS for webhook endpoints

---

## Troubleshooting

### Common Issues

**Webhook not receiving messages**
1. Check webhook URL is publicly accessible
2. Verify SSL certificate is valid
3. Check firewall rules allow inbound traffic
4. Verify webhook is registered with platform

**Signature verification failed**
1. Ensure secret token matches platform configuration
2. Check timestamp freshness (within 5 minutes)
3. Verify payload hasn't been modified

**Messages not sending**
1. Check API token is valid and not expired
2. Verify bot has permissions to send messages
3. Check rate limits haven't been exceeded
4. Review platform-specific error messages

### Debug Mode

Enable verbose logging:

```rust
TriggerHandlerConfig {
    verbose: true,
    ..Default::default()
}
```

Or via environment:
```bash
RUST_LOG=aof_triggers=debug cargo run
```

### Health Check

```bash
curl http://localhost:8080/health
# {"status": "healthy", "timestamp": "2024-01-15T10:30:00Z"}
```

---

## Examples

### Custom Platform Adapter

```rust
use aof_triggers::{TriggerPlatform, TriggerMessage, TriggerResponse, PlatformError};

pub struct CustomPlatform {
    api_key: String,
}

#[async_trait]
impl TriggerPlatform for CustomPlatform {
    async fn parse_message(
        &self,
        raw: &[u8],
        headers: &HashMap<String, String>,
    ) -> Result<TriggerMessage, PlatformError> {
        // Parse your platform's webhook format
        todo!()
    }

    async fn send_response(
        &self,
        channel: &str,
        response: TriggerResponse,
    ) -> Result<(), PlatformError> {
        // Send response via your platform's API
        todo!()
    }

    fn platform_name(&self) -> &'static str {
        "custom"
    }

    async fn verify_signature(&self, payload: &[u8], signature: &str) -> bool {
        // Implement your signature verification
        true
    }

    fn bot_name(&self) -> &str {
        "custombot"
    }
}
```

### Interactive Response with Buttons

```rust
use aof_triggers::response::{TriggerResponseBuilder, Action, ActionStyle};

let response = TriggerResponseBuilder::new()
    .text("Task created successfully! What would you like to do?")
    .success()
    .action(Action {
        id: "view_status".to_string(),
        label: "View Status".to_string(),
        value: "status:task-123".to_string(),
        style: ActionStyle::Primary,
    })
    .action(Action {
        id: "cancel".to_string(),
        label: "Cancel".to_string(),
        value: "cancel:task-123".to_string(),
        style: ActionStyle::Danger,
    })
    .build();
```

---

## Support

- **Documentation**: https://github.com/yourusername/aof/docs
- **Issues**: https://github.com/yourusername/aof/issues
- **Discord**: https://discord.gg/aof-community

---

*Generated by AOF Hive Mind Documentation Agent*
